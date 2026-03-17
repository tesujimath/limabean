use limabean_booking::{Booking, CostSpec, LimaParserBookingTypes, PostingSpec, PriceSpec};
use rust_decimal::Decimal;
use time::Date;

use crate::api::types::raw;

impl<'a> PostingSpec for raw::PostingSpec<'a> {
    type Types = LimaParserBookingTypes<'a>;
    type CostSpec = raw::CostSpec<'a>;
    type PriceSpec = raw::PriceSpec<'a>;

    fn account(&self) -> &'a str {
        self.acc
    }

    fn currency(&self) -> Option<&'a str> {
        self.cur
    }

    fn units(&self) -> Option<Decimal> {
        self.units
    }

    fn cost(&self) -> Option<&Self::CostSpec> {
        self.cost_spec.as_ref()
    }

    fn price(&self) -> Option<&Self::PriceSpec> {
        self.price_spec.as_ref()
    }
}

impl<'a> CostSpec for raw::CostSpec<'a> {
    type Types = LimaParserBookingTypes<'a>;

    fn currency(&self) -> Option<&'a str> {
        self.cur
    }

    fn per_unit(&self) -> Option<Decimal> {
        self.per_unit
    }

    fn total(&self) -> Option<Decimal> {
        self.total
    }

    fn date(&self) -> Option<Date> {
        self.date
    }

    fn label(&self) -> Option<&'a str> {
        self.label
    }

    fn merge(&self) -> bool {
        self.merge
    }
}

impl<'a> PriceSpec for raw::PriceSpec<'a> {
    type Types = LimaParserBookingTypes<'a>;

    fn currency(&self) -> Option<&'a str> {
        self.cur
    }

    fn per_unit(&self) -> Option<Decimal> {
        self.per_unit
    }

    fn total(&self) -> Option<Decimal> {
        self.total
    }
}

impl From<raw::Booking> for Booking {
    fn from(value: raw::Booking) -> Self {
        use Booking::*;
        use raw::Booking as raw;

        match value {
            raw::Strict => Strict,
            raw::StrictWithSize => StrictWithSize,
            raw::None => None,
            raw::Average => Average,
            raw::Fifo => Fifo,
            raw::Lifo => Lifo,
            raw::Hifo => Hifo,
        }
    }
}
