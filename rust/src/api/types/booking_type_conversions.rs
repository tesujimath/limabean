use super::booked::*;
use beancount_parser_lima as parser;

impl<'a>
    From<(
        parser::Currency<'a>,
        &'_ limabean_booking::PostingCost<limabean_booking::LimaParserBookingTypes<'a>>,
    )> for Cost<'a>
{
    fn from(
        value: (
            parser::Currency<'a>,
            &'_ limabean_booking::PostingCost<limabean_booking::LimaParserBookingTypes<'a>>,
        ),
    ) -> Self {
        Cost {
            date: value.1.date,
            per_unit: value.1.per_unit,
            total: value.1.total,
            cur: value.0.into(),
            label: value.1.label.clone(),
            merge: value.1.merge,
        }
    }
}

impl<'a> From<&'_ limabean_booking::Price<limabean_booking::LimaParserBookingTypes<'a>>>
    for Price<'a>
{
    fn from(
        value: &'_ limabean_booking::Price<limabean_booking::LimaParserBookingTypes<'a>>,
    ) -> Self {
        Price {
            per_unit: value.per_unit,
            total: value.total,
            cur: value.currency,
        }
    }
}
