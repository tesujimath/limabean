use beancount_parser_lima as parser;

pub(crate) fn book<'a, 'r, 'b>(
    directives: &'r [Directive<'a>],
    options: &parser::Options<'a>,
) -> Result<LoadSuccess<'b>, LoadError>
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

    Loader::new(default_booking_option, &tolerance).collect(directives)
}

mod loader;

mod types;

pub(crate) use crate::api::booking::loader::{LoadError, LoadSuccess, Loader};
use crate::api::types::raw::Directive;
