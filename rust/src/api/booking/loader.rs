use beancount_parser_lima::{self as parser};
use limabean_booking::{
    Booking, Bookings, Interpolated, LimaParserBookingTypes, LimaTolerance, is_supported_method,
};

use rust_decimal::Decimal;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt::Debug,
};
use tabulator::{Align, Cell};
use time::Date;

use crate::api::types::{Element, booked, raw};

#[derive(Debug)]
pub(crate) struct Loader<'a, 'd, 't> {
    // hashbrown HashMaps are used here for their Entry API, which is still unstable in std::collections::HashMap
    open_accounts: hashbrown::HashMap<&'a str, parser::Spanned<Element<'static>>>,
    closed_accounts: hashbrown::HashMap<&'a str, parser::Spanned<Element<'static>>>,
    accounts: HashMap<&'a str, AccountBuilder<'a, 'd>>,
    currency_usage: hashbrown::HashMap<&'a str, i32>,
    // TODO internal_plugins, just as a struct of bool
    // internal_plugins: &'p hashbrown::HashMap<InternalPlugin, Option<String>>,
    default_booking: Booking,
    tolerance: &'t LimaTolerance<'a>,
    warnings: Vec<parser::AnnotatedWarning>,
}

pub(crate) struct LoadSuccess<'a> {
    pub(crate) directives: Vec<booked::Directive<'a>>,
    pub(crate) warnings: Vec<parser::AnnotatedWarning>,
}

pub(crate) struct LoadError {
    pub(crate) errors: Vec<parser::AnnotatedError>,
}

impl<'a, 'd, 't> Loader<'a, 'd, 't> {
    pub(crate) fn new(
        default_booking: Booking,
        tolerance: &'t LimaTolerance<'a>,
        // internal_plugins: &'p hashbrown::HashMap<InternalPlugin, Option<String>>,
    ) -> Self {
        Self {
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
    fn validate<'b>(
        self,
        directives: Vec<booked::Directive<'b>>,
        mut errors: Vec<parser::AnnotatedError>,
    ) -> Result<LoadSuccess<'b>, LoadError>
    where
        'a: 'b,
    {
        let Self {
            accounts, warnings, ..
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

    pub(crate) fn collect<'r, 'b, I>(mut self, directives: I) -> Result<LoadSuccess<'b>, LoadError>
    where
        'a: 'r + 'b + 'd,
        'r: 'b + 'd,
        I: IntoIterator<Item = &'r raw::Directive<'a>>,
    {
        let mut errors = Vec::default();
        let mut booked_directives = Vec::default();

        for raw in directives {
            match self.directive(raw, booked_directives.len()) {
                Ok(booked_variant) => {
                    booked_directives.push(booked::Directive {
                        span: raw.span,
                        date: raw.date,
                        tags: raw.tags.clone(),
                        links: raw.links.clone(),
                        metadata: raw.metadata.clone(),
                        variant: booked_variant,
                    });
                }
                Err(e) => {
                    errors.push(e);
                }
            }
        }

        self.validate(booked_directives, errors)
    }

    fn directive<'r, 'b>(
        &mut self,
        directive: &'r raw::Directive<'a>,
        idx: usize,
    ) -> Result<booked::DirectiveVariant<'b>, parser::AnnotatedError>
    where
        'a: 'r + 'b + 'd,
        'r: 'b + 'd,
    {
        use booked::DirectiveVariant as BDV;
        use raw::DirectiveVariant as RDV;

        let date = directive.date;
        let element = directive.into();

        match &directive.variant {
            RDV::Transaction(transaction) => self.transaction(transaction, date, &element),
            RDV::Price(price) => Ok(BDV::Price(price.clone())),
            RDV::Balance(balance) => self.balance(balance, date, &element),
            RDV::Open(open) => self.open(open, date, &element),
            RDV::Close(close) => self.close(close, date, &element),
            RDV::Commodity(commodity) => Ok(BDV::Commodity(commodity.clone())),
            RDV::Pad(pad) => self.pad(pad, date, idx, &element),
            RDV::Document(document) => Ok(BDV::Document(document.clone())),
            RDV::Note(note) => Ok(BDV::Note(note.clone())),
            RDV::Event(event) => Ok(BDV::Event(event.clone())),
            RDV::Query(query) => Ok(BDV::Query(query.clone())),
            RDV::Custom(custom) => Ok(BDV::Custom(custom.clone())),
        }
    }

    fn transaction<'r, 'b>(
        &mut self,
        transaction: &'r raw::Transaction<'a>,
        date: Date,
        element: &parser::Spanned<Element<'static>>,
    ) -> Result<booked::DirectiveVariant<'b>, parser::AnnotatedError>
    where
        'a: 'r + 'b + 'd,
        'r: 'b + 'd,
    {
        let description = transaction.payee.as_ref().map_or_else(
            || {
                transaction
                    .narration
                    .as_ref()
                    .map_or("post", |narration| narration.as_ref())
            },
            |payee| payee.as_ref(),
        );

        // TODO auto accounts
        // let auto_accounts = if self
        //     .internal_plugins
        //     .contains_key(&InternalPlugin::AutoAccounts)
        // {
        //     let mut auto_accounts = HashSet::default();

        //     for account in postings.iter().map(|posting| posting.account()) {
        //         let account_name = account.item().into();
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
            self.book(date, &transaction.postings, description, element)?;

        Ok(booked::DirectiveVariant::Transaction(booked::Transaction {
            flag: transaction.flag.clone(),
            payee: transaction.payee.as_ref().map(|payee| payee.as_ref()),
            narration: transaction
                .narration
                .as_ref()
                .map(|narration| narration.as_ref()),
            postings,
            // TODO implicit prices
            // prices,
            // TODO auto accounts
            // auto_accounts,
        }))
    }

    fn book<'r, 'b>(
        &mut self,
        date: Date,
        postings: &'r [raw::PostingSpec<'a>],
        description: &'d str,
        element: &parser::Spanned<Element<'static>>,
    ) -> Result<BookedPostingsAndPrices<'a>, parser::AnnotatedError>
    where
        'a: 'r + 'b + 'd,
        'r: 'b + 'd,
    {
        // ugh, difference of reference vs value
        let postings = postings.iter().collect::<Vec<_>>();

        match limabean_booking::book(
            date,
            &postings,
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
                    .zip(&postings)
                    .filter_map(|(interpolated, posting)| {
                        let posting_element = (*posting).into();
                        self.validate_account_and_currency(
                            posting.acc,
                            interpolated.currency,
                            &posting_element,
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
                    .zip(&postings)
                    .flat_map(|(interpolated, posting)| {
                        let account = posting.acc;
                        let flag = posting.flag.clone();
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
                                    prices.insert((
                                        currency,
                                        booked::Price {
                                            cur: cost_cur,
                                            per_unit: cost.per_unit,
                                            total: None,
                                        },
                                    ));

                                    booked::Posting {
                                        span: posting.span,
                                        flag: posting.flag.clone(),
                                        acc: account,
                                        units: cost.units,
                                        cur: currency,
                                        cost: Some(loader_cur_posting_cost_to_cost(cost_cur, cost)),
                                        price: None,
                                        tags: posting.tags.clone(),
                                        links: posting.links.clone(),
                                        metadata: posting.metadata.clone(),
                                    }
                                })
                                .collect::<Vec<_>>()
                        } else {
                            if let Some(price) = &price {
                                prices.insert((
                                    currency,
                                    booked::Price {
                                        cur: price.currency,
                                        per_unit: price.per_unit,
                                        total: None,
                                    },
                                ));
                            }

                            vec![booked::Posting {
                                span: posting.span,
                                flag: posting.flag.clone(),
                                acc: account,
                                units,
                                cur: currency,
                                cost: None,
                                price: price.map(|price| (&price).into()),
                                tags: posting.tags.clone(),
                                links: posting.links.clone(),
                                metadata: posting.metadata.clone(),
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
                    let account = self.get_mut_valid_account(account_name, element)?;

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
        account_name: &'a str,
        element: &parser::Spanned<Element<'static>>,
    ) -> Result<(), parser::AnnotatedError> {
        if self.open_accounts.contains_key(account_name) {
            Ok(())
        } else if let Some(closed) = self.closed_accounts.get(account_name) {
            Err(element
                .error("account was closed")
                .related_to(closed)
                .into())
        } else {
            Err(element.error("account not open").into())
        }
    }

    fn get_valid_account(
        &self,
        account_name: &'a str,
        element: &parser::Spanned<Element<'static>>,
    ) -> Result<&AccountBuilder<'a, 'd>, parser::AnnotatedError> {
        self.validate_account(account_name, element)?;
        Ok(self.accounts.get(account_name).unwrap())
    }

    fn get_mut_valid_account(
        &mut self,
        account_name: &'a str,
        element: &parser::Spanned<Element<'static>>,
    ) -> Result<&mut AccountBuilder<'a, 'd>, parser::AnnotatedError> {
        self.validate_account(account_name, element)?;
        Ok(self.accounts.get_mut(account_name).unwrap())
    }

    fn validate_account_and_currency(
        &self,
        account_name: &'a str,
        currency: &'a str,
        element: &parser::Spanned<Element<'static>>,
    ) -> Result<(), parser::AnnotatedError> {
        let account = self.get_valid_account(account_name, element)?;
        account.validate_currency(currency, element)
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
    fn rollup_units(&self, base_account_name: &str) -> hashbrown::HashMap<&'a str, Decimal> {
        // TODO internal plugin balance rollup
        // if self
        //     .internal_plugins
        //     .contains_key(&InternalPlugin::BalanceRollup)
        // {
        //     let mut rollup_units = hashbrown::HashMap::<&'a str, Decimal>::default();
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

    fn balance<'r, 'b>(
        &mut self,
        balance: &'r raw::Balance<'a>,
        date: Date,
        element: &parser::Spanned<Element<'static>>,
    ) -> Result<booked::DirectiveVariant<'b>, parser::AnnotatedError>
    where
        'a: 'r + 'b,
        'r: 'b,
    {
        let margin = calculate_balance_margin(
            balance.units,
            balance.cur,
            balance.tolerance.unwrap_or(Decimal::ZERO),
            self.rollup_units(balance.acc),
        );

        let account = self.get_mut_valid_account(balance.acc, element)?;
        account.validate_currency(balance.cur, element)?;
        // pad can't last beyond balance
        let pad_idx = account.pad_idx.take();

        if margin.is_empty() {
            // balance assertion is correct, and we already cleared the pad, so:

            account.balance_diagnostics.clear();
            return Ok(booked::DirectiveVariant::Balance(balance.clone()));
        }

        if pad_idx.is_none() {
            // balance assertion is incorrect and we have no pad to take up the slack, so:

            let err = Err(construct_balance_error_and_clear_diagnostics(
                account, &margin, element,
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
        positions.accumulate(balance.units, balance.cur, None, Booking::default());
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

        // let pad_source = pad.source().item().into();

        // let pad_postings =
        //     calculate_balance_pad_postings(&margin, balance.account().item().into(), pad_source);

        // if let DirectiveVariant::Pad(pad) = &mut pad_directive.loaded {
        //     pad.postings = pad_postings;
        // }

        // let pad_account = self.accounts.get_mut(pad_source).unwrap();
        // adjust_account_to_match_balance(pad_account, &margin, Adjustment::Subtract);

        Ok(booked::DirectiveVariant::Balance(balance.clone()))
    }

    fn open<'r, 'b>(
        &mut self,
        open: &'r raw::Open<'a>,
        _date: Date,
        element: &parser::Spanned<Element<'static>>,
    ) -> Result<booked::DirectiveVariant<'b>, parser::AnnotatedError>
    where
        'a: 'r + 'b,
        'r: 'b,
    {
        use hashbrown::hash_map::Entry::*;
        match self.open_accounts.entry(open.acc) {
            Occupied(open_entry) => {
                return Err(element
                    .error("account already opened")
                    .related_to(open_entry.get())
                    .into());
            }
            Vacant(open_entry) => {
                open_entry.insert(*element);

                // cannot reopen a closed account
                if let Some(closed) = self.closed_accounts.get(open.acc) {
                    return Err(element
                        .error("account was closed")
                        .related_to(closed)
                        .into());
                } else {
                    let mut booking = open
                        .booking
                        .map(|booking| booking.into())
                        .unwrap_or(self.default_booking);

                    if !is_supported_method(booking) {
                        let default_booking = Booking::default();
                        self.warnings.push(
                            element .warning(format!( "booking method {booking} unsupported, falling back to default {default_booking}" )) .into(),
                        );
                        booking = default_booking;
                    }

                    self.accounts.insert(
                        open.acc,
                        AccountBuilder::new(
                            open.currencies.iter().flatten().copied(),
                            booking,
                            *element,
                        ),
                    );
                }
            }
        }

        if let Some(booking) = open.booking {
            if is_supported_method(booking.into()) {
            } else {
                self.warnings.push(
                    element
                        .warning("booking method {} unsupported, falling back to default")
                        .into(),
                );
            }
        }

        Ok(booked::DirectiveVariant::Open(open.clone()))
    }

    fn close<'r, 'b>(
        &mut self,
        close: &'r raw::Close<'a>,
        _date: Date,
        element: &parser::Spanned<Element<'static>>,
    ) -> Result<booked::DirectiveVariant<'b>, parser::AnnotatedError>
    where
        'a: 'r + 'b,
        'r: 'b,
    {
        use hashbrown::hash_map::Entry::*;
        match self.open_accounts.entry(close.acc) {
            Occupied(open_entry) => {
                match self.closed_accounts.entry(close.acc) {
                    Occupied(closed_entry) => {
                        // cannot reclose a closed account
                        return Err(element
                            .error("account was already closed")
                            .related_to(closed_entry.get())
                            .into());
                    }
                    Vacant(closed_entry) => {
                        open_entry.remove_entry();
                        closed_entry.insert(*element);
                    }
                }
            }
            Vacant(_) => {
                return Err(element.error("account not open").into());
            }
        }

        Ok(booked::DirectiveVariant::Close(close.clone()))
    }

    fn pad<'r, 'b>(
        &mut self,
        pad: &'r raw::Pad<'a>,
        _date: Date,
        idx: usize,
        element: &parser::Spanned<Element<'static>>,
    ) -> Result<booked::DirectiveVariant<'b>, parser::AnnotatedError>
    where
        'a: 'r + 'b,
        'r: 'b,
    {
        let account = self.get_mut_valid_account(pad.acc, element)?;

        let unused_pad_idx = account.pad_idx.replace(idx);

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
            pad.clone(), //     Pad {
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
    balance_currency: &'a str,
    balance_tolerance: Decimal,
    account_rollup: hashbrown::HashMap<&'a str, Decimal>,
) -> HashMap<&'a str, Decimal> {
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
//     margin: &HashMap<&'a str, Decimal>,
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

fn construct_balance_error_and_clear_diagnostics<'a, 'd>(
    account: &mut AccountBuilder<'a, 'd>,
    margin: &HashMap<&'a str, Decimal>,
    element: &parser::Spanned<Element<'static>>,
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

fn adjust_account_to_match_balance<'a, 'd>(
    account: &mut AccountBuilder<'a, 'd>,
    margin: &HashMap<&'a str, Decimal>,
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
struct AccountBuilder<'a, 'd> {
    allowed_currencies: HashSet<&'a str>,
    positions: LoaderPositions<'a>,
    opened: parser::Spanned<Element<'static>>,
    pad_idx: Option<usize>, // index in directives in Loader
    balance_diagnostics: Vec<BalanceDiagnostic<'a, 'd>>,
    booking: Booking,
}

impl<'a, 'd> AccountBuilder<'a, 'd> {
    fn new<I>(
        allowed_currencies: I,
        booking: Booking,
        opened: parser::Spanned<Element<'static>>,
    ) -> Self
    where
        I: Iterator<Item = &'a str>,
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
    fn is_currency_valid(&self, currency: &'a str) -> bool {
        self.allowed_currencies.is_empty() || self.allowed_currencies.contains(currency)
    }

    fn validate_currency(
        &self,
        currency: &'a str,
        element: &parser::Spanned<Element<'static>>,
    ) -> Result<(), parser::AnnotatedError> {
        if self.is_currency_valid(currency) {
            Ok(())
        } else {
            Err(element
                .error("currency incompatible with account")
                .related_to(&self.opened)
                .into())
        }
    }
}

#[derive(Debug)]
struct BalanceDiagnostic<'a, 'd> {
    date: Date,
    description: Option<&'d str>,
    amount: Option<LoaderAmount<'a>>,
    positions: Option<LoaderPositions<'a>>,
}

pub(crate) fn pad_flag() -> parser::Flag {
    parser::Flag::Letter(TryInto::<parser::FlagLetter>::try_into('P').unwrap())
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
            currency: value.currency().item().into(),
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

impl<'a> From<&'_ LoaderAmount<'a>> for Cell<'a, 'static> {
    fn from(value: &'_ LoaderAmount<'a>) -> Self {
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
        label,
        merge,
    } = cost;
    let mut cells = vec![
        (date.to_string(), Align::Left).into(),
        per_unit.into(),
        (Into::<&str>::into(currency), Align::Left).into(),
    ];
    if let Some(label) = label {
        cells.push((label.clone(), Align::Left).into())
    }
    if merge {
        cells.push(("*", Align::Left).into())
    }
    Cell::Row(cells, GUTTER_MINOR)
}

fn loader_cur_posting_cost_to_cost<'a>(
    currency: &'a str,
    cost: limabean_booking::PostingCost<LimaParserBookingTypes<'a>>,
) -> booked::Cost<'a> {
    booked::Cost {
        date: cost.date,
        per_unit: cost.per_unit,
        total: cost.total,
        cur: currency,
        label: cost.label,
        merge: cost.merge,
    }
}

// TODO find where this should go
const GUTTER_MINOR: &str = " ";
const GUTTER_MEDIUM: &str = "  ";
