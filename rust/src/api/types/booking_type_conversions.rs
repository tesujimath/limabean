use super::booked::*;
use beancount_parser_lima as parser;

impl<'a>
    From<(
        &'a parser::Currency<'a>,
        &limabean_booking::PostingCost<limabean_booking::LimaParserBookingTypes<'a>>,
    )> for Cost<'a>
{
    fn from(
        value: (
            &'a parser::Currency<'a>,
            &limabean_booking::PostingCost<limabean_booking::LimaParserBookingTypes<'a>>,
        ),
    ) -> Self {
        Cost {
            date: value.1.date,
            per_unit: value.1.per_unit,
            total: value.1.total,
            cur: value.0.as_ref(),
            label: value.1.label,
            merge: value.1.merge,
        }
    }
}

impl<'a> From<&'a limabean_booking::Price<limabean_booking::LimaParserBookingTypes<'a>>>
    for Price<'a>
{
    fn from(
        value: &'a limabean_booking::Price<limabean_booking::LimaParserBookingTypes<'a>>,
    ) -> Self {
        Price {
            per_unit: value.per_unit,
            total: value.total,
            cur: value.currency.as_ref(),
        }
    }
}
