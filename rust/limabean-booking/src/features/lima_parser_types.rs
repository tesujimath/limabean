use super::{Booking, BookingTypes, CostSpec, PostingSpec, PriceSpec, Tolerance, ToleranceNumber};
use beancount_parser_lima as parser;
use rust_decimal::Decimal;
use time::Date;

impl<'a> BookingTypes for &'a parser::Spanned<parser::Posting<'a>> {
    type Account = &'a str;
    type Date = time::Date;
    type Currency = parser::Currency<'a>;
    type Number = Decimal;
    type Label = &'a str;
}

pub type LimaParserBookingTypes<'a> = &'a parser::Spanned<parser::Posting<'a>>;

impl<'a> PostingSpec for &'a parser::Spanned<parser::Posting<'a>> {
    type Types = LimaParserBookingTypes<'a>;
    type CostSpec = &'a parser::CostSpec<'a>;
    type PriceSpec = &'a parser::PriceSpec<'a>;

    fn account(&self) -> &'a str {
        parser::Posting::account(self).item().as_ref()
    }

    fn currency(&self) -> Option<parser::Currency<'a>> {
        parser::Posting::currency(self).map(|cur| *cur.item())
    }

    fn units(&self) -> Option<Decimal> {
        parser::Posting::amount(self).map(|amount| amount.item().value())
    }

    fn cost(&self) -> Option<Self::CostSpec> {
        self.cost_spec().as_ref().map(|cost_spec| cost_spec.item())
    }

    fn price(&self) -> Option<Self::PriceSpec> {
        self.price_annotation()
            .as_ref()
            .map(|cost_spec| cost_spec.item())
    }
}

impl<'a> BookingTypes for &'a parser::CostSpec<'a> {
    type Account = &'a str;
    type Date = time::Date;
    type Currency = parser::Currency<'a>;
    type Number = Decimal;
    type Label = &'a str;
}

impl<'a> CostSpec for &'a parser::CostSpec<'a> {
    type Types = LimaParserBookingTypes<'a>;

    fn currency(&self) -> Option<parser::Currency<'a>> {
        parser::CostSpec::currency(self).map(|currency| *currency.item())
    }

    fn per_unit(&self) -> Option<Decimal> {
        parser::CostSpec::per_unit(self).map(|per_unit| per_unit.value())
    }

    fn total(&self) -> Option<Decimal> {
        parser::CostSpec::total(self).map(|total| total.value())
    }

    fn date(&self) -> Option<Date> {
        parser::CostSpec::date(self).map(|date| *date.item())
    }

    fn label(&self) -> Option<&'a str> {
        parser::CostSpec::label(self).map(|label| label.item().as_ref())
    }

    fn merge(&self) -> bool {
        parser::CostSpec::merge(self)
    }
}

impl<'a> BookingTypes for &'a parser::PriceSpec<'a> {
    type Account = &'a str;
    type Date = time::Date;
    type Currency = parser::Currency<'a>;
    type Number = Decimal;
    type Label = &'a str;
}

impl<'a> PriceSpec for &'a parser::PriceSpec<'a> {
    type Types = LimaParserBookingTypes<'a>;

    fn currency(&self) -> Option<parser::Currency<'a>> {
        use parser::PriceSpec::*;

        match self {
            BareCurrency(currency) => Some(*currency),
            CurrencyAmount(_, currency) => Some(*currency),
            _ => None,
        }
    }

    fn per_unit(&self) -> Option<Decimal> {
        use parser::PriceSpec::*;
        use parser::ScopedExprValue::*;

        match self {
            BareAmount(PerUnit(expr)) => Some(expr.value()),
            CurrencyAmount(PerUnit(expr), _) => Some(expr.value()),
            _ => None,
        }
    }

    fn total(&self) -> Option<Decimal> {
        use parser::PriceSpec::*;
        use parser::ScopedExprValue::*;

        match self {
            BareAmount(Total(expr)) => Some(expr.value()),
            CurrencyAmount(Total(expr), _) => Some(expr.value()),
            _ => None,
        }
    }
}

struct SumWithMinNonZeroScale {
    sum: Decimal,
    min_nonzero_scale: Option<u32>,
}

impl FromIterator<Decimal> for SumWithMinNonZeroScale {
    fn from_iter<T: IntoIterator<Item = Decimal>>(iter: T) -> Self {
        let mut sum = Decimal::ZERO;
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

impl<'a> BookingTypes for &parser::Options<'a> {
    type Account = &'a str;
    type Date = time::Date;
    type Currency = parser::Currency<'a>;
    type Number = Decimal;
    type Label = &'a str;
}

impl<'a> Tolerance for &parser::Options<'a> {
    type Types = LimaParserBookingTypes<'a>;

    // Beancount Precision & Tolerances
    // https://docs.google.com/document/d/1lgHxUUEY-UVEgoF6cupz2f_7v7vEF7fiJyiSlYYlhOo
    fn residual(
        &self,
        values: impl Iterator<Item = ToleranceNumber<Self>>,
        cur: &<Self::Types as BookingTypes>::Currency,
    ) -> Option<<Self::Types as BookingTypes>::Number> {
        // TODO don't iterate twice over values
        let values = values.collect::<Vec<_>>();
        let values = values.into_iter();

        let multiplier = self
            .inferred_tolerance_multiplier()
            .map(|m| *m.item())
            .unwrap_or(default_inferred_tolerance_multiplier());
        let s = values.collect::<SumWithMinNonZeroScale>();
        let residual = s.sum;
        let abs_residual = residual.abs();

        if let Some(min_nonzero_scale) = s.min_nonzero_scale.as_ref() {
            (abs_residual >= Decimal::new(1, *min_nonzero_scale) * multiplier).then_some(residual)
        } else {
            // TODO should we have kept currency as a parser::Currency all along, to avoid extra validation here??
            let cur = TryInto::<parser::Currency>::try_into(*cur).unwrap();
            let tolerance = self
                .inferred_tolerance_default(&cur)
                .or(self.inferred_tolerance_default_fallback());

            if let Some(tolerance) = tolerance {
                (abs_residual > tolerance).then_some(residual)
            } else {
                (!residual.is_zero()).then_some(residual)
            }
        }
    }
}

impl From<parser::Booking> for Booking {
    fn from(value: parser::Booking) -> Self {
        use Booking::*;
        use parser::Booking as parser;

        match value {
            parser::Strict => Strict,
            parser::StrictWithSize => StrictWithSize,
            parser::None => None,
            parser::Average => Average,
            parser::Fifo => Fifo,
            parser::Lifo => Lifo,
            parser::Hifo => Hifo,
        }
    }
}

// TODO where should default_inferred_tolerance_multiplier be defined?
// (we can't depend on the main limabean crate here)
fn default_inferred_tolerance_multiplier() -> Decimal {
    Decimal::new(5, 1) // 0.5
}
