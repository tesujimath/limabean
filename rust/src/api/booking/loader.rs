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

use crate::api::types::{
    booked,
    parser_type_conversions::{from_flag, from_key_values, from_links, from_tags},
    raw,
};

#[derive(Debug)]
pub(crate) struct Loader<'a, T> {
    directives: Vec<booked::Directive<'a>>,
    // hashbrown HashMaps are used here for their Entry API, which is still unstable in std::collections::HashMap
    open_accounts: hashbrown::HashMap<&'a str, Span>,
    closed_accounts: hashbrown::HashMap<&'a str, Span>,
    accounts: HashMap<&'a str, AccountBuilder<'a>>,
    currency_usage: hashbrown::HashMap<&'a str, i32>,
    // TODO internal_plugins
    // internal_plugins: &'b hashbrown::HashMap<InternalPlugin, Option<String>>,
    default_booking: Booking,
    tolerance: T,
    warnings: Vec<parser::AnnotatedWarning>,
}

pub(crate) struct LoadSuccess<'a> {
    pub(crate) directives: Vec<booked::Directive<'a>>,
    pub(crate) warnings: Vec<parser::AnnotatedWarning>,
}

pub(crate) struct LoadError {
    pub(crate) errors: Vec<parser::AnnotatedError>,
}

impl<'a, T> Loader<'a, T> {
    pub(crate) fn new(
        default_booking: Booking,
        tolerance: T,
        // internal_plugins: &'b hashbrown::HashMap<InternalPlugin, Option<String>>,
    ) -> Self {
        Self {
            directives: Vec::default(),
            open_accounts: hashbrown::HashMap::default(),
            closed_accounts: hashbrown::HashMap::default(),
            accounts: HashMap::default(),
            currency_usage: hashbrown::HashMap::default(),
            // internal_plugins,
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

        // TODO check for unused pad directives
        // for account in accounts.values() {
        //     if let Some(pad_idx) = &account.pad_idx {
        //         errors.push(directives[*pad_idx].parsed.error("unused").into())
        //     }
        // }

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
        // TODO these should be raw directives not parser directives
        I: IntoIterator<Item = &'a Spanned<parser::Directive<'a>>>,
        T: limabean_booking::Tolerance<Types = limabean_booking::LimaParserBookingTypes<'a>> + Copy,
    {
        let mut errors = Vec::default();

        for raw in directives {
            match self.directive(raw) {
                Ok(booked_variant) => {
                    self.directives.push(booked::Directive {
                        span: raw.into(),
                        date: *raw.date().item(),
                        tags: from_tags(raw.metadata().tags()),
                        links: from_links(raw.metadata().links()),
                        metadata: from_key_values(raw.metadata().key_values()),
                        variant: booked_variant,
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
    ) -> Result<booked::DirectiveVariant<'a>, parser::AnnotatedError>
    where
        T: limabean_booking::Tolerance<Types = limabean_booking::LimaParserBookingTypes<'a>> + Copy,
    {
        use parser::DirectiveVariant as PDV;

        let date = *directive.date().item();

        match directive.variant() {
            PDV::Transaction(transaction) => {
                self.transaction(&into_spanned_loader_element(directive), transaction, date)
            }
            _ => todo!("all other directives"),
            // PDV::Price(_price) => Ok(DirectiveVariant::NA),
            // PDV::Balance(balance) => self.balance(balance, date, element),
            // PDV::Open(open) => self.open(open, date, element),
            // PDV::Close(close) => self.close(close, date, element),
            // PDV::Commodity(_commodity) => Ok(DirectiveVariant::NA),
            // PDV::Pad(pad) => self.pad(pad, date, element),
            // PDV::Document(_document) => Ok(DirectiveVariant::NA),
            // PDV::Note(_note) => Ok(DirectiveVariant::NA),
            // PDV::Event(_event) => Ok(DirectiveVariant::NA),
            // PDV::Query(_query) => Ok(DirectiveVariant::NA),
            // PDV::Custom(_custom) => Ok(DirectiveVariant::NA),
        }
    }

    fn transaction(
        &mut self,
        element: &parser::Spanned<LoaderElement>,
        transaction: &'a parser::Transaction<'a>,
        date: Date,
    ) -> Result<booked::DirectiveVariant<'a>, parser::AnnotatedError>
    where
        T: limabean_booking::Tolerance<Types = limabean_booking::LimaParserBookingTypes<'a>> + Copy,
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

        // TODO auto accounts
        // let auto_accounts = if self
        //     .internal_plugins
        //     .contains_key(&InternalPlugin::AutoAccounts)
        // {
        //     let mut auto_accounts = HashSet::default();

        //     for account in postings.iter().map(|posting| posting.account()) {
        //         let account_name = account.item().as_ref();
        //         if !self.accounts.contains_key(account_name) {
        //             auto_accounts.insert(account_name);

        //             self.accounts.insert(
        //                 account_name,
        //                 AccountBuilder::new(empty(), self.default_booking, *account.span()),
        //             );
        //             self.open_accounts.insert(account_name, *account.span());
        //         }
        //     }
        //     auto_accounts
        // } else {
        //     HashSet::default()
        // };

        let BookedPostingsAndPrices { postings, prices } =
            self.book(element, date, &postings, description)?;

        Ok(booked::DirectiveVariant::Transaction(booked::Transaction {
            flag: from_flag(*transaction.flag().item()),
            payee: transaction.payee().map(|payee| payee.item().as_ref()),
            narration: transaction
                .narration()
                .map(|narration| narration.item().as_ref()),
            postings,
            // TODO implicit prices
            // prices,
            // TODO auto accounts
            // auto_accounts,
        }))
    }

    fn book(
        &mut self,
        element: &parser::Spanned<LoaderElement>,
        date: Date,
        postings: &[&'a parser::Spanned<parser::Posting<'a>>],
        description: &'a str,
    ) -> Result<BookedPostingsAndPrices<'a>, parser::AnnotatedError>
    where
        T: limabean_booking::Tolerance<Types = limabean_booking::LimaParserBookingTypes<'a>> + Copy,
    {
        match limabean_booking::book(
            date,
            postings,
            self.tolerance,
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
                let mut prices: HashSet<(&str, booked::Price)> = HashSet::default();

                // check all postings have valid accounts and currencies
                // returning the first error
                if let Some(error) = interpolated_postings
                    .iter()
                    .zip(postings)
                    .filter_map(|(interpolated, posting)| {
                        self.validate_account_and_currency(
                            &into_spanned_loader_element(posting),
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

                        let posting_span: raw::Span = (*posting).into();

                        if let Some(costs) = cost {
                            costs
                                .into_currency_costs()
                                .map(|(cost_cur, cost)| {
                                    prices.insert((
                                        currency.as_ref(),
                                        booked::Price {
                                            cur: cost_cur.as_ref(),
                                            per_unit: cost.per_unit,
                                            total: None,
                                        },
                                    ));

                                    booked::Posting {
                                        span: posting_span,
                                        flag: posting.flag().map(|flag| from_flag(*flag.item())),
                                        acc: account,
                                        units: cost.units,
                                        cur: currency.as_ref(),
                                        cost: Some((&cost_cur, &cost).into()),
                                        price: None,
                                        tags: from_tags(posting.metadata().tags()),
                                        links: from_links(posting.metadata().links()),
                                        metadata: from_key_values(posting.metadata().key_values()),
                                    }
                                })
                                .collect::<Vec<_>>()
                        } else {
                            if let Some(price) = &price {
                                prices.insert((
                                    currency.as_ref(),
                                    booked::Price {
                                        cur: price.currency.as_ref(),
                                        per_unit: price.per_unit,
                                        total: None,
                                    },
                                ));
                            }

                            vec![booked::Posting {
                                span: posting_span,
                                flag: posting.flag().map(|flag| from_flag(*flag.item())),
                                acc: account,
                                units,
                                cur: currency.as_ref(),
                                cost: None,
                                price: price.map(|price| (&price).into()),
                                tags: from_tags(posting.metadata().tags()),
                                links: from_links(posting.metadata().links()),
                                metadata: from_key_values(posting.metadata().key_values()),
                            }]
                        }
                    })
                    .collect::<Vec<_>>();

                // group postings by account and currency for balance diagnostics
                let mut account_posting_amounts =
                    hashbrown::HashMap::<&str, VecDeque<LoaderAmount<'_>>>::new();
                for booked in &booked_postings {
                    use hashbrown::hash_map::Entry::*;

                    let currency = booked.cur;
                    let units = booked.units;

                    self.tally_currency_usage(currency);

                    let account_name = booked.acc;

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
        element: &parser::Spanned<LoaderElement>,
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
        element: &parser::Spanned<LoaderElement>,
        account_name: &'a str,
    ) -> Result<&AccountBuilder<'a>, parser::AnnotatedError> {
        self.validate_account(element, account_name)?;
        Ok(self.accounts.get(account_name).unwrap())
    }

    fn get_mut_valid_account(
        &mut self,
        element: &parser::Spanned<LoaderElement>,
        account_name: &'a str,
    ) -> Result<&mut AccountBuilder<'a>, parser::AnnotatedError> {
        self.validate_account(element, account_name)?;
        Ok(self.accounts.get_mut(account_name).unwrap())
    }

    fn validate_account_and_currency(
        &self,
        element: &parser::Spanned<LoaderElement>,
        account_name: &'a str,
        currency: parser::Currency<'a>,
    ) -> Result<(), parser::AnnotatedError> {
        let account = self.get_valid_account(element, account_name)?;
        account.validate_currency(element, currency)
    }

    fn tally_currency_usage(&mut self, currency: &'a str) {
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
        // TODO internal plugin balance rollup
        // if self
        //     .internal_plugins
        //     .contains_key(&InternalPlugin::BalanceRollup)
        // {
        //     let mut rollup_units = hashbrown::HashMap::<parser::Currency<'a>, Decimal>::default();
        //     self.accounts
        //         .keys()
        //         .filter_map(|s| {
        //             s.starts_with(base_account_name)
        //                 .then_some(self.accounts.get(s).unwrap().positions.units())
        //         })
        //         .for_each(|account| {
        //             account.into_iter().for_each(|(cur, number)| {
        //                 use hashbrown::hash_map::Entry::*;
        //                 match rollup_units.entry(*cur) {
        //                     Occupied(mut entry) => {
        //                         let existing_number = entry.get_mut();
        //                         *existing_number += number;
        //                     }
        //                     Vacant(entry) => {
        //                         entry.insert(number);
        //                     }
        //                 }
        //             });
        //         });
        //     rollup_units
        // } else {
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
        // }
    }

    fn balance(
        &mut self,
        balance: &'a parser::Balance,
        date: Date,
        element: parser::Spanned<LoaderElement>,
    ) -> Result<booked::DirectiveVariant<'a>, parser::AnnotatedError>
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
            return Ok(booked::DirectiveVariant::Balance(balance.into()));
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
        let mut positions = LoaderPositions::default();
        positions.accumulate(balance_units, balance_currency, None, Booking::default());
        account.balance_diagnostics.push(BalanceDiagnostic {
            date,
            description: None,
            amount: None,
            positions: Some(positions),
        });

        // TODO pad postings
        // let pad_directive = &mut self.directives[pad_idx];
        // let parser::DirectiveVariant::Pad(pad) = pad_directive.parsed.variant() else {
        //     panic!(
        //         "directive at pad_idx {pad_directive} is not a pad, is {:?}",
        //         pad_directive
        //     );
        // };

        // let pad_source = pad.source().item().as_ref();

        // let pad_postings =
        //     calculate_balance_pad_postings(&margin, balance.account().item().as_ref(), pad_source);

        // if let DirectiveVariant::Pad(pad) = &mut pad_directive.loaded {
        //     pad.postings = pad_postings;
        // }

        // let pad_account = self.accounts.get_mut(pad_source).unwrap();
        // adjust_account_to_match_balance(pad_account, &margin, Adjustment::Subtract);

        Ok(booked::DirectiveVariant::Balance(balance.into()))
    }

    fn open(
        &mut self,
        open: &'a parser::Open,
        _date: Date,
        element: parser::Spanned<LoaderElement>,
    ) -> Result<booked::DirectiveVariant<'a>, parser::AnnotatedError> {
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

        Ok(booked::DirectiveVariant::Open(open.into()))
    }

    fn close(
        &mut self,
        close: &'a parser::Close,
        _date: Date,
        element: parser::Spanned<LoaderElement>,
    ) -> Result<booked::DirectiveVariant<'a>, parser::AnnotatedError> {
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

        Ok(booked::DirectiveVariant::Close(close.into()))
    }

    fn pad(
        &mut self,
        pad: &'a parser::Pad<'a>,
        _date: Date,
        element: parser::Spanned<LoaderElement>,
    ) -> Result<booked::DirectiveVariant<'a>, parser::AnnotatedError> {
        let n_directives = self.directives.len();
        let account_name = pad.account().item().as_ref();
        let account = self.get_mut_valid_account(&element, account_name)?;

        let unused_pad_idx = account.pad_idx.replace(n_directives);

        // TODO unused pad directives are errors
        // https://beancount.github.io/docs/beancount_language_syntax.html#unused-pad-directives
        // if let Some(unused_pad_idx) = unused_pad_idx {
        //     return Err(self.directives[unused_pad_idx]
        //         .parsed
        //         .error("unused")
        //         .into());
        // }

        // TODO pad postings
        Ok(booked::DirectiveVariant::Pad(
            pad.into(), //     Pad {
                        //     postings: Vec::default(),
                        // }
        ))
    }
}

struct BookedPostingsAndPrices<'a> {
    postings: Vec<booked::Posting<'a>>,
    prices: HashSet<(&'a str, booked::Price<'a>)>,
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

// TODO calculate_balance_pad_postings
// fn calculate_balance_pad_postings<'a>(
//     margin: &HashMap<parser::Currency<'a>, Decimal>,
//     balance_account: &'a str,
//     pad_source: &'a str,
// ) -> Vec<Posting<'a>> {
//     margin
//         .iter()
//         .flat_map(|(cur, number)| {
//             vec![
//                 Posting {
//                     flag: Some(pad_flag()),
//                     account: balance_account,
//                     units: *number,
//                     currency: *cur,
//                     cost: None,
//                     price: None,
//                 },
//                 Posting {
//                     flag: Some(pad_flag()),
//                     account: pad_source,
//                     units: -*number,
//                     currency: *cur,
//                     cost: None,
//                     price: None,
//                 },
//             ]
//         })
//         .collect::<Vec<_>>()
// }

fn construct_balance_error_and_clear_diagnostics<'a>(
    account: &mut AccountBuilder<'a>,
    margin: &HashMap<parser::Currency<'a>, Decimal>,
    element: &parser::Spanned<LoaderElement>,
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
                        bd.positions
                            .map(loader_positions_into_cell)
                            .unwrap_or(Cell::Empty),
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
    positions: LoaderPositions<'a>,
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
            positions: LoaderPositions::default(),
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
        element: &parser::Spanned<LoaderElement>,
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
    amount: Option<LoaderAmount<'a>>,
    positions: Option<LoaderPositions<'a>>,
}

pub(crate) fn pad_flag() -> parser::Flag {
    parser::Flag::Letter(TryInto::<parser::FlagLetter>::try_into('P').unwrap())
}

// TODO remove the other Element ans rename this one
#[derive(Clone, Debug)]
pub(crate) struct LoaderElement {
    element_type: &'static str,
}

impl parser::ElementType for LoaderElement {
    fn element_type(&self) -> &'static str {
        self.element_type
    }
}

pub(crate) fn into_spanned_loader_element<T>(
    value: &parser::Spanned<T>,
) -> parser::Spanned<LoaderElement>
where
    T: parser::ElementType,
{
    parser::spanned(
        LoaderElement {
            element_type: value.element_type(),
        },
        *value.span(),
    )
}

// TODO find a better home for LoaderAmount and change its name back when Amount is deleted from book
#[derive(PartialEq, Eq, Clone, Debug)]
pub(crate) struct LoaderAmount<'a> {
    pub(crate) number: Decimal,
    pub(crate) currency: &'a str,
}

impl<'a> From<(Decimal, &'a str)> for LoaderAmount<'a> {
    fn from(value: (Decimal, &'a str)) -> Self {
        Self {
            number: value.0,
            currency: value.1,
        }
    }
}

impl<'a> From<&'a parser::Amount<'a>> for LoaderAmount<'a> {
    fn from(value: &'a parser::Amount<'a>) -> Self {
        LoaderAmount {
            number: value.number().value(),
            currency: value.currency().item().as_ref(),
        }
    }
}

impl<'a> From<LoaderAmount<'a>> for Cell<'static, 'static> {
    fn from(value: LoaderAmount) -> Self {
        Cell::Row(
            vec![
                value.number.into(),
                (value.currency.to_string(), Align::Left).into(),
            ],
            GUTTER_MINOR,
        )
    }
}

impl<'a, 'b> From<&'b LoaderAmount<'a>> for Cell<'a, 'static>
where
    'a: 'b,
{
    fn from(value: &'b LoaderAmount<'a>) -> Self {
        Cell::Row(
            vec![value.number.into(), (value.currency, Align::Left).into()],
            GUTTER_MINOR,
        )
    }
}

// TODO rename once Positions is deleted from book types
type LoaderPositions<'a> =
    limabean_booking::Positions<limabean_booking::LimaParserBookingTypes<'a>>;
// should be From, but both types are third-party
fn loader_positions_into_cell<'a>(positions: LoaderPositions<'a>) -> Cell<'a, 'static> {
    Cell::Stack(
        positions
            .into_iter()
            .map(loader_position_into_cell)
            .collect::<Vec<_>>(),
    )
}

type LoaderPosition<'a> = limabean_booking::Position<limabean_booking::LimaParserBookingTypes<'a>>;

fn loader_position_into_cell<'a>(position: LoaderPosition<'a>) -> Cell<'a, 'static> {
    let LoaderPosition {
        units,
        currency,
        cost,
    } = position;
    let mut cells = vec![
        units.into(),
        (Into::<&str>::into(currency), Align::Left).into(),
    ];
    if let Some(cost) = cost {
        cells.push(loader_cost_into_cell(cost))
    }
    Cell::Row(cells, GUTTER_MINOR)
}

type LoaderCost<'a> = limabean_booking::Cost<limabean_booking::LimaParserBookingTypes<'a>>;
fn loader_cost_into_cell<'a>(cost: LoaderCost<'a>) -> Cell<'a, 'static> {
    let LoaderCost {
        date,
        per_unit,
        total: _total,
        currency,
        label: _label,
        merge: _merge,
    } = cost;
    let mut cells = vec![
        (date.to_string(), Align::Left).into(),
        per_unit.into(),
        (Into::<&str>::into(currency), Align::Left).into(),
    ];
    if let Some(label) = &cost.label {
        cells.push((*label, Align::Left).into())
    }
    if cost.merge {
        cells.push(("*", Align::Left).into())
    }
    Cell::Row(cells, GUTTER_MINOR)
}

// TODO find where this should go
const GUTTER_MINOR: &str = " ";
const GUTTER_MEDIUM: &str = "  ";
