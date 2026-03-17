use beancount_parser_lima as parser;

pub(crate) fn book<'a>(
    directives: &[Directive<'a>],
    options: &parser::Options<'a>,
    // plugins: &[parser::Plugin<'a>],
) -> Result<LoadSuccess<'a>, LoadError> {
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

    // let plugins = match collate_plugins(plugins) {
    //     Ok(plugins) => plugins,
    //     Err(errors) => {
    //         todo!("errors from plugins")
    //     }
    // };

    let tolerance = options.into();

    Loader::new(
        default_booking_option,
        &tolerance, // TODO , &plugins.internal
    )
    .collect(directives)
}

mod loader;

mod types;

use crate::api::types::raw::Directive;
pub(crate) use crate::api::{
    booking::loader::{LoadError, LoadSuccess, Loader},
    plugins::collate_plugins,
    types::booked,
};
