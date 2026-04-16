use beancount_parser_lima as parser;

pub(crate) fn book<'a, 'r, 'b>(
    directives: &'r [Directive<'a>],
    validate: bool,
    options: &parser::Options<'a>,
) -> Result<BookingSuccess<'b>, BookingFailure>
where
    'a: 'b,
    'r: 'b,
{
    let default_booking = limabean_booking::Booking::default();
    let default_booking_option = if let Some(booking_method) = options.booking_method() {
        let booking = Into::<limabean_booking::Booking>::into(*booking_method.item());
        if limabean_booking::is_supported_method(booking) {
            booking
        } else {
            // TODO warning
            // warnings.push(booking_method.warning(format!(
            //     "Unsupported booking method, falling back to {default_booking}"
            // )));
            default_booking
        }
    } else {
        default_booking
    };

    let tolerance = options.into();

    Accumulator::new(default_booking_option, &tolerance, validate).collect(directives)
}

mod accumulator;

mod types;

pub(crate) use crate::api::booking::accumulator::{Accumulator, BookingFailure, BookingSuccess};
use crate::api::types::raw::Directive;
