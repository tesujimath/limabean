use beancount_parser_lima::{self as parser};
use limabean_booking::{
    Booking, Bookings, Interpolated, LimaParserBookingTypes, LimaTolerance, is_supported_method,
};

use rust_decimal::Decimal;
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet, VecDeque},
    fmt::Debug,
};
use tabulator::{Align, Cell};
use time::Date;

use crate::api::types::{ElementIdx, IndexedReport, booked, raw};

#[derive(Debug)]
pub(crate) struct Accumulator<'a, 'd, 't> {
    // hashbrown HashMaps are used here for their Entry API, which is still unstable in std::collections::HashMap
    open_accounts: hashbrown::HashMap<&'a str, ElementIdx>,
    closed_accounts: hashbrown::HashMap<&'a str, ElementIdx>,
    accounts: HashMap<&'a str, AccountBuilder<'a, 'd>>,
    currency_usage: hashbrown::HashMap<&'a str, i32>,
    default_booking: Booking,
    tolerance: &'t LimaTolerance<'a>,
    warnings: Vec<IndexedReport>,
}

pub(crate) struct BookingSuccess<'a> {
    pub(crate) directives: Vec<booked::Directive<'a>>,
    pub(crate) warnings: Vec<IndexedReport>,
}

pub(crate) struct BookingFailure {
    pub(crate) errors: Vec<IndexedReport>,
}

impl<'a, 'd, 't> Accumulator<'a, 'd, 't> {
    pub(crate) fn new(default_booking: Booking, tolerance: &'t LimaTolerance<'a>) -> Self {
        Self {
            open_accounts: hashbrown::HashMap::default(),
            closed_accounts: hashbrown::HashMap::default(),
            accounts: HashMap::default(),
            currency_usage: hashbrown::HashMap::default(),
            default_booking,
            tolerance,
            warnings: Vec::default(),
        }
    }

    // generate any errors before building
    fn validate<'b>(
        self,
        directives: Vec<booked::Directive<'b>>,
        mut errors: Vec<IndexedReport>,
    ) -> Result<BookingSuccess<'b>, BookingFailure>
    where
        'a: 'b,
    {
        let Self {
            accounts, warnings, ..
        } = self;

        // check for unused pad directives
        for account in accounts.values() {
            if let Some((_, pad)) = &account.pad {
                errors.push(pad.report("unused, no balance directive"))
            }
        }

        if errors.is_empty() {
            Ok(BookingSuccess {
                directives,
                warnings,
            })
        } else {
            Err(BookingFailure { errors })
        }
    }

    pub(crate) fn collect<'r, 'b, I>(
        mut self,
        directives: I,
    ) -> Result<BookingSuccess<'b>, BookingFailure>
    where
        'a: 'r + 'b + 'd,
        'r: 'b + 'd,
        I: IntoIterator<Item = &'r raw::Directive<'a>>,
    {
        let mut errors = Vec::default();
        let mut booked_directives = Vec::default();

        for (raw_idx, raw) in directives.into_iter().enumerate() {
            match self.directive(
                raw,
                raw_idx.into(),
                booked_directives.len(),
                &mut booked_directives,
            ) {
                Ok((booked_variant, pad_txn)) => {
                    booked_directives.push(booked::Directive {
                        raw_idx,
                        date: raw.date,
                        tags: raw.tags.clone(),
                        links: raw.links.clone(),
                        metadata: raw.metadata.clone(),
                        variant: booked_variant,
                    });

                    if let Some(pad_txn) = pad_txn {
                        booked_directives.push(booked::Directive {
                            raw_idx,
                            date: raw.date,
                            tags: raw.tags.clone(),
                            links: raw.links.clone(),
                            metadata: raw.metadata.clone(),
                            variant: pad_txn,
                        });
                    }
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
        element: ElementIdx,
        booked_idx: usize,
        booked_directives: &mut Vec<booked::Directive<'b>>,
    ) -> Result<
        (
            booked::DirectiveVariant<'b>,
            Option<booked::DirectiveVariant<'b>>,
        ),
        IndexedReport,
    >
    where
        'a: 'r + 'b + 'd,
        'r: 'b + 'd,
    {
        use booked::DirectiveVariant as BDV;
        use raw::DirectiveVariant as RDV;

        let date = directive.date;

        match &directive.variant {
            RDV::Transaction(transaction) => self
                .transaction(transaction, date, element)
                .map(|x| (x, None)),
            RDV::Price(price) => Ok((BDV::Price(price.clone()), None)),
            RDV::Balance(balance) => self
                .balance(balance, date, element, booked_directives)
                .map(|x| (x, None)),
            RDV::Open(open) => self.open(open, date, element).map(|x| (x, None)),
            RDV::Close(close) => self.close(close, date, element).map(|x| (x, None)),
            RDV::Commodity(commodity) => Ok((BDV::Commodity(commodity.clone()), None)),
            RDV::Pad(pad) => self.pad(pad, date, booked_idx, element, booked_directives),
            RDV::Document(document) => Ok((BDV::Document(document.clone()), None)),
            RDV::Note(note) => Ok((BDV::Note(note.clone()), None)),
            RDV::Event(event) => Ok((BDV::Event(event.clone()), None)),
            RDV::Query(query) => Ok((BDV::Query(query.clone()), None)),
            RDV::Custom(custom) => Ok((BDV::Custom(custom.clone()), None)),
        }
    }

    fn transaction<'r, 'b>(
        &mut self,
        transaction: &'r raw::Transaction<'a>,
        date: Date,
        element: ElementIdx,
    ) -> Result<booked::DirectiveVariant<'b>, IndexedReport>
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

        let booked_postings = self.book(date, &transaction.postings, description, element)?;

        Ok(booked::DirectiveVariant::Transaction(booked::Transaction {
            flag: transaction.flag.clone(),
            payee: transaction.payee.as_ref().map(|payee| payee.as_ref()),
            narration: transaction
                .narration
                .as_ref()
                .map(|narration| narration.as_ref()),
            postings: booked_postings,
        }))
    }

    fn book<'r, 'b>(
        &mut self,
        date: Date,
        postings: &'r [raw::PostingSpec<'a>],
        description: &'d str,
        element: ElementIdx,
    ) -> Result<Vec<booked::Posting<'a>>, IndexedReport>
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
                // check all postings have valid accounts and currencies
                // returning the first error
                if let Some(error) = interpolated_postings
                    .iter()
                    .zip(&postings)
                    .enumerate()
                    .filter_map(|(posting_idx, (interpolated, posting))| {
                        let posting_element = (element, posting_idx).into();
                        self.validate_account_and_currency(
                            posting.acc,
                            interpolated.currency,
                            posting_element,
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
                    .enumerate()
                    .flat_map(|(posting_idx, (interpolated, posting))| {
                        let account = posting.acc;
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
                                .map(|(cost_cur, cost)| booked::Posting {
                                    raw_idx: Some(posting_idx),
                                    flag: posting.flag.clone(),
                                    acc: account,
                                    units: cost.units,
                                    cur: currency,
                                    cost: Some(cur_posting_cost_to_cost(cost_cur, cost)),
                                    price: None,
                                    tags: posting.tags.clone(),
                                    links: posting.links.clone(),
                                    metadata: posting.metadata.clone(),
                                })
                                .collect::<Vec<_>>()
                        } else {
                            vec![booked::Posting {
                                raw_idx: Some(posting_idx),
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
                    hashbrown::HashMap::<&str, VecDeque<Amount<'_>>>::new();
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

                Ok(booked_postings)
            }
            Err(e) => {
                use limabean_booking::BookingError::*;

                match &e {
                    Transaction(e) => Err(element.report(e.to_string())),
                    Posting(idx, e) => {
                        // TODO attach posting error to actual posting
                        // let bad_posting = postings[*idx];
                        // bad_posting.error(e.to_string()).into()
                        Err(element.report(format!("{e} on posting {idx}")))
                    }
                }
            }
        }
    }

    fn validate_account(
        &self,
        account_name: &'a str,
        element: ElementIdx,
    ) -> Result<(), IndexedReport> {
        if self.open_accounts.contains_key(account_name) {
            Ok(())
        } else if let Some(closed) = self.closed_accounts.get(account_name) {
            Err(element.report("account was closed").related_to(*closed))
        } else {
            Err(element.report("account not open"))
        }
    }

    fn get_valid_account(
        &self,
        account_name: &'a str,
        element: ElementIdx,
    ) -> Result<&AccountBuilder<'a, 'd>, IndexedReport> {
        self.validate_account(account_name, element)?;
        Ok(self.accounts.get(account_name).unwrap())
    }

    fn get_mut_valid_account(
        &mut self,
        account_name: &'a str,
        element: ElementIdx,
    ) -> Result<&mut AccountBuilder<'a, 'd>, IndexedReport> {
        self.validate_account(account_name, element)?;
        Ok(self.accounts.get_mut(account_name).unwrap())
    }

    fn validate_account_and_currency(
        &self,
        account_name: &'a str,
        currency: &'a str,
        element: ElementIdx,
    ) -> Result<(), IndexedReport> {
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

    // get the total units for given currency in an account
    fn total_units_for_currency(&self, account_name: &str, currency: &str) -> Decimal {
        self.accounts
            .get(account_name)
            .map(|account| {
                account
                    .positions
                    .units()
                    .iter()
                    .filter_map(|(cur, number)| (**cur == currency).then_some(*number))
                    .sum()
            })
            .unwrap_or_default()
        // }
    }

    fn balance<'r, 'b>(
        &mut self,
        balance: &'r raw::Balance<'a>,
        date: Date,
        element: ElementIdx,
        booked_directives: &mut [booked::Directive<'b>],
    ) -> Result<booked::DirectiveVariant<'b>, IndexedReport>
    where
        'a: 'r + 'b,
        'r: 'b,
    {
        let margin = calculate_balance_margin(
            balance.units,
            balance.tolerance.unwrap_or(Decimal::ZERO),
            self.total_units_for_currency(balance.acc, balance.cur),
        );

        let account = self.get_mut_valid_account(balance.acc, element)?;
        account.validate_currency(balance.cur, element)?;
        // pad can't last beyond balance
        let pad = account.pad.take();

        if margin.is_none() {
            // balance assertion is correct, and we already cleared the pad, so:

            account.balance_diagnostics.clear();

            // but if there was a pad directive, we ought to have used it, so:
            if let Some((_, pad_idx)) = pad {
                return Err(pad_idx.report("unused, no balance adjustment required"));
            } else {
                return Ok(booked::DirectiveVariant::Balance(balance.clone()));
            }
        }
        let margin = margin.unwrap();

        if pad.is_none() {
            // balance assertion is incorrect and we have no pad to take up the slack, so:

            let err = Err(construct_balance_error_and_clear_diagnostics(
                account,
                balance.cur,
                margin,
                element,
            ));

            // even though we have a balance error, we adjust the account to match, in order to localise balance failures
            adjust_account_to_match_balance(account, balance.cur, margin, Adjustment::Add);

            return err;
        }
        let (booked_pad_idx, _) = pad.unwrap();

        adjust_account_to_match_balance(account, balance.cur, margin, Adjustment::Add);
        account.balance_diagnostics.clear();

        // initialize balance diagnostics according to balance assertion
        let mut positions = Positions::default();
        positions.accumulate(balance.units, balance.cur, None, Booking::default());
        account.balance_diagnostics.push(BalanceDiagnostic {
            date,
            description: None,
            amount: None,
            positions: Some(positions),
        });

        let booked::DirectiveVariant::Pad(pad) = &booked_directives[booked_pad_idx].variant else {
            panic!(
                "directive at pad_idx {} is not a pad, is {:?}",
                booked_pad_idx, &booked_directives[booked_pad_idx]
            );
        };
        let pad_source = pad.source;

        let booked::DirectiveVariant::Transaction(txn) =
            &mut booked_directives[booked_pad_idx + 1].variant
        else {
            panic!(
                "directive at pad_idx {} is not a pad, is {:?}",
                booked_pad_idx + 1,
                &booked_directives[booked_pad_idx + 1]
            );
        };

        txn.postings = calculate_balance_pad_postings(balance.cur, margin, balance.acc, pad_source);

        let pad_account = self.accounts.get_mut(pad_source).unwrap();
        adjust_account_to_match_balance(pad_account, balance.cur, margin, Adjustment::Subtract);

        Ok(booked::DirectiveVariant::Balance(balance.clone()))
    }

    fn open<'r, 'b>(
        &mut self,
        open: &'r raw::Open<'a>,
        _date: Date,
        element: ElementIdx,
    ) -> Result<booked::DirectiveVariant<'b>, IndexedReport>
    where
        'a: 'r + 'b,
        'r: 'b,
    {
        use hashbrown::hash_map::Entry::*;
        match self.open_accounts.entry(open.acc) {
            Occupied(open_entry) => {
                return Err(element
                    .report("account already opened")
                    .related_to(*open_entry.get()));
            }
            Vacant(open_entry) => {
                open_entry.insert(element);

                // cannot reopen a closed account
                if let Some(closed) = self.closed_accounts.get(open.acc) {
                    return Err(element.report("account was closed").related_to(*closed));
                } else {
                    let mut booking = open
                        .booking
                        .map(|booking| booking.into())
                        .unwrap_or(self.default_booking);

                    if !is_supported_method(booking) {
                        let default_booking = Booking::default();
                        self.warnings.push(
                            element.report(format!( "booking method {booking} unsupported, falling back to default {default_booking}" )),
                        );
                        booking = default_booking;
                    }

                    self.accounts.insert(
                        open.acc,
                        AccountBuilder::new(
                            open.currencies.iter().flatten().copied(),
                            booking,
                            element,
                        ),
                    );
                }
            }
        }

        if let Some(booking) = open.booking {
            if is_supported_method(booking.into()) {
            } else {
                self.warnings
                    .push(element.report("booking method {} unsupported, falling back to default"));
            }
        }

        Ok(booked::DirectiveVariant::Open(open.clone()))
    }

    fn close<'r, 'b>(
        &mut self,
        close: &'r raw::Close<'a>,
        _date: Date,
        element: ElementIdx,
    ) -> Result<booked::DirectiveVariant<'b>, IndexedReport>
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
                            .report("account was already closed")
                            .related_to(*closed_entry.get()));
                    }
                    Vacant(closed_entry) => {
                        open_entry.remove_entry();
                        closed_entry.insert(element);
                    }
                }
            }
            Vacant(_) => {
                return Err(element.report("account not open"));
            }
        }

        Ok(booked::DirectiveVariant::Close(close.clone()))
    }

    fn pad<'r, 'b>(
        &mut self,
        pad: &'r raw::Pad<'a>,
        _date: Date,
        idx: usize,
        element: ElementIdx,
        booked_directives: &[booked::Directive<'b>],
    ) -> Result<
        (
            booked::DirectiveVariant<'b>,
            Option<booked::DirectiveVariant<'b>>,
        ),
        IndexedReport,
    >
    where
        'a: 'r + 'b,
        'r: 'b,
    {
        let account = self.get_mut_valid_account(pad.acc, element)?;

        let unused_pad = account.pad.take();

        // unused pad directives are errors
        // https://beancount.github.io/docs/beancount_language_syntax.html#unused-pad-directives
        if let Some((_, unused_pad)) = unused_pad {
            return Err(unused_pad
                .report("unused, second pad encountered")
                .related_to(element));
        }

        account.pad = Some((idx, element));

        Ok((
            booked::DirectiveVariant::Pad(pad.clone()),
            Some(booked::DirectiveVariant::Transaction(booked::Transaction {
                flag: Cow::Borrowed(PAD_FLAG),
                payee: None,
                narration: None,
                postings: Vec::default(),
            })),
        ))
    }
}

fn calculate_balance_margin(
    balance_units: Decimal,
    balance_tolerance: Decimal,
    account_units: Decimal,
) -> Option<Decimal> {
    // what's the gap between what we have and what the balance says we should have?
    let margin = balance_units - account_units;
    (margin.abs() > balance_tolerance).then_some(margin)
}

fn calculate_balance_pad_postings<'a>(
    cur: &'a str,
    margin: Decimal,
    balance_account: &'a str,
    pad_source: &'a str,
) -> Vec<booked::Posting<'a>> {
    vec![
        booked::Posting {
            raw_idx: None,
            flag: Some(Cow::Borrowed(PAD_FLAG)),
            acc: balance_account,
            units: margin,
            cur,
            cost: None,
            price: None,
            tags: None,
            links: None,
            metadata: None,
        },
        booked::Posting {
            raw_idx: None,
            flag: Some(Cow::Borrowed(PAD_FLAG)),
            acc: pad_source,
            units: -margin,
            cur,
            cost: None,
            price: None,
            tags: None,
            links: None,
            metadata: None,
        },
    ]
}

fn construct_balance_error_and_clear_diagnostics<'a, 'd>(
    account: &mut AccountBuilder<'a, 'd>,
    cur: &'a str,
    margin: Decimal,
    element: ElementIdx,
) -> IndexedReport {
    let reason = format!(
        "accumulated {}, error {} {}",
        if account.positions.is_empty() {
            "zero".to_string()
        } else {
            account.positions.to_string()
        },
        margin,
        cur,
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
        .report(reason)
        .with_annotation(annotation.to_string())
}

#[derive(PartialEq, Eq, Debug)]
enum Adjustment {
    Add,
    Subtract,
}

fn adjust_account_to_match_balance<'a, 'd>(
    account: &mut AccountBuilder<'a, 'd>,
    cur: &'a str,
    units: Decimal,
    adjustment: Adjustment,
) {
    use Adjustment::*;

    // reset accumulated balance to what was asserted, to localise errors
    account.positions.accumulate(
        if adjustment == Add { units } else { -units },
        cur,
        None,
        Booking::default(),
    );
    // booking method doesn't matter if no cost
}

#[derive(Debug)]
struct AccountBuilder<'a, 'd> {
    allowed_currencies: HashSet<&'a str>,
    positions: Positions<'a>,
    opened: ElementIdx,
    pad: Option<(usize, ElementIdx)>,
    balance_diagnostics: Vec<BalanceDiagnostic<'a, 'd>>,
    booking: Booking,
}

impl<'a, 'd> AccountBuilder<'a, 'd> {
    fn new<I>(allowed_currencies: I, booking: Booking, opened: ElementIdx) -> Self
    where
        I: Iterator<Item = &'a str>,
    {
        AccountBuilder {
            allowed_currencies: allowed_currencies.collect(),
            positions: Positions::default(),
            opened,
            pad: None,
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
        element: ElementIdx,
    ) -> Result<(), IndexedReport> {
        if self.is_currency_valid(currency) {
            Ok(())
        } else {
            Err(element
                .report("currency incompatible with account")
                .related_to(self.opened))
        }
    }
}

#[derive(Debug)]
struct BalanceDiagnostic<'a, 'd> {
    date: Date,
    description: Option<&'d str>,
    amount: Option<Amount<'a>>,
    positions: Option<Positions<'a>>,
}

const PAD_FLAG: &str = "'P";

#[derive(PartialEq, Eq, Clone, Debug)]
struct Amount<'a> {
    number: Decimal,
    currency: &'a str,
}

impl<'a> From<(Decimal, &'a str)> for Amount<'a> {
    fn from(value: (Decimal, &'a str)) -> Self {
        Self {
            number: value.0,
            currency: value.1,
        }
    }
}

impl<'a> From<&'a parser::Amount<'a>> for Amount<'a> {
    fn from(value: &'a parser::Amount<'a>) -> Self {
        Amount {
            number: value.number().value(),
            currency: value.currency().item().into(),
        }
    }
}

impl<'a> From<Amount<'a>> for Cell<'static, 'static> {
    fn from(value: Amount) -> Self {
        Cell::Row(
            vec![
                value.number.into(),
                (value.currency.to_string(), Align::Left).into(),
            ],
            GUTTER_MINOR,
        )
    }
}

impl<'a> From<&'_ Amount<'a>> for Cell<'a, 'static> {
    fn from(value: &'_ Amount<'a>) -> Self {
        Cell::Row(
            vec![value.number.into(), (value.currency, Align::Left).into()],
            GUTTER_MINOR,
        )
    }
}

type Positions<'a> = limabean_booking::Positions<limabean_booking::LimaParserBookingTypes<'a>>;

// should be From, but both types are third-party
fn positions_into_cell<'a>(positions: Positions<'a>) -> Cell<'a, 'static> {
    Cell::Stack(
        positions
            .into_iter()
            .map(position_into_cell)
            .collect::<Vec<_>>(),
    )
}

type Position<'a> = limabean_booking::Position<limabean_booking::LimaParserBookingTypes<'a>>;

fn position_into_cell<'a>(position: Position<'a>) -> Cell<'a, 'static> {
    let Position {
        units,
        currency,
        cost,
    } = position;
    let mut cells = vec![
        units.into(),
        (Into::<&str>::into(currency), Align::Left).into(),
    ];
    if let Some(cost) = cost {
        cells.push(cost_into_cell(cost))
    }
    Cell::Row(cells, GUTTER_MINOR)
}

type Cost<'a> = limabean_booking::Cost<limabean_booking::LimaParserBookingTypes<'a>>;
fn cost_into_cell<'a>(cost: Cost<'a>) -> Cell<'a, 'static> {
    let Cost {
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

fn cur_posting_cost_to_cost<'a>(
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

const GUTTER_MINOR: &str = " ";
const GUTTER_MEDIUM: &str = "  ";
