use std::fmt::Debug;

use super::{
    AnnotatedPosting, BookedOrUnbookedPosting, BookingError, BookingTypes, CostSpec, Interpolated,
    Number, PostingBookingError, PostingCost, PostingCosts, PostingSpec, Price, PriceSpec,
    Tolerance, TransactionBookingError,
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
    tolerance: &T,
) -> Result<Interpolation<B, P>, BookingError>
where
    B: BookingTypes + 'a,
    P: PostingSpec<Types = B> + Debug + 'a,
    T: Tolerance<Types = B>,
{
    let mut weights = costeds.iter().map(|c| c.weight()).collect::<Vec<_>>();
    let mut residual = tolerance.residual(weights.iter().filter_map(|w| *w), currency);

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
                interpolate_from_annotated(date, currency, w.unwrap(), annotated)
            }

            BookedOrUnbookedPosting::Booked(i) => Ok((i, true)),
        })
        .collect::<Result<Vec<_>, BookingError>>()?;

    Ok(Interpolation {
        booked_and_unbooked_postings,
        residual,
    })
}

pub(crate) fn interpolate_from_annotated<'a, 'b, B, P>(
    date: B::Date,
    currency: &B::Currency,
    weight: B::Number,
    annotated: AnnotatedPosting<P, B::Currency>,
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
{
    match (
        units(&annotated.posting, weight),
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
        (Some(UnitsAndPerUnit { units, per_unit }), Some(currency), Some(cost), _) => {
            match (annotated.cost_currency, per_unit) {
                (Some(cost_currency), Some(per_unit)) => Ok((
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
                                per_unit,
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

        (Some(UnitsAndPerUnit { units, per_unit }), Some(currency), None, Some(_price)) => {
            // price without cost
            match (per_unit, annotated.price_currency) {
                (Some(per_unit), Some(price_currency)) => Ok((
                    Interpolated {
                        posting: annotated.posting,
                        idx: annotated.idx,
                        units,
                        currency,
                        cost: None,
                        price: Some(Price {
                            per_unit,
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
struct UnitsAndPerUnit<N> {
    units: N,
    per_unit: Option<N>,
}

// infer the units once we know the weight
fn units<B, P>(posting: &P, weight: B::Number) -> Option<UnitsAndPerUnit<B::Number>>
where
    B: BookingTypes,
    P: PostingSpec<Types = B>,
{
    // TODO review unit inference from cost and price and weight
    if let Some(cost_spec) = posting.cost() {
        units_from_cost_spec(posting.units(), weight, &cost_spec)
    } else if let Some(price_spec) = posting.price() {
        units_from_price_spec(posting.units(), weight, &price_spec)
    } else {
        posting.units().map(|units| UnitsAndPerUnit {
            units,
            per_unit: None,
        })
    }
}

fn units_from_cost_spec<B, CS>(
    posting_units: Option<B::Number>,
    weight: B::Number,
    cost_spec: &CS,
) -> Option<UnitsAndPerUnit<B::Number>>
where
    B: BookingTypes,
    CS: CostSpec<Types = B> + Debug,
{
    match (posting_units, cost_spec.per_unit(), cost_spec.total()) {
        (Some(units), Some(per_unit), _) => Some(UnitsAndPerUnit {
            units,
            per_unit: Some(per_unit),
        }),
        (None, Some(per_unit), _) => {
            let units = (weight / per_unit).rescaled(weight.scale());
            Some(UnitsAndPerUnit {
                units,
                per_unit: Some(per_unit),
            })
        }
        (Some(units), None, Some(cost_total)) => {
            let per_unit = cost_total / units;
            Some(UnitsAndPerUnit {
                units,
                per_unit: Some(per_unit),
            })
        }
        (Some(units), None, None) => Some(UnitsAndPerUnit {
            units,
            per_unit: None,
        }),
        (None, None, _) => None, // TODO is this correct?
    }
}

fn units_from_price_spec<B, PS>(
    posting_units: Option<B::Number>,
    weight: B::Number,
    price_spec: &PS,
) -> Option<UnitsAndPerUnit<B::Number>>
where
    B: BookingTypes,
    PS: PriceSpec<Types = B> + Debug,
{
    match (posting_units, price_spec.per_unit(), price_spec.total()) {
        (Some(units), Some(per_unit), _) => Some(UnitsAndPerUnit {
            units,
            per_unit: Some(per_unit),
        }),
        (None, Some(per_unit), _) => {
            let units = (weight / per_unit).rescaled(weight.scale());
            Some(UnitsAndPerUnit {
                units,
                per_unit: Some(per_unit),
            })
        }
        (Some(units), None, Some(total)) => {
            let per_unit = total / units;
            Some(UnitsAndPerUnit {
                units,
                per_unit: Some(per_unit),
            })
        }
        (Some(units), None, None) => {
            let per_unit = weight / units;
            Some(UnitsAndPerUnit {
                units,
                per_unit: Some(per_unit),
            })
        }
        (None, None, _) => None,
    }
}
