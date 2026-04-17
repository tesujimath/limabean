use beancount_parser_lima::{self as parser};
use limabean_booking::{
    Booking, Bookings, Interpolated, LimaParserBookingTypes, LimaTolerance, is_supported_method,
};

use rust_decimal::Decimal;
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    fmt::Debug,
    ops::Range,
};
use tabulator::{Align, Cell};
use time::Date;

use crate::api::types::{ElementIdx, IndexedReport, booked, raw};

#[derive(Debug)]
pub(crate) struct Accumulator<'a, 't> {
    // hashbrown HashMaps are used here for their Entry API, which is still unstable in std::collections::HashMap
    open_accounts: hashbrown::HashMap<&'a str, ElementIdx>,
    closed_accounts: hashbrown::HashMap<&'a str, ElementIdx>,
    accounts: HashMap<&'a str, AccountBuilder<'a>>,
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

impl<'a, 't> Accumulator<'a, 't> {
    pub(crate) fn new(default_booking: Booking, tolerance: &'t LimaTolerance<'a>) -> Self {
        Self {
            open_accounts: hashbrown::HashMap::default(),
            closed_accounts: hashbrown::HashMap::default(),
            accounts: HashMap::default(),
            default_booking,
            tolerance,
            warnings: Vec::default(),
        }
    }

    // generate any errors before building
    fn validate(
        self,
        directives: Vec<booked::Directive<'a>>,
        mut errors: Vec<IndexedReport>,
    ) -> Result<BookingSuccess<'a>, BookingFailure> {
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

    pub(crate) fn collect<I>(mut self, directives: I) -> Result<BookingSuccess<'a>, BookingFailure>
    where
        I: IntoIterator<Item = &'a raw::Directive<'a>>,
    {
        let mut errors = Vec::default();
        let mut booked_directives = Vec::default();

        for (raw_idx, raw) in directives.into_iter().enumerate() {
            let raw_element_idx = raw_idx.into();
            match self.directive(raw, raw_element_idx, &mut booked_directives) {
                Ok((booked_variant, pad_txn)) => {
                    if let booked::DirectiveVariant::Balance(booked::Balance {
                        raw,
                        unused_pad,
                        margin,
                    }) = &booked_variant
                    {
                        if let Some(unused_pad) = unused_pad {
                            errors
                                .push(unused_pad.report("unused, no balance adjustment required"));
                        }
                        if let Some(margin) = margin {
                            let e = self.balance_report(
                                raw.acc,
                                *margin,
                                raw.cur,
                                raw_element_idx,
                                &booked_directives,
                            );
                            errors.push(e);
                        }
                    }

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

    fn directive(
        &mut self,
        directive: &'a raw::Directive<'a>,
        element: ElementIdx,
        booked_directives: &mut Vec<booked::Directive<'a>>,
    ) -> Result<
        (
            booked::DirectiveVariant<'a>,
            Option<booked::DirectiveVariant<'a>>,
        ),
        IndexedReport,
    > {
        use booked::DirectiveVariant as BDV;
        use raw::DirectiveVariant as RDV;

        let date = directive.date;

        match &directive.variant {
            RDV::Transaction(transaction) => self
                .transaction(transaction, date, element)
                .map(|x| (x, None)),
            RDV::Price(price) => Ok((BDV::Price(price.clone()), None)),
            RDV::Balance(balance) => self
                .balance(balance, element, booked_directives)
                .map(|x| (x, None)),
            RDV::Open(open) => self.open(open, date, element).map(|x| (x, None)),
            RDV::Close(close) => self.close(close, date, element).map(|x| (x, None)),
            RDV::Commodity(commodity) => Ok((BDV::Commodity(commodity.clone()), None)),
            RDV::Pad(pad) => self.pad(pad, date, booked_directives.len(), element),
            RDV::Document(document) => Ok((BDV::Document(document.clone()), None)),
            RDV::Note(note) => Ok((BDV::Note(note.clone()), None)),
            RDV::Event(event) => Ok((BDV::Event(event.clone()), None)),
            RDV::Query(query) => Ok((BDV::Query(query.clone()), None)),
            RDV::Custom(custom) => Ok((BDV::Custom(custom.clone()), None)),
        }
    }

    fn transaction(
        &mut self,
        transaction: &'a raw::Transaction<'a>,
        date: Date,
        element: ElementIdx,
    ) -> Result<booked::DirectiveVariant<'a>, IndexedReport> {
        let booked_postings = self.book(date, &transaction.postings, element)?;

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

    fn book(
        &mut self,
        date: Date,
        postings: &'a [raw::PostingSpec<'a>],
        element: ElementIdx,
    ) -> Result<Vec<booked::Posting<'a>>, IndexedReport> {
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

                for (account_name, updated_positions) in updated_inventory {
                    let account = self.get_mut_valid_account(account_name, element)?;

                    account.positions = updated_positions;
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
    ) -> Result<&AccountBuilder<'a>, IndexedReport> {
        self.validate_account(account_name, element)?;
        Ok(self.accounts.get(account_name).unwrap())
    }

    fn get_mut_valid_account(
        &mut self,
        account_name: &'a str,
        element: ElementIdx,
    ) -> Result<&mut AccountBuilder<'a>, IndexedReport> {
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

    fn account_and_subaccounts(
        &self,
        base_account_name: &'_ str,
    ) -> impl Iterator<Item = &AccountBuilder<'a>> {
        // base account is known
        self.accounts
            .iter()
            .filter_map(move |(candidate, account)| {
                is_nonstrict_subaccount(base_account_name, candidate).then_some(account)
            })
    }

    // get the total units for given currency in an account and all its subaccounts
    fn total_rollup_units_for_currency(&self, base_account_name: &str, currency: &str) -> Decimal {
        self.account_and_subaccounts(base_account_name)
            .map(|account| {
                account
                    .positions
                    .units()
                    .get(&currency)
                    .copied()
                    .unwrap_or(Decimal::ZERO)
            })
            .sum()
    }

    fn balance(
        &mut self,
        balance: &'a raw::Balance<'a>,
        element: ElementIdx,
        booked_directives: &mut [booked::Directive<'a>],
    ) -> Result<booked::DirectiveVariant<'a>, IndexedReport> {
        let margin = calculate_balance_margin(
            balance.units,
            balance.tolerance.unwrap_or(Decimal::ZERO),
            self.total_rollup_units_for_currency(balance.acc, balance.cur),
        );

        let account = self.get_mut_valid_account(balance.acc, element)?;
        account.validate_currency(balance.cur, element)?;

        let new_window_end = booked_directives.len();
        let new_window = match account.balance_window.take() {
            Some(old_window) => old_window.end..new_window_end,
            None => 0..new_window_end,
        };
        account.balance_window = Some(new_window);

        // pad can't last beyond balance
        let pad = account.pad.take();

        if margin.is_none() {
            // if there was a pad directive, we ought to have used it, so:
            let unused_pad = pad.map(|(_, pad_idx)| pad_idx);
            return Ok(booked::DirectiveVariant::Balance(booked::Balance {
                raw: balance.clone(),
                unused_pad,
                margin: None,
            }));
        }
        let margin = margin.unwrap();

        if pad.is_none() {
            // even though we have a balance error, we adjust the account to match, in order to localise balance failures
            adjust_account_to_match_balance(account, balance.cur, margin, Adjustment::Add);

            return Ok(booked::DirectiveVariant::Balance(booked::Balance {
                raw: balance.clone(),
                unused_pad: None,
                margin: Some(margin),
            }));
        }
        let (booked_pad_idx, _) = pad.unwrap();

        adjust_account_to_match_balance(account, balance.cur, margin, Adjustment::Add);

        // initialize balance diagnostics according to balance assertion
        let mut positions = Positions::default();
        positions.accumulate(balance.units, balance.cur, None, Booking::default());

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

        Ok(booked::DirectiveVariant::Balance(booked::Balance {
            raw: (balance.clone()),
            unused_pad: None,
            margin: None,
        }))
    }

    fn balance_report(
        &self,
        base_account_name: &'a str,
        margin: Decimal,
        cur: &'a str,
        element: ElementIdx,
        booked_directives: &[booked::Directive<'a>],
    ) -> IndexedReport {
        let base_account = self.accounts.get(base_account_name).unwrap();
        let balance_window = base_account.balance_window.as_ref().unwrap();

        let mut diagnostics = Vec::default();

        let mut total = if balance_window.start < booked_directives.len()
            && let booked::DirectiveVariant::Balance(bal) =
                &booked_directives[balance_window.start].variant
        {
            if bal.raw.cur == cur {
                // let bal_date = booked_directives[balance_window.start].date;
                // diagnostics.push((bal_date, None, bal.raw.units));

                bal.raw.units
            } else {
                Decimal::ZERO
            }
        } else {
            Decimal::ZERO
        };

        for dct in booked_directives {
            if let booked::DirectiveVariant::Transaction(txn) = &dct.variant {
                for pst in &txn.postings {
                    if is_nonstrict_subaccount(base_account_name, pst.acc) {
                        total += pst.units;
                        let description = txn.payee.or(txn.narration);

                        diagnostics.push((dct.date, pst.acc, pst.units, total, description));
                    }
                }
            }
        }

        let reason = format!("accumulated {}, error {} {}", total, margin, cur,);

        // determine context for error by collating postings since last balance
        let annotation = Cell::Stack(
            diagnostics
                .into_iter()
                .map(|(date, acc, units, total, description)| {
                    Cell::Row(
                        vec![
                            (date.to_string(), Align::Left).into(),
                            (acc.to_string(), Align::Left).into(),
                            units.into(),
                            total.into(),
                            description
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

    fn open(
        &mut self,
        open: &'a raw::Open<'a>,
        _date: Date,
        element: ElementIdx,
    ) -> Result<booked::DirectiveVariant<'a>, IndexedReport> {
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

    fn close(
        &mut self,
        close: &'a raw::Close<'a>,
        _date: Date,
        element: ElementIdx,
    ) -> Result<booked::DirectiveVariant<'a>, IndexedReport> {
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

    fn pad(
        &mut self,
        pad: &'a raw::Pad<'a>,
        _date: Date,
        idx: usize,
        element: ElementIdx,
    ) -> Result<
        (
            booked::DirectiveVariant<'a>,
            Option<booked::DirectiveVariant<'a>>,
        ),
        IndexedReport,
    > {
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

/// is candidate equal or a subaccount of base_account_name
fn is_nonstrict_subaccount(base_account_name: &str, candidate: &str) -> bool {
    // base account is known
    candidate
        .strip_prefix(base_account_name)
        .is_some_and(|s| s.is_empty() || s.starts_with(':'))
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

#[derive(PartialEq, Eq, Debug)]
enum Adjustment {
    Add,
    Subtract,
}

fn adjust_account_to_match_balance<'a>(
    account: &mut AccountBuilder<'a>,
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
struct AccountBuilder<'a> {
    allowed_currencies: HashSet<&'a str>,
    positions: Positions<'a>,
    opened: ElementIdx,
    pad: Option<(usize, ElementIdx)>,
    balance_window: Option<Range<usize>>,
    booking: Booking,
}

impl<'a> AccountBuilder<'a> {
    fn new<I>(allowed_currencies: I, booking: Booking, opened: ElementIdx) -> Self
    where
        I: Iterator<Item = &'a str>,
    {
        AccountBuilder {
            allowed_currencies: allowed_currencies.collect(),
            positions: Positions::default(),
            opened,
            pad: None,
            balance_window: None,
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
