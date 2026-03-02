use std::fmt::Debug;

use super::{
    AnnotatedPosting, BookedOrUnbookedPosting, BookingError, BookingTypes, CostSpec, Interpolated,
    Number, PostingBookingError, PostingCost, PostingCosts, PostingSpec, Price, PriceSpec,
    Tolerance, TransactionBookingError, tolerance_residual,
};

#[derive(Debug)]
pub(crate) struct Interpolation<B, P>
where
    B: BookingTypes,
    P: PostingSpec<Types = B>,
{
    pub(crate) booked_and_unbooked_postings: Vec<(
        Interpolated<B, P>,
        bool, // booked
    )>,

    pub(crate) residual: Option<B::Number>,
}

pub(crate) fn interpolate_from_costed<'a, 'b, B, P, T>(
    date: B::Date,
    currency: &B::Currency,
    costeds: Vec<BookedOrUnbookedPosting<B, P>>,
    tolerance: T,
) -> Result<Interpolation<B, P>, BookingError>
where
    B: BookingTypes + 'a,
    P: PostingSpec<Types = B> + Debug + 'a,
    T: Tolerance<Types = B> + Copy,
{
    let mut weights = costeds.iter().map(|c| c.weight()).collect::<Vec<_>>();
    let mut residual = tolerance_residual(tolerance, weights.iter().filter_map(|w| *w), currency);

    let unknown = weights
        .iter()
        .enumerate()
        .filter(|w| w.1.is_none())
        .collect::<Vec<_>>();

    if unknown.len() == 1 {
        let i_unknown = unknown[0].0;
        weights[i_unknown] = Some(-residual.unwrap_or_default());
        residual = None;
    } else if unknown.len() > 1 {
        return Err(BookingError::Transaction(
            TransactionBookingError::TooManyMissingNumbers,
        ));
    }

    let booked_and_unbooked_postings = costeds
        .into_iter()
        .zip(weights)
        .map(|(c, w)| match c {
            BookedOrUnbookedPosting::Unbooked(annotated) => {
                interpolate_from_annotated(date, currency, w.unwrap(), annotated, tolerance)
            }

            BookedOrUnbookedPosting::Booked(i) => Ok((i, true)),
        })
        .collect::<Result<Vec<_>, BookingError>>()?;

    Ok(Interpolation {
        booked_and_unbooked_postings,
        residual,
    })
}

pub(crate) fn interpolate_from_annotated<'a, 'b, B, P, T>(
    date: B::Date,
    currency: &B::Currency,
    weight: B::Number,
    annotated: AnnotatedPosting<P, B::Currency>,
    tolerance: T,
) -> Result<
    (
        Interpolated<B, P>,
        bool, // booked
    ),
    BookingError,
>
where
    B: BookingTypes + 'a,
    P: PostingSpec<Types = B> + Debug + 'a,
    T: Tolerance<Types = B>,
{
    match (
        units(
            &annotated.posting,
            weight,
            currency,
            annotated.currency.as_ref(),
            tolerance,
        ),
        annotated.currency,
        annotated.posting.cost(),
        annotated.posting.price(),
    ) {
        (_, _, None, None) => {
            // simple case with no cost or price
            Ok((
                Interpolated {
                    posting: annotated.posting,
                    idx: annotated.idx,
                    units: weight,
                    currency: currency.clone(),
                    cost: None,
                    price: None,
                },
                false,
            ))
        }
        (
            Some(UnitsAndConversion {
                units,
                conversion: per_unit,
            }),
            Some(currency),
            Some(cost),
            _,
        ) => {
            match (annotated.cost_currency, per_unit) {
                (Some(cost_currency), Some(conversion)) => Ok((
                    Interpolated {
                        posting: annotated.posting,
                        idx: annotated.idx,
                        units,
                        currency,
                        cost: Some(PostingCosts {
                            cost_currency,
                            adjustments: vec![PostingCost {
                                date: cost.date().unwrap_or(date),
                                units,
                                per_unit: conversion.per_unit,
                                total: conversion.total,
                                label: cost.label(),
                                merge: cost.merge(),
                            }],
                        }),
                        price: None, // ignored in favour of cost
                    },
                    false,
                )),
                (None, Some(_)) => Err(BookingError::Posting(
                    annotated.idx,
                    PostingBookingError::CannotInferCurrency,
                )),
                (Some(_), None) => Err(BookingError::Posting(
                    annotated.idx,
                    PostingBookingError::CannotInferUnits,
                )),
                (None, None) => Err(BookingError::Posting(
                    annotated.idx,
                    PostingBookingError::CannotInferAnything,
                )),
            }
        }

        (Some(UnitsAndConversion { units, conversion }), Some(currency), None, Some(_price)) => {
            // price without cost
            match (conversion, annotated.price_currency) {
                (Some(conversion), Some(price_currency)) => Ok((
                    Interpolated {
                        posting: annotated.posting,
                        idx: annotated.idx,
                        units,
                        currency,
                        cost: None,
                        price: Some(Price {
                            per_unit: conversion.per_unit,
                            total: Some(conversion.total),
                            currency: price_currency,
                        }),
                    },
                    false,
                )),
                (None, Some(_)) => Err(BookingError::Posting(
                    annotated.idx,
                    PostingBookingError::CannotInferPricePerUnit,
                )),
                (Some(_), None) => Err(BookingError::Posting(
                    annotated.idx,
                    PostingBookingError::CannotInferPriceCurrency,
                )),
                (None, None) => Err(BookingError::Posting(
                    annotated.idx,
                    PostingBookingError::CannotInferPrice,
                )),
            }
        }

        (None, Some(_), _, _) => Err(BookingError::Posting(
            annotated.idx,
            PostingBookingError::CannotInferUnits,
        )),
        (Some(_), None, _, _) => Err(BookingError::Posting(
            annotated.idx,
            PostingBookingError::CannotInferCurrency,
        )),
        (None, None, _, _) => Err(BookingError::Posting(
            annotated.idx,
            PostingBookingError::CannotInferAnything,
        )),
    }
}

#[derive(Clone, Debug)]
struct UnitsAndConversion<N> {
    units: N,
    conversion: Option<Conversion<N>>,
}

#[derive(Clone, Debug)]
struct Conversion<N> {
    per_unit: N,
    total: N,
}

// infer the units once we know the weight
fn units<B, P, T>(
    posting: &P,
    weight: B::Number,
    currency: &B::Currency,
    annotated_currency: Option<&B::Currency>,
    tolerance: T,
) -> Option<UnitsAndConversion<B::Number>>
where
    B: BookingTypes,
    P: PostingSpec<Types = B>,
    T: Tolerance<Types = B>,
{
    tracing::debug!(
        "units, currency {}, annotated currency {:?}, weight {}, posting {:?}",
        currency,
        annotated_currency,
        weight,
        posting
    );
    if let Some(cost_spec) = posting.cost() {
        units_from_cost_spec(posting.units(), weight, &cost_spec, tolerance)
    } else if let Some(price_spec) = posting.price() {
        units_from_price_spec(posting.units(), weight, &price_spec, tolerance)
    } else {
        posting.units().map(|units| UnitsAndConversion {
            units,
            conversion: None,
        })
    }
}

fn units_from_cost_spec<B, CS, T>(
    posting_units: Option<B::Number>,
    weight: B::Number,
    cost_spec: &CS,
    tolerance: T,
) -> Option<UnitsAndConversion<B::Number>>
where
    B: BookingTypes,
    CS: CostSpec<Types = B> + Debug,
    T: Tolerance<Types = B>,
{
    tracing::debug!(
        "units_from_cost_spec weight {}, posting-units {:?}, cost-per-unit {:?}, cost-total {:?}",
        weight,
        posting_units,
        cost_spec.per_unit(),
        cost_spec.total()
    );
    match (posting_units, cost_spec.per_unit(), cost_spec.total()) {
        (Some(units), Some(per_unit), total) => {
            let total = total.unwrap_or(units * per_unit);
            Some(UnitsAndConversion {
                units,
                conversion: Some(Conversion { per_unit, total }),
            })
        }
        (None, Some(per_unit), total) => {
            if let Some(units) = weight.checked_div(per_unit) {
                let units = units.rescaled(weight.scale());
                let total = total.unwrap_or(units * per_unit);
                Some(UnitsAndConversion {
                    units,
                    conversion: Some(Conversion { per_unit, total }),
                })
            } else {
                None
            }
        }
        (Some(units), None, Some(cost_total)) => {
            infer_per_unit::<B, T>(cost_total, units, tolerance)
        }
        (Some(units), None, None) => infer_per_unit::<B, T>(weight, units, tolerance),
        (None, None, _) => None,
    }
}

fn units_from_price_spec<B, PS, T>(
    posting_units: Option<B::Number>,
    weight: B::Number,
    price_spec: &PS,
    tolerance: T,
) -> Option<UnitsAndConversion<B::Number>>
where
    B: BookingTypes,
    PS: PriceSpec<Types = B> + Debug,
    T: Tolerance<Types = B>,
{
    match (posting_units, price_spec.per_unit(), price_spec.total()) {
        (Some(units), Some(per_unit), total) => {
            let total = total.unwrap_or(units * per_unit);
            Some(UnitsAndConversion {
                units,
                conversion: Some(Conversion { per_unit, total }),
            })
        }
        (None, Some(per_unit), total) => {
            if let Some(units) = weight.checked_div(per_unit) {
                let units = units.rescaled(weight.scale());
                let total = total.unwrap_or(units * per_unit);
                Some(UnitsAndConversion {
                    units,
                    conversion: Some(Conversion { per_unit, total }),
                })
            } else {
                None
            }
        }
        (Some(units), None, Some(price_total)) => {
            infer_per_unit::<B, T>(price_total, units, tolerance)
        }
        (Some(units), None, None) => infer_per_unit::<B, T>(weight, units, tolerance),
        (None, None, _) => None,
    }
}

fn infer_per_unit<B, T>(
    total: B::Number,
    units: B::Number,
    _tolerance: T,
) -> Option<UnitsAndConversion<B::Number>>
where
    B: BookingTypes,
    T: Tolerance<Types = B>,
{
    let per_unit = total.checked_div(units).map(|per_unit| per_unit.abs());
    // TODO scale according to tolerance
    Some(UnitsAndConversion {
        units,
        conversion: per_unit.map(|per_unit| Conversion { per_unit, total }),
    })
}
