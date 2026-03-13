use limabean_booking::{Booking, BookingTypes, CostSpec, PostingSpec, PriceSpec};
use rust_decimal::Decimal;
use std::marker::PhantomData;
use time::Date;

use crate::api::types::raw as api;

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct LimabeanApiBookingTypes<'a>(PhantomData<&'a str>);

impl<'a> BookingTypes for LimabeanApiBookingTypes<'a> {
    type Account = &'a str;
    type Date = time::Date;
    type Currency = &'a str;
    type Number = Decimal;
    type Label = &'a str;
}

impl<'a> PostingSpec for &'a api::PostingSpec<'a> {
    type Types = LimabeanApiBookingTypes<'a>;
    type CostSpec = api::CostSpec<'a>;
    type PriceSpec = api::PriceSpec<'a>;

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

impl<'a> CostSpec for api::CostSpec<'a> {
    type Types = LimabeanApiBookingTypes<'a>;

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

impl<'a> PriceSpec for api::PriceSpec<'a> {
    type Types = LimabeanApiBookingTypes<'a>;

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

impl From<api::Booking> for Booking {
    fn from(value: api::Booking) -> Self {
        use Booking::*;
        use api::Booking as api;

        match value {
            api::Strict => Strict,
            api::StrictWithSize => StrictWithSize,
            api::None => None,
            api::Average => Average,
            api::Fifo => Fifo,
            api::Lifo => Lifo,
            api::Hifo => Hifo,
        }
    }
}
