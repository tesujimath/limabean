use std::marker::PhantomData;

use super::{Booking, BookingTypes, CostSpec, PostingSpec, PriceSpec, Tolerance};
use beancount_parser_lima as parser;
use rust_decimal::Decimal;
use time::Date;

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct LimaParserBookingTypes<'a>(PhantomData<&'a str>);

impl<'a> BookingTypes for LimaParserBookingTypes<'a> {
    type Account = &'a str;
    type Date = time::Date;
    type Currency = parser::Currency<'a>;
    type Number = Decimal;
    type Label = &'a str;
}

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

impl<'a> BookingTypes for &parser::Options<'a> {
    type Account = &'a str;
    type Date = time::Date;
    type Currency = parser::Currency<'a>;
    type Number = Decimal;
    type Label = &'a str;
}

impl<'a> Tolerance for &parser::Options<'a> {
    type Types = LimaParserBookingTypes<'a>;

    fn inferred_tolerance_default(
        &self,
        currency: &<Self::Types as BookingTypes>::Currency,
    ) -> Option<<Self::Types as BookingTypes>::Number> {
        parser::Options::inferred_tolerance_default(self, currency)
            .or_else(|| parser::Options::inferred_tolerance_default_fallback(self))
    }

    fn inferred_tolerance_multiplier(&self) -> Option<<Self::Types as BookingTypes>::Number> {
        parser::Options::inferred_tolerance_multiplier(self).map(|x| *x.item())
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
