use hashbrown::HashMap;
use std::{fmt::Debug, iter::repeat_n};

use super::{
    AnnotatedPosting, Booking, BookingError, BookingTypes, Bookings, CategorizedByCurrency,
    Interpolated, Interpolation, Inventory, Positions, PostingSpec, Reductions, Tolerance,
    TransactionBookingError, book_reductions, categorize_by_currency, interpolate_from_costed,
};

/// Whether the given booking method is supported by this crate.
///
/// See this [issue for the current status of the average booking method](https://github.com/tesujimath/limabean/issues/7).
pub fn is_supported_method(method: Booking) -> bool {
    use Booking::*;

    match method {
        Strict => true,
        StrictWithSize => true,
        None => true,
        Average => false,
        Fifo => true,
        Lifo => true,
        Hifo => true,
    }
}

/// Book the postings for the given date, returning updated inventory and interpolated postings.
/// The interpolated postings are aligned with the original postings, in that they may be zipped together and
/// will always correspond.
pub fn book<'a, 'b, B, P, T, I, M>(
    date: B::Date,
    postings: &[P],
    tolerance: T,
    inventory: I,
    method: M,
) -> Result<Bookings<B, P>, BookingError>
where
    B: BookingTypes + 'a,
    P: PostingSpec<Types = B> + Debug + 'a,
    T: Tolerance<Types = B> + Copy,
    I: Fn(B::Account) -> Option<&'b Positions<B>> + Copy,
    M: Fn(B::Account) -> Booking + Copy,
    'a: 'b,
{
    let BookingsAndResiduals {
        bookings,
        residuals,
    } = book_with_residuals(date, postings, tolerance, inventory, method)?;
    if !residuals.is_empty() {
        let mut currencies = residuals.keys().collect::<Vec<_>>();
        currencies.sort();
        let message = currencies
            .into_iter()
            .map(|cur| format!("{} {}", -*residuals.get(cur).unwrap(), cur))
            .collect::<Vec<String>>()
            .join(", ");
        return Err(BookingError::Transaction(
            TransactionBookingError::Unbalanced(message),
        ));
    }

    Ok(bookings)
}

pub(crate) type Residuals<C, N> = HashMap<C, N>;

pub(crate) struct BookingsAndResiduals<B, P>
where
    B: BookingTypes,
    P: PostingSpec<Types = B> + Debug,
{
    pub(crate) bookings: Bookings<B, P>,
    pub(crate) residuals: Residuals<B::Currency, B::Number>,
}

// this exists so we can test the booking algorithm with unbalanced transactions
// as per OG Beancount booking_full_test.py
pub(crate) fn book_with_residuals<'a, 'b, B, P, T, I, M>(
    date: B::Date,
    postings: &[P],
    tolerance: T,
    inventory: I,
    method: M,
) -> Result<BookingsAndResiduals<B, P>, BookingError>
where
    B: BookingTypes + 'a,
    P: PostingSpec<Types = B> + Debug + 'a,
    T: Tolerance<Types = B> + Copy,
    I: Fn(B::Account) -> Option<&'b Positions<B>> + Copy,
    M: Fn(B::Account) -> Booking + Copy,
    'a: 'b,
{
    let CategorizedByCurrency(currency_groups) = categorize_by_currency(postings, inventory)?;

    let mut booking_accumulator = BookingAccumulator::new(postings.len());
    // let mut interpolated_postings = repeat_n(None, postings.len()).collect::<Vec<_>>();
    // let mut updated_inventory = Inventory::default();
    // let mut residuals = Residuals::<B::Currency, B::Number>::default();

    for (cur, annotated_postings) in currency_groups {
        book_currency_group(
            date,
            cur,
            annotated_postings,
            tolerance,
            inventory,
            method,
            &mut booking_accumulator,
        )?;
    }

    let BookingAccumulator {
        interpolated_postings,
        updated_inventory,
        residuals,
    } = booking_accumulator;

    let interpolated_postings = interpolated_postings
        .into_iter()
        .map(|p| p.unwrap())
        .collect::<Vec<_>>();

    Ok(BookingsAndResiduals {
        bookings: Bookings {
            interpolated_postings,
            updated_inventory,
        },
        residuals,
    })
}

struct BookingAccumulator<B, P>
where
    B: BookingTypes,
    P: PostingSpec<Types = B>,
{
    interpolated_postings: Vec<Option<Interpolated<B, P>>>,
    updated_inventory: Inventory<B>,
    residuals: Residuals<B::Currency, B::Number>,
}

impl<B, P> BookingAccumulator<B, P>
where
    B: BookingTypes,
    P: PostingSpec<Types = B>,
{
    fn new(n_postings: usize) -> Self {
        BookingAccumulator {
            interpolated_postings: repeat_n(None, n_postings).collect::<Vec<_>>(),
            updated_inventory: Inventory::default(),
            residuals: Residuals::<B::Currency, B::Number>::default(),
        }
    }
}

fn book_currency_group<'a, 'b, B, P, T, I, M>(
    date: B::Date,
    cur: B::Currency,
    annotated_postings: Vec<AnnotatedPosting<P, B::Currency>>,
    tolerance: T,
    inventory: I,
    method: M,
    accumulator: &mut BookingAccumulator<B, P>,
) -> Result<(), BookingError>
where
    B: BookingTypes + 'a,
    P: PostingSpec<Types = B> + Debug + 'a,
    T: Tolerance<Types = B> + Copy,
    I: Fn(B::Account) -> Option<&'b Positions<B>> + Copy,
    M: Fn(B::Account) -> Booking + Copy,
    'a: 'b,
{
    let Reductions {
        updated_inventory: updated_inventory_for_cur,
        postings: costed_postings,
    } = book_reductions(
        annotated_postings,
        tolerance,
        |account| {
            accumulator
                .updated_inventory
                .get(&account)
                .or_else(|| inventory(account.clone()))
        },
        method,
    )?;

    incorporate_inventory_updates::<B>(
        updated_inventory_for_cur,
        &mut accumulator.updated_inventory,
    );

    let Interpolation {
        booked_and_unbooked_postings,
        residual,
    } = interpolate_from_costed(date, &cur, costed_postings, tolerance)?;

    if let Some(residual) = residual {
        accumulator.residuals.insert(cur.clone(), residual);
    }

    let updated_inventory_for_cur = book_augmentations(
        booked_and_unbooked_postings
            .iter()
            .filter_map(|(p, booked)| (!booked).then_some(p)),
        |account| {
            accumulator
                .updated_inventory
                .get(&account)
                .or_else(|| inventory(account.clone()))
        },
        method,
    )?;

    incorporate_inventory_updates::<B>(
        updated_inventory_for_cur,
        &mut accumulator.updated_inventory,
    );

    for (p, _) in booked_and_unbooked_postings.into_iter() {
        let idx = p.idx;
        accumulator.interpolated_postings[idx] = Some(p);
    }

    Ok(())
}

fn incorporate_inventory_updates<B>(updates: Inventory<B>, inventory: &mut Inventory<B>)
where
    B: BookingTypes,
{
    for (account, positions) in updates {
        inventory.insert(account, positions);
    }
}

fn book_augmentations<'a, 'b, B, P, I, M>(
    interpolateds: impl Iterator<Item = &'b Interpolated<B, P>>,
    inventory: I,
    method: M,
) -> Result<Inventory<B>, BookingError>
where
    B: BookingTypes + 'a,
    P: PostingSpec<Types = B> + Debug + 'a,
    I: Fn(B::Account) -> Option<&'a Positions<B>> + Copy,
    M: Fn(B::Account) -> Booking + Copy,
    'a: 'b,
{
    let mut updated_inventory = HashMap::default();

    for interpolated in interpolateds {
        use hashbrown::hash_map::Entry::*;

        let posting = &interpolated.posting;
        let account = posting.account();
        let account_method = method(account.clone());

        let previous_positions = match updated_inventory.entry(account.clone()) {
            Occupied(entry) => entry.into_mut(),
            Vacant(entry) => entry.insert(inventory(account).cloned().unwrap_or_default()),
        };

        if let Some(posting_costs) = interpolated.cost.as_ref() {
            for (cur, cost) in posting_costs.iter() {
                previous_positions.accumulate(
                    interpolated.units,
                    interpolated.currency.clone(),
                    Some((cur.clone(), cost.clone()).into()),
                    account_method,
                );
            }
        } else {
            previous_positions.accumulate(
                interpolated.units,
                interpolated.currency.clone(),
                None,
                account_method,
            );
        }
    }
    Ok(updated_inventory.into())
}
