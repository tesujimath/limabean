use hashbrown::{HashMap, HashSet};
use std::{fmt::Debug, iter::once};

use crate::{BookingTypes, tolerance_residual};

use super::{
    AnnotatedPosting, BookedOrUnbookedPosting, Booking, BookingError, Cost, CostSpec, Interpolated,
    Inventory, Number, Position, Positions, PostingBookingError, PostingCost, PostingCosts,
    PostingSpec, Tolerance,
};

#[derive(Debug)]
pub(crate) struct Reductions<B, P>
where
    B: BookingTypes,
    P: PostingSpec<Types = B>,
{
    pub(crate) updated_inventory: Inventory<B>,
    pub(crate) postings: Vec<BookedOrUnbookedPosting<B, P>>,
}

pub(crate) fn book_reductions<'a, B, P, T, I, M>(
    annotateds: Vec<AnnotatedPosting<P, B::Currency>>,
    tolerance: T,
    inventory: I,
    method: M,
) -> Result<Reductions<B, P>, BookingError>
where
    B: BookingTypes + 'a,
    P: PostingSpec<Types = B> + Debug + 'a,
    T: Tolerance<Types = B> + Copy,
    I: Fn(B::Account) -> Option<&'a Positions<B>> + Copy,
    M: Fn(B::Account) -> Booking + Copy,
{
    let mut updated_inventory = HashMap::default();
    let mut costed_postings = Vec::default();

    for annotated in annotateds {
        let account = annotated.posting.account();
        let previous_positions = updated_inventory
            .get(&account)
            .or_else(|| inventory(account.clone()));
        let account_method = method(account.clone());

        let Reduced {
            reducing_posting: costed_posting,
            updated_positions,
        } = reduce(annotated, tolerance, account_method, previous_positions)?;

        costed_postings.push(costed_posting);
        if let Some(updated_positions) = updated_positions {
            updated_inventory.insert(account, updated_positions);
        }
    }

    Ok(Reductions {
        updated_inventory: updated_inventory.into(),
        postings: costed_postings,
    })
}

struct Reduced<B, P>
where
    B: BookingTypes,
    P: PostingSpec<Types = B>,
{
    reducing_posting: BookedOrUnbookedPosting<B, P>,
    updated_positions: Option<Positions<B>>,
}

fn reduce<'a, B, P, T>(
    annotated: AnnotatedPosting<P, B::Currency>,
    tolerance: T,
    method: Booking,
    previous_positions: Option<&Positions<B>>,
) -> Result<Reduced<B, P>, BookingError>
where
    B: BookingTypes + 'a,
    P: PostingSpec<Types = B> + Debug + 'a,
    T: Tolerance<Types = B>,
{
    use BookedOrUnbookedPosting::*;

    if method != Booking::None
        && let (Some(posting_currency), Some(posting_units), Some(_posting_cost), Some(positions)) = (
            &annotated.currency,
            annotated.posting.units(),
            annotated.posting.cost(),
            previous_positions,
        )
        && is_potential_reduction(posting_units, posting_currency, positions)
    {
        // find positions whose costs match what we have
        let matched = match_positions(
            posting_currency,
            annotated.posting.cost().as_ref(),
            positions,
        );

        if matched.is_empty() {
            Err(BookingError::Posting(
                annotated.idx,
                PostingBookingError::NoPositionMatches,
            ))
        } else if matched.len() == 1 {
            let (reducing_posting, updated_positions) = reduce_matched_position(
                posting_units,
                posting_currency,
                annotated.posting,
                annotated.idx,
                positions,
                matched[0],
            )?;

            Ok(Reduced {
                reducing_posting,
                updated_positions: Some(updated_positions),
            })
        } else if is_sell_all_at_cost(
            posting_units,
            posting_currency,
            positions,
            &matched,
            tolerance,
        ) {
            let (reducing_posting, updated_positions) = reduce_all_sold_at_cost(
                posting_units,
                posting_currency,
                annotated.posting,
                annotated.idx,
                positions,
                matched,
            )?;

            Ok(Reduced {
                reducing_posting,
                updated_positions: Some(updated_positions),
            })
        } else {
            let (reducing_posting, updated_positions) = reduce_multiple_positions(
                posting_units,
                posting_currency,
                annotated.posting,
                annotated.idx,
                positions,
                matched,
                method,
            )?;

            Ok(Reduced {
                reducing_posting,
                updated_positions: Some(updated_positions),
            })
        }
    } else {
        Ok(Reduced {
            reducing_posting: Unbooked(annotated),
            updated_positions: None,
        })
    }
}

// do any positions in this currency have a sign opposite to ours?
fn is_potential_reduction<B>(
    posting_units: B::Number,
    posting_currency: &B::Currency,
    previous_positions: &Positions<B>,
) -> bool
where
    B: BookingTypes,
{
    if let Some(ann_sign) = posting_units.sign()
        && previous_positions
            .iter()
            .filter(|pos| &pos.currency == posting_currency)
            .any(|pos| {
                pos.units
                    .sign()
                    .is_some_and(|pos_sign| pos_sign != ann_sign)
            })
    {
        true
    } else {
        false
    }
}

fn reduce_matched_position<'a, B, P>(
    posting_units: B::Number,
    posting_currency: &B::Currency,
    posting: P,
    posting_idx: usize,
    previous_positions: &Positions<B>,
    matched_position_idx: usize,
) -> Result<(BookedOrUnbookedPosting<B, P>, Positions<B>), BookingError>
where
    B: BookingTypes + 'a,
    P: PostingSpec<Types = B> + Debug + 'a,
{
    use BookedOrUnbookedPosting::*;

    let Position {
        currency: _matched_currency,
        units: matched_units,
        cost: matched_cost,
    } = &previous_positions[matched_position_idx];

    if posting_units.abs() > matched_units.abs() {
        Err(BookingError::Posting(
            posting_idx,
            PostingBookingError::NotEnoughLotsToReduce,
        ))
    } else {
        // Book 'em, Danno!
        let matched_cost = matched_cost.as_ref().unwrap();
        let updated_positions = Positions::from_previous(
            previous_positions
                .iter()
                .enumerate()
                .filter_map(|(i, pos)| {
                    if i == matched_position_idx {
                        let updated_pos = pos.with_accumulated(posting_units);
                        (updated_pos.units != Number::zero()).then_some(updated_pos)
                    } else {
                        Some(pos.clone())
                    }
                })
                .collect::<Vec<_>>(),
        );

        Ok((
            Booked(Interpolated {
                posting,
                idx: posting_idx,
                units: posting_units,
                currency: posting_currency.clone(),
                cost: Some(PostingCosts {
                    cost_currency: matched_cost.currency.clone(),
                    adjustments: vec![PostingCost {
                        date: matched_cost.date,
                        units: posting_units,
                        per_unit: matched_cost.per_unit,
                        total: matched_cost.total,
                        label: matched_cost.label.as_ref().cloned(),
                        merge: matched_cost.merge,
                    }],
                }),
                price: None, // ignored in favour of cost
            }),
            updated_positions,
        ))
    }
}

// is this "sell everything that matches"?
// that is, matched positions together with this one sum to zero-ish updated_inventory
fn is_sell_all_at_cost<B, T>(
    posting_units: B::Number,
    posting_currency: &B::Currency,
    positions: &Positions<B>,
    matched: &[usize],
    tolerance: T,
) -> bool
where
    B: BookingTypes,
    T: Tolerance<Types = B>,
{
    let tol = tolerance_residual(
        tolerance,
        matched
            .iter()
            .map(|i| positions[*i].units)
            .chain(once(posting_units)),
        posting_currency,
    );
    tol.is_none()
}

fn reduce_multiple_positions<'a, B, P>(
    posting_units: B::Number,
    posting_currency: &B::Currency,
    posting: P,
    posting_idx: usize,
    positions: &Positions<B>,
    mut matched: Vec<usize>,
    method: Booking,
) -> Result<(BookedOrUnbookedPosting<B, P>, Positions<B>), BookingError>
where
    B: BookingTypes + 'a,
    P: PostingSpec<Types = B> + Debug + 'a,
{
    match method {
        Booking::Fifo | Booking::Lifo | Booking::Hifo => {
            check_sufficient_matched_units(posting_units, posting_idx, positions, &matched)?;
            let cost_currency = get_unique_cost_currency(posting_idx, positions, &matched)?;

            // all that differs between the [FLH]ifo methods is the order in which we select matched postings for reduction
            if method == Booking::Lifo {
                matched.reverse();
            } else if method == Booking::Hifo {
                // sort by cost per-unit, greater first
                matched.sort_by(|i, j| {
                    positions[*j]
                        .cost
                        .as_ref()
                        .unwrap()
                        .per_unit
                        .cmp(&positions[*i].cost.as_ref().unwrap().per_unit)
                });
            }

            reduce_ordered_positions(
                posting_units,
                posting_currency.clone(),
                cost_currency,
                posting,
                posting_idx,
                positions,
                &matched,
            )
        }

        Booking::StrictWithSize => {
            // not only do we filter to positions which match the posting units, but we take the oldest by cost date
            let mut matched_with_size = matched
                .into_iter()
                .filter(|i| positions[*i].units == -posting_units)
                .collect::<Vec<_>>();
            matched_with_size.sort_by(|i, j| {
                positions[*i]
                    .cost
                    .as_ref()
                    .unwrap()
                    .date
                    .cmp(&positions[*j].cost.as_ref().unwrap().date)
            });

            if !matched_with_size.is_empty() {
                reduce_matched_position(
                    posting_units,
                    posting_currency,
                    posting,
                    posting_idx,
                    positions,
                    matched_with_size[0],
                )
            } else {
                Err(BookingError::Posting(
                    posting_idx,
                    PostingBookingError::AmbiguousMatches,
                ))
            }
        }

        _ => Err(BookingError::Posting(
            posting_idx,
            PostingBookingError::AmbiguousMatches,
        )),
    }
}

fn reduce_ordered_positions<'a, B, P>(
    posting_units: B::Number,
    posting_currency: B::Currency,
    cost_currency: B::Currency,
    posting: P,
    posting_idx: usize,
    positions: &Positions<B>,
    matched: &[usize],
) -> Result<(BookedOrUnbookedPosting<B, P>, Positions<B>), BookingError>
where
    B: BookingTypes + 'a,
    P: PostingSpec<Types = B> + Debug + 'a,
{
    use BookedOrUnbookedPosting::*;

    let mut remaining_units = posting_units;
    let mut updated_position_units = positions.iter().map(|p| p.units).collect::<Vec<_>>();
    let mut adjustments = Vec::default();

    for i in matched {
        let cost_i = positions[*i].cost.as_ref().unwrap();
        let consumed = if remaining_units.abs() <= updated_position_units[*i].abs() {
            remaining_units
        } else {
            -updated_position_units[*i]
        };

        updated_position_units[*i] += consumed;
        remaining_units -= consumed;

        adjustments.push(PostingCost {
            date: cost_i.date,
            units: consumed,
            per_unit: cost_i.per_unit,
            total: cost_i.total,
            label: cost_i.label.as_ref().cloned(),
            merge: cost_i.merge,
        });

        if remaining_units == B::Number::zero() {
            break;
        }
    }

    if remaining_units != B::Number::zero() {
        return Err(BookingError::Posting(
            posting_idx,
            PostingBookingError::NotEnoughLotsToReduce,
        ));
    }

    let updated_positions = Positions::from_previous(
        updated_position_units
            .into_iter()
            .enumerate()
            .filter_map(|(i, units)| {
                let position = &positions[i];
                (units != B::Number::zero()).then_some(Position {
                    currency: posting_currency.clone(),
                    units,
                    cost: position.cost.clone(),
                })
            })
            .collect::<Vec<_>>(),
    );

    Ok((
        Booked(Interpolated {
            posting,
            idx: posting_idx,
            units: posting_units,
            currency: posting_currency,
            cost: Some(PostingCosts {
                cost_currency,
                adjustments,
            }),
            price: None, // ignored in favour of cost
        }),
        updated_positions,
    ))
}

fn check_sufficient_matched_units<B>(
    posting_units: B::Number,
    posting_idx: usize,
    positions: &Positions<B>,
    matched: &[usize],
) -> Result<(), BookingError>
where
    B: BookingTypes,
{
    let total_matched_units: B::Number = matched.iter().map(|i| positions[*i].units).sum();

    if posting_units <= total_matched_units {
        Ok(())
    } else {
        Err(BookingError::Posting(
            posting_idx,
            PostingBookingError::NotEnoughLotsToReduce,
        ))
    }
}

fn reduce_all_sold_at_cost<'a, B, P>(
    posting_units: B::Number,
    posting_currency: &B::Currency,
    posting: P,
    posting_idx: usize,
    positions: &Positions<B>,
    matched: Vec<usize>,
) -> Result<(BookedOrUnbookedPosting<B, P>, Positions<B>), BookingError>
where
    B: BookingTypes + 'a,
    P: PostingSpec<Types = B> + Debug + 'a,
{
    use BookedOrUnbookedPosting::*;

    let cost_currency = get_unique_cost_currency(posting_idx, positions, &matched)?;

    let matched_set = matched.iter().copied().collect::<HashSet<_>>();

    let updated_positions = Positions::from_previous(
        positions
            .iter()
            .enumerate()
            .filter_map(|(i, pos)| (!matched_set.contains(&i)).then_some(pos.clone()))
            .collect::<Vec<_>>(),
    );
    let adjustments = matched
        .iter()
        .map(|i| {
            let matched_position = &positions[*i];
            let matched_cost = matched_position.cost.clone().unwrap();
            PostingCost {
                date: matched_cost.date,
                units: -matched_position.units,
                per_unit: matched_cost.per_unit,
                total: matched_cost.total,
                label: matched_cost.label,
                merge: matched_cost.merge,
            }
        })
        .collect::<Vec<_>>();

    Ok((
        Booked(Interpolated {
            posting,
            idx: posting_idx,
            units: posting_units,
            currency: posting_currency.clone(),
            cost: Some(PostingCosts {
                cost_currency,
                adjustments,
            }),
            price: None, // ignored in favour of cost
        }),
        updated_positions,
    ))
}

fn get_unique_cost_currency<B>(
    posting_idx: usize,
    positions: &Positions<B>,
    matched: &[usize],
) -> Result<B::Currency, BookingError>
where
    B: BookingTypes,
{
    let cost_currencies = matched
        .iter()
        .map(|i| positions[*i].cost.as_ref().unwrap().currency.clone())
        .collect::<HashSet<_>>();

    if cost_currencies.len() == 1 {
        let cost_currency = cost_currencies.into_iter().next().unwrap();
        Ok(cost_currency)
    } else {
        Err(BookingError::Posting(
            posting_idx,
            PostingBookingError::MultipleCostCurrenciesMatch,
        ))
    }
}

fn match_positions<B, CS>(
    posting_currency: &B::Currency,
    cost_spec: Option<&CS>,
    positions: &Positions<B>,
) -> Vec<usize>
where
    B: BookingTypes,
    CS: CostSpec<Types = B> + Debug,
{
    positions
        .iter()
        .enumerate()
        .filter_map(|(i, pos)| {
            if &pos.currency != posting_currency {
                None
            } else {
                match (pos.cost.as_ref(), cost_spec) {
                    (Some(pos_cost), Some(cost_spec)) => {
                        cost_matches_spec(pos_cost, cost_spec).then_some(i)
                    }
                    _ => None,
                }
            }
        })
        .collect::<Vec<_>>()
}

fn cost_matches_spec<B, CS>(cost: &Cost<B>, cost_spec: &CS) -> bool
where
    B: BookingTypes,
    CS: CostSpec<Types = B>,
{
    !(
        cost_spec.date().is_some_and(|date| date != cost.date)
            || cost_spec
                .currency()
                .is_some_and(|cost_spec_currency| cost_spec_currency != cost.currency)
            || cost_spec
                .per_unit()
                .is_some_and(|cost_spec_units| cost_spec_units != cost.per_unit)
            || cost_spec
                .currency()
                .is_some_and(|cost_spec_currency| cost_spec_currency != cost.currency)
            || cost_spec
                .label()
                .is_some_and(|cost_spec_label| cost.label != Some(cost_spec_label))
        // TODO merge
    )
}
