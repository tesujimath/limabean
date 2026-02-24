mod book;
#[cfg(test)]
pub(crate) use book::book_with_residuals;
pub use book::{book, is_supported_method};

mod categorize;
pub(crate) use categorize::categorize_by_currency;

mod errors;
pub use errors::{BookingError, PostingBookingError, TransactionBookingError};

mod features;

mod interpolate;
pub(crate) use interpolate::{Interpolation, interpolate_from_costed};

mod internal_types;
pub(crate) use internal_types::*;

mod public_types;
pub use public_types::{
    Booking, BookingTypes, Bookings, Cost, CostSpec, Interpolated, Inventory, Number, Position,
    Positions, Posting, PostingCost, PostingCosts, PostingSpec, Price, PriceSpec, Sign, Tolerance,
};

mod reductions;
pub(crate) use reductions::{Reductions, book_reductions};

#[cfg(test)]
mod tests;
