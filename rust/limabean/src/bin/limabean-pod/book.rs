use beancount_parser_lima::{
    self as parser, BeancountParser, BeancountSources, ParseError, ParseSuccess, Span, Spanned,
};
use limabean_booking::{Booking, Bookings, Interpolated, is_supported_method};
use std::{io::Write, iter::empty, path::Path};

use rust_decimal::Decimal;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt::Debug,
};
use tabulator::{Align, Cell};
use time::Date;

use crate::{
    format::{GUTTER_MEDIUM, beancount::write_booked_as_beancount, edn::write_booked_as_edn},
    plugins::InternalPlugins,
};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(crate) enum Format {
    Beancount,
    Edn,
}

pub(crate) fn write_bookings_from<W1, W2>(
    path: &Path,
    format: Format,
    out_w: W1,
    error_w: W2,
) -> Result<(), crate::Error>
where
    W1: Write + Copy,
    W2: Write + Copy,
{
    let sources = BeancountSources::try_from(path)
        .map_err(|e| crate::Error::CannotReadFile(path.into(), e))?;
    let parser = BeancountParser::new(&sources);

    match parser.parse() {
        Ok(ParseSuccess {
            directives,
            options,
            plugins,
            mut warnings,
        }) => {
            let internal_plugins = plugins.iter().collect::<InternalPlugins>();
            for unknown in &internal_plugins.unknown {
                warnings.push(unknown.warning("unknown plugin"));
            }

            let default_booking = Booking::default();
            let default_booking_option = if let Some(booking_method) = options.booking_method() {
                let booking = Into::<Booking>::into(*booking_method.item());
                if is_supported_method(booking) {
                    booking
                } else {
                    warnings.push(booking_method.warning(format!(
                        "Unsupported booking method, falling back to {default_booking}"
                    )));
                    default_booking
                }
            } else {
                default_booking
            };

            sources.write_errors_or_warnings(error_w, warnings)?;

            match Loader::new(default_booking_option, &options, &internal_plugins)
                .collect(&directives)
            {
                Ok(LoadSuccess {
                    directives,
                    warnings,
                }) => {
                    if !warnings.is_empty() {
                        sources.write_errors_or_warnings(error_w, warnings)?;
                    }

                    match format {
                        Format::Beancount => {
                            write_booked_as_beancount(&directives, &options, out_w)
                        }
                        Format::Edn => write_booked_as_edn(&directives, &options, out_w)
                            .map_err(Into::<crate::Error>::into),
                    }
                }
                Err(LoadError { errors, .. }) => {
                    sources
                        .write_errors_or_warnings(error_w, errors)
                        .map_err(Into::<crate::Error>::into)?;
                    Err(crate::Error::FatalAndAlreadyExplained)
                }
            }
        }

        Err(ParseError { errors, warnings }) => {
            sources.write_errors_or_warnings(error_w, errors)?;
            sources.write_errors_or_warnings(error_w, warnings)?;
            Err(crate::Error::FatalAndAlreadyExplained)
        }
    }
}

#[derive(Debug)]
pub(crate) struct Loader<'a, 'b, T> {
    directives: Vec<Directive<'a>>,
    // hashbrown HashMaps are used here for their Entry API, which is still unstable in std::collections::HashMap
    open_accounts: hashbrown::HashMap<&'a str, Span>,
    closed_accounts: hashbrown::HashMap<&'a str, Span>,
    accounts: HashMap<&'a str, AccountBuilder<'a>>,
    currency_usage: hashbrown::HashMap<parser::Currency<'a>, i32>,
    internal_plugins: &'b InternalPlugins,
    default_booking: Booking,
    tolerance: T,
    warnings: Vec<parser::AnnotatedWarning>,
}

pub(crate) struct LoadSuccess<'a> {
    pub(crate) directives: Vec<Directive<'a>>,
    pub(crate) warnings: Vec<parser::AnnotatedWarning>,
}

pub(crate) struct LoadError {
    pub(crate) errors: Vec<parser::AnnotatedError>,
}

impl<'a, 'b, T> Loader<'a, 'b, T> {
    pub(crate) fn new(
        default_booking: Booking,
        tolerance: T,
        internal_plugins: &'b InternalPlugins,
    ) -> Self {
        Self {
            directives: Vec::default(),
            open_accounts: hashbrown::HashMap::default(),
            closed_accounts: hashbrown::HashMap::default(),
            accounts: HashMap::default(),
            currency_usage: hashbrown::HashMap::default(),
            internal_plugins,
            default_booking,
            tolerance,
            warnings: Vec::default(),
        }
    }

    // generate any errors before building
    fn validate(
        self,
        mut errors: Vec<parser::AnnotatedError>,
    ) -> Result<LoadSuccess<'a>, LoadError> {
        let Self {
            directives,
            accounts,
            warnings,
            ..
        } = self;

        // check for unused pad directives
        for account in accounts.values() {
            if let Some(pad_idx) = &account.pad_idx {
                errors.push(directives[*pad_idx].parsed.error("unused").into())
            }
        }

        if errors.is_empty() {
            Ok(LoadSuccess {
                directives,
                warnings,
            })
        } else {
            Err(LoadError { errors })
        }
    }

    pub(crate) fn collect<I>(mut self, directives: I) -> Result<LoadSuccess<'a>, LoadError>
    where
        I: IntoIterator<Item = &'a Spanned<parser::Directive<'a>>>,
        T: limabean_booking::Tolerance<Types = limabean_booking::LimaParserBookingTypes<'a>>,
    {
        let mut errors = Vec::default();

        for directive in directives {
            match self.directive(directive) {
                Ok(loaded) => {
                    self.directives.push(Directive {
                        parsed: directive,
                        loaded,
                    });
                }
                Err(e) => {
                    errors.push(e);
                }
            }
        }

        self.validate(errors)
    }

    fn directive(
        &mut self,
        directive: &'a Spanned<parser::Directive<'a>>,
    ) -> Result<DirectiveVariant<'a>, parser::AnnotatedError>
    where
        T: limabean_booking::Tolerance<Types = limabean_booking::LimaParserBookingTypes<'a>>,
    {
        use parser::DirectiveVariant::*;

        let date = *directive.date().item();
        let element = into_spanned_element(directive);

        match directive.variant() {
            Transaction(transaction) => self.transaction(transaction, date, element),
            Price(_price) => Ok(DirectiveVariant::NA),
            Balance(balance) => self.balance(balance, date, element),
            Open(open) => self.open(open, date, element),
            Close(close) => self.close(close, date, element),
            Commodity(_commodity) => Ok(DirectiveVariant::NA),
            Pad(pad) => self.pad(pad, date, element),
            Document(_document) => Ok(DirectiveVariant::NA),
            Note(_note) => Ok(DirectiveVariant::NA),
            Event(_event) => Ok(DirectiveVariant::NA),
            Query(_query) => Ok(DirectiveVariant::NA),
            Custom(_custom) => Ok(DirectiveVariant::NA),
        }
    }

    fn transaction(
        &mut self,
        transaction: &'a parser::Transaction<'a>,
        date: Date,
        element: parser::Spanned<Element>,
    ) -> Result<DirectiveVariant<'a>, parser::AnnotatedError>
    where
        T: limabean_booking::Tolerance<Types = limabean_booking::LimaParserBookingTypes<'a>>,
    {
        let description = transaction.payee().map_or_else(
            || {
                transaction
                    .narration()
                    .map_or("post", |narration| narration.item())
            },
            |payee| payee.item(),
        );

        let postings = transaction.postings().collect::<Vec<_>>();

        let auto_accounts = if self.internal_plugins.auto_accounts {
            let mut auto_accounts = HashSet::default();

            for account in postings.iter().map(|posting| posting.account()) {
                let account_name = account.item().as_ref();
                if !self.accounts.contains_key(account_name) {
                    auto_accounts.insert(account_name);

                    self.accounts.insert(
                        account_name,
                        AccountBuilder::new(empty(), self.default_booking, *account.span()),
                    );
                    self.open_accounts.insert(account_name, *account.span());
                }
            }
            auto_accounts
        } else {
            HashSet::default()
        };

        let BookedPostingsAndPrices { postings, prices } =
            self.book(&element, date, &postings, description)?;

        Ok(DirectiveVariant::Transaction(Transaction {
            postings,
            _prices: prices,
            auto_accounts,
        }))
    }

    fn book(
        &mut self,
        element: &parser::Spanned<Element>,
        date: Date,
        postings: &[&'a parser::Spanned<parser::Posting<'a>>],
        description: &'a str,
    ) -> Result<BookedPostingsAndPrices<'a>, parser::AnnotatedError>
    where
        T: limabean_booking::Tolerance<Types = limabean_booking::LimaParserBookingTypes<'a>>,
    {
        match limabean_booking::book(
            date,
            postings,
            &self.tolerance,
            |accname| self.accounts.get(accname).map(|acc| &acc.positions),
            |accname| {
                self.accounts
                    .get(accname)
                    .map(|acc| acc.booking)
                    .unwrap_or(self.default_booking)
            },
        ) {
            Ok(Bookings {
                interpolated_postings,
                updated_inventory,
            }) => {
                let mut prices: HashSet<(parser::Currency, parser::Currency, Decimal)> =
                    HashSet::default();

                // check all postings have valid accounts and currencies
                // returning the first error
                if let Some(error) = interpolated_postings
                    .iter()
                    .zip(postings)
                    .filter_map(|(interpolated, posting)| {
                        self.validate_account_and_currency(
                            &into_spanned_element(posting),
                            posting.account().item().as_ref(),
                            interpolated.currency,
                        )
                        .map_or_else(Some, |_| None)
                    })
                    .next()
                {
                    return Err(error);
                }

                // an interpolated posting arising from a reduction with multiple costs is mapped here to several postings,
                // each with a simple cost, so we don't have to deal with composite costs for a posting elsewhere
                let booked_postings = interpolated_postings
                    .into_iter()
                    .zip(postings)
                    .flat_map(|(interpolated, posting)| {
                        let account = posting.account().item().as_ref();
                        let flag = posting.flag().map(|flag| *flag.item());
                        let Interpolated {
                            units,
                            currency,
                            cost,
                            price,
                            ..
                        } = interpolated;
                        if let Some(costs) = cost {
                            costs
                                .into_currency_costs()
                                .map(|(cost_cur, cost)| {
                                    prices.insert((currency, cost_cur, cost.per_unit));

                                    Posting {
                                        flag,
                                        account,
                                        units: cost.units,
                                        currency,
                                        cost: Some(cur_posting_cost_to_cost(cost_cur, cost)),
                                        price: None,
                                    }
                                })
                                .collect::<Vec<_>>()
                        } else {
                            if let Some(price) = &price {
                                prices.insert((currency, price.currency, price.per_unit));
                            }

                            vec![Posting {
                                flag,
                                account,
                                units,
                                currency,
                                cost: None,
                                price,
                            }]
                        }
                    })
                    .collect::<Vec<_>>();

                // group postings by account and currency for balance diagnostics
                let mut account_posting_amounts =
                    hashbrown::HashMap::<&str, VecDeque<Amount<'_>>>::new();
                for booked in &booked_postings {
                    use hashbrown::hash_map::Entry::*;

                    let currency = booked.currency;
                    let units = booked.units;

                    self.tally_currency_usage(currency);

                    let account_name = booked.account;

                    match account_posting_amounts.entry(account_name) {
                        Occupied(entry) => {
                            entry.into_mut().push_back((units, currency).into());
                        }
                        Vacant(entry) => {
                            let mut amounts = VecDeque::new();
                            amounts.push_back((units, currency).into());
                            entry.insert(amounts);
                        }
                    }
                }

                for (account_name, updated_positions) in updated_inventory {
                    let account = self.get_mut_valid_account(element, account_name)?;

                    account.positions = updated_positions;

                    if let Some(mut posting_amounts) = account_posting_amounts.remove(account_name)
                    {
                        let last_amount = posting_amounts.pop_back().unwrap();

                        for amount in posting_amounts {
                            account.balance_diagnostics.push(BalanceDiagnostic {
                                date,
                                description: Some(description),
                                amount: Some(amount),
                                positions: None,
                            });
                        }

                        account.balance_diagnostics.push(BalanceDiagnostic {
                            date,
                            description: Some(description),
                            amount: Some(last_amount),
                            positions: Some(account.positions.clone()),
                        });
                    }
                }

                Ok(BookedPostingsAndPrices {
                    postings: booked_postings,
                    prices,
                })
            }
            Err(e) => {
                use limabean_booking::BookingError::*;

                match &e {
                    Transaction(e) => Err(element.error(e.to_string()).into()),
                    Posting(idx, e) => {
                        // TODO attach posting error to actual posting
                        // let bad_posting = postings[*idx];
                        // bad_posting.error(e.to_string()).into()
                        Err(element.error(format!("{e} on posting {idx}")).into())
                    }
                }
            }
        }
    }

    fn validate_account(
        &self,
        element: &parser::Spanned<Element>,
        account_name: &'a str,
    ) -> Result<(), parser::AnnotatedError> {
        if self.open_accounts.contains_key(account_name) {
            Ok(())
        } else if let Some(closed) = self.closed_accounts.get(account_name) {
            Err(element
                .error_with_contexts("account was closed", vec![("close".to_string(), *closed)])
                .into())
        } else {
            Err(element.error("account not open").into())
        }
    }

    fn get_valid_account(
        &self,
        element: &parser::Spanned<Element>,
        account_name: &'a str,
    ) -> Result<&AccountBuilder<'a>, parser::AnnotatedError> {
        self.validate_account(element, account_name)?;
        Ok(self.accounts.get(account_name).unwrap())
    }

    fn get_mut_valid_account(
        &mut self,
        element: &parser::Spanned<Element>,
        account_name: &'a str,
    ) -> Result<&mut AccountBuilder<'a>, parser::AnnotatedError> {
        self.validate_account(element, account_name)?;
        Ok(self.accounts.get_mut(account_name).unwrap())
    }

    fn validate_account_and_currency(
        &self,
        element: &parser::Spanned<Element>,
        account_name: &'a str,
        currency: parser::Currency<'a>,
    ) -> Result<(), parser::AnnotatedError> {
        let account = self.get_valid_account(element, account_name)?;
        account.validate_currency(element, currency)
    }

    fn tally_currency_usage(&mut self, currency: parser::Currency<'a>) {
        use hashbrown::hash_map::Entry::*;

        match self.currency_usage.entry(currency) {
            Occupied(mut usage) => {
                let usage = usage.get_mut();
                *usage += 1;
            }
            Vacant(usage) => {
                usage.insert(1);
            }
        }
    }

    // base account is known
    fn rollup_units(
        &self,
        base_account_name: &str,
    ) -> hashbrown::HashMap<parser::Currency<'a>, Decimal> {
        if self.internal_plugins.balance_rollup {
            let mut rollup_units = hashbrown::HashMap::<parser::Currency<'a>, Decimal>::default();
            self.accounts
                .keys()
                .filter_map(|s| {
                    s.starts_with(base_account_name)
                        .then_some(self.accounts.get(s).unwrap().positions.units())
                })
                .for_each(|account| {
                    account.into_iter().for_each(|(cur, number)| {
                        use hashbrown::hash_map::Entry::*;
                        match rollup_units.entry(*cur) {
                            Occupied(mut entry) => {
                                let existing_number = entry.get_mut();
                                *existing_number += number;
                            }
                            Vacant(entry) => {
                                entry.insert(number);
                            }
                        }
                    });
                });
            rollup_units
        } else {
            self.accounts
                .get(base_account_name)
                .map(|account| {
                    account
                        .positions
                        .units()
                        .iter()
                        .map(|(cur, number)| (**cur, *number))
                        .collect::<hashbrown::HashMap<_, _>>()
                })
                .unwrap_or_default()
        }
    }

    fn balance(
        &mut self,
        balance: &'a parser::Balance,
        date: Date,
        element: parser::Spanned<Element>,
    ) -> Result<DirectiveVariant<'a>, parser::AnnotatedError>
    where
        T: limabean_booking::Tolerance<Types = limabean_booking::LimaParserBookingTypes<'a>>,
    {
        let account_name = balance.account().item().as_ref();
        let balance_currency = *balance.atol().amount().currency().item();
        let balance_units = balance.atol().amount().number().value();
        let balance_tolerance = balance
            .atol()
            .tolerance()
            .map(|x| *x.item())
            .unwrap_or(Decimal::ZERO);
        let margin = calculate_balance_margin(
            balance_units,
            balance_currency,
            balance_tolerance,
            self.rollup_units(account_name),
        );

        let account = self.get_mut_valid_account(&element, account_name)?;
        account.validate_currency(&element, balance_currency)?;
        // pad can't last beyond balance
        let pad_idx = account.pad_idx.take();

        if margin.is_empty() {
            // balance assertion is correct, and we already cleared the pad, so:

            account.balance_diagnostics.clear();
            return Ok(DirectiveVariant::NA);
        }

        if pad_idx.is_none() {
            // balance assertion is incorrect and we have no pad to take up the slack, so:

            let err = Err(construct_balance_error_and_clear_diagnostics(
                account, &margin, &element,
            ));

            // even though we have a balance error, we adjust the account to match, in order to localise balance failures
            adjust_account_to_match_balance(account, &margin, Adjustment::Add);

            return err;
        }
        let pad_idx = pad_idx.unwrap();

        adjust_account_to_match_balance(account, &margin, Adjustment::Add);
        account.balance_diagnostics.clear();

        // initialize balance diagnostics according to balance assertion
        let mut positions = Positions::default();
        positions.accumulate(balance_units, balance_currency, None, Booking::default());
        account.balance_diagnostics.push(BalanceDiagnostic {
            date,
            description: None,
            amount: None,
            positions: Some(positions),
        });

        let pad_directive = &mut self.directives[pad_idx];
        let parser::DirectiveVariant::Pad(pad) = pad_directive.parsed.variant() else {
            panic!(
                "directive at pad_idx {pad_directive} is not a pad, is {:?}",
                pad_directive
            );
        };

        let pad_source = pad.source().item().as_ref();

        let pad_postings =
            calculate_balance_pad_postings(&margin, balance.account().item().as_ref(), pad_source);

        if let DirectiveVariant::Pad(pad) = &mut pad_directive.loaded {
            pad.postings = pad_postings;
        }

        let pad_account = self.accounts.get_mut(pad_source).unwrap();
        adjust_account_to_match_balance(pad_account, &margin, Adjustment::Subtract);

        Ok(DirectiveVariant::NA)
    }

    fn open(
        &mut self,
        open: &'a parser::Open,
        _date: Date,
        element: parser::Spanned<Element>,
    ) -> Result<DirectiveVariant<'a>, parser::AnnotatedError> {
        use hashbrown::hash_map::Entry::*;
        match self.open_accounts.entry(open.account().item().as_ref()) {
            Occupied(open_entry) => {
                return Err(element
                    .error_with_contexts(
                        "account already opened",
                        vec![("open".to_string(), *open_entry.get())],
                    )
                    .into());
            }
            Vacant(open_entry) => {
                let span = element.span();
                open_entry.insert(*span);

                // cannot reopen a closed account
                if let Some(closed) = self.closed_accounts.get(&open.account().item().as_ref()) {
                    return Err(element
                        .error_with_contexts(
                            "account was closed",
                            vec![("close".to_string(), *closed)],
                        )
                        .into());
                } else {
                    let mut booking = open
                        .booking()
                        .map(|booking| Into::<Booking>::into(*booking.item()))
                        .unwrap_or(self.default_booking);

                    if !is_supported_method(booking) {
                        let default_booking = Booking::default();
                        self.warnings.push(
                            element .warning(format!( "booking method {booking} unsupported, falling back to default {default_booking}" )) .into(),
                        );
                        booking = default_booking;
                    }

                    self.accounts.insert(
                        open.account().item().as_ref(),
                        AccountBuilder::new(open.currencies().map(|c| *c.item()), booking, *span),
                    );
                }
            }
        }

        if let Some(booking) = open.booking() {
            let booking = Into::<Booking>::into(*booking.item());
            if is_supported_method(booking) {
            } else {
                self.warnings.push(
                    element
                        .warning("booking method {} unsupported, falling back to default")
                        .into(),
                );
            }
        }

        Ok(DirectiveVariant::NA)
    }

    fn close(
        &mut self,
        close: &'a parser::Close,
        _date: Date,
        element: parser::Spanned<Element>,
    ) -> Result<DirectiveVariant<'a>, parser::AnnotatedError> {
        use hashbrown::hash_map::Entry::*;
        match self.open_accounts.entry(close.account().item().as_ref()) {
            Occupied(open_entry) => {
                match self.closed_accounts.entry(close.account().item().as_ref()) {
                    Occupied(closed_entry) => {
                        // cannot reclose a closed account
                        return Err(element
                            .error_with_contexts(
                                "account was already closed",
                                vec![("close".to_string(), *closed_entry.get())],
                            )
                            .into());
                    }
                    Vacant(closed_entry) => {
                        open_entry.remove_entry();
                        closed_entry.insert(*element.span());
                    }
                }
            }
            Vacant(_) => {
                return Err(element.error("account not open").into());
            }
        }

        Ok(DirectiveVariant::NA)
    }

    fn pad(
        &mut self,
        pad: &'a parser::Pad<'a>,
        _date: Date,
        element: parser::Spanned<Element>,
    ) -> Result<DirectiveVariant<'a>, parser::AnnotatedError> {
        let n_directives = self.directives.len();
        let account_name = pad.account().item().as_ref();
        let account = self.get_mut_valid_account(&element, account_name)?;

        let unused_pad_idx = account.pad_idx.replace(n_directives);

        // unused pad directives are errors
        // https://beancount.github.io/docs/beancount_language_syntax.html#unused-pad-directives
        if let Some(unused_pad_idx) = unused_pad_idx {
            return Err(self.directives[unused_pad_idx]
                .parsed
                .error("unused")
                .into());
        }

        Ok(DirectiveVariant::Pad(Pad {
            postings: Vec::default(),
        }))
    }
}

struct BookedPostingsAndPrices<'a> {
    postings: Vec<Posting<'a>>,
    prices: HashSet<(parser::Currency<'a>, parser::Currency<'a>, Decimal)>,
}

fn calculate_balance_margin<'a>(
    balance_units: Decimal,
    balance_currency: parser::Currency<'a>,
    balance_tolerance: Decimal,
    account_rollup: hashbrown::HashMap<parser::Currency<'a>, Decimal>,
) -> HashMap<parser::Currency<'a>, Decimal> {
    // what's the gap between what we have and what the balance says we should have?
    let mut inventory_has_balance_currency = false;
    let mut margin = account_rollup
        .into_iter()
        .map(|(cur, number)| {
            if balance_currency == cur {
                inventory_has_balance_currency = true;
                (cur, balance_units - Into::<Decimal>::into(number))
            } else {
                (cur, -(Into::<Decimal>::into(number)))
            }
        })
        .filter_map(|(cur, number)| {
            // discard anything below the tolerance
            (number.abs() > balance_tolerance).then_some((cur, number))
        })
        .collect::<HashMap<_, _>>();

    // cope with the case of balance currency wasn't in inventory
    if !inventory_has_balance_currency && (balance_units.abs() > balance_tolerance) {
        margin.insert(balance_currency, balance_units);
    }

    margin
}

fn calculate_balance_pad_postings<'a>(
    margin: &HashMap<parser::Currency<'a>, Decimal>,
    balance_account: &'a str,
    pad_source: &'a str,
) -> Vec<Posting<'a>> {
    margin
        .iter()
        .flat_map(|(cur, number)| {
            vec![
                Posting {
                    flag: Some(pad_flag()),
                    account: balance_account,
                    units: *number,
                    currency: *cur,
                    cost: None,
                    price: None,
                },
                Posting {
                    flag: Some(pad_flag()),
                    account: pad_source,
                    units: -*number,
                    currency: *cur,
                    cost: None,
                    price: None,
                },
            ]
        })
        .collect::<Vec<_>>()
}

fn construct_balance_error_and_clear_diagnostics<'a>(
    account: &mut AccountBuilder<'a>,
    margin: &HashMap<parser::Currency<'a>, Decimal>,
    element: &parser::Spanned<Element>,
) -> parser::AnnotatedError {
    let reason = format!(
        "accumulated {}, error {}",
        if account.positions.is_empty() {
            "zero".to_string()
        } else {
            account.positions.to_string()
        },
        margin
            .iter()
            .map(|(cur, number)| format!("{number} {cur}"))
            .collect::<Vec<String>>()
            .join(", ")
    );

    // determine context for error by collating postings since last balance
    let annotation = Cell::Stack(
        account
            .balance_diagnostics
            .drain(..)
            .map(|bd| {
                Cell::Row(
                    vec![
                        (bd.date.to_string(), Align::Left).into(),
                        bd.amount.map(|amt| amt.into()).unwrap_or(Cell::Empty),
                        bd.positions.map(positions_into_cell).unwrap_or(Cell::Empty),
                        bd.description
                            .map(|d| (d, Align::Left).into())
                            .unwrap_or(Cell::Empty),
                    ],
                    GUTTER_MEDIUM,
                )
            })
            .collect::<Vec<_>>(),
    );

    element
        .error(reason)
        .with_annotation(annotation.to_string())
}

#[derive(PartialEq, Eq, Debug)]
enum Adjustment {
    Add,
    Subtract,
}

fn adjust_account_to_match_balance<'a>(
    account: &mut AccountBuilder<'a>,
    margin: &HashMap<parser::Currency<'a>, Decimal>,
    adjustment: Adjustment,
) {
    use Adjustment::*;

    // reset accumulated balance to what was asserted, to localise errors
    for (cur, units) in margin.iter() {
        account.positions.accumulate(
            if adjustment == Add { *units } else { -*units },
            *cur,
            None,
            Booking::default(),
        );
        // booking method doesn't matter if no cost
    }
}

#[derive(Debug)]
struct AccountBuilder<'a> {
    allowed_currencies: HashSet<parser::Currency<'a>>,
    positions: Positions<'a>,
    opened: Span,
    pad_idx: Option<usize>, // index in directives in Loader
    balance_diagnostics: Vec<BalanceDiagnostic<'a>>,
    booking: Booking,
}

impl<'a> AccountBuilder<'a> {
    fn new<I>(allowed_currencies: I, booking: Booking, opened: Span) -> Self
    where
        I: Iterator<Item = parser::Currency<'a>>,
    {
        AccountBuilder {
            allowed_currencies: allowed_currencies.collect(),
            positions: Positions::default(),
            opened,
            pad_idx: None,
            balance_diagnostics: Vec::default(),
            booking,
        }
    }

    /// all currencies are valid unless any were specified during open
    fn is_currency_valid(&self, currency: parser::Currency<'_>) -> bool {
        self.allowed_currencies.is_empty() || self.allowed_currencies.contains(&currency)
    }

    fn validate_currency(
        &self,
        element: &parser::Spanned<Element>,
        currency: parser::Currency<'_>,
    ) -> Result<(), parser::AnnotatedError> {
        if self.is_currency_valid(currency) {
            Ok(())
        } else {
            Err(element
                .error_with_contexts(
                    "invalid currency for account",
                    vec![("open".to_string(), self.opened)],
                )
                .into())
        }
    }
}

#[derive(Debug)]
struct BalanceDiagnostic<'a> {
    date: Date,
    description: Option<&'a str>,
    amount: Option<Amount<'a>>,
    positions: Option<Positions<'a>>,
}

pub(crate) fn pad_flag() -> parser::Flag {
    parser::Flag::Letter(TryInto::<parser::FlagLetter>::try_into('P').unwrap())
}

pub(crate) mod types;
pub(crate) use types::*;
