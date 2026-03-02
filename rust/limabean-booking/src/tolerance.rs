use crate::{BookingTypes, Number, Tolerance};

// Beancount Precision & Tolerances
// https://docs.google.com/document/d/1lgHxUUEY-UVEgoF6cupz2f_7v7vEF7fiJyiSlYYlhOo
pub(crate) fn tolerance_residual<B, T>(
    tol: T,
    values: impl Iterator<Item = B::Number>,
    cur: &B::Currency,
) -> Option<B::Number>
where
    B: BookingTypes,
    T: Tolerance<Types = B>,
{
    // TODO don't iterate twice over values
    let values = values.collect::<Vec<_>>();
    let values = values.into_iter();

    let multiplier = tol
        .inferred_tolerance_multiplier()
        .unwrap_or(default_inferred_tolerance_multiplier::<B>());
    let s = values.collect::<SumWithMinNonZeroScale<B>>();
    let residual = s.sum;
    let abs_residual = residual.abs();

    if let Some(min_nonzero_scale) = s.min_nonzero_scale.as_ref() {
        (abs_residual >= B::Number::new(1, *min_nonzero_scale) * multiplier).then_some(residual)
    } else {
        let tolerance = tol.inferred_tolerance_default(cur);

        if let Some(tolerance) = tolerance {
            (abs_residual > tolerance).then_some(residual)
        } else {
            (residual != B::Number::zero()).then_some(residual)
        }
    }
}

#[derive(Clone, Debug)]
struct SumWithMinNonZeroScale<B>
where
    B: BookingTypes,
{
    sum: B::Number,
    min_nonzero_scale: Option<u32>,
}

impl<B> FromIterator<B::Number> for SumWithMinNonZeroScale<B>
where
    B: BookingTypes,
{
    fn from_iter<T: IntoIterator<Item = B::Number>>(iter: T) -> Self {
        let mut sum = B::Number::zero();
        let mut min_nonzero_scale = None;
        for value in iter {
            sum += value;
            if value.scale() > 0 {
                if min_nonzero_scale.is_none() {
                    min_nonzero_scale = Some(value.scale());
                } else if let Some(scale) = min_nonzero_scale
                    && value.scale() < scale
                {
                    min_nonzero_scale = Some(value.scale());
                }
            }
        }

        Self {
            sum,
            min_nonzero_scale,
        }
    }
}

fn default_inferred_tolerance_multiplier<B>() -> B::Number
where
    B: BookingTypes,
{
    B::Number::new(5, 1) // 0.5
}
