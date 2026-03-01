#![doc = include_str!("../README.md")]

mod book;
#[cfg(test)]
pub(crate) use book::book_with_residuals;
pub use book::{book, is_supported_method};

mod categorize;
pub(crate) use categorize::{CategorizedByCurrency, categorize_by_currency};

mod errors;
pub use errors::{BookingError, PostingBookingError, TransactionBookingError};

mod features;
#[cfg(feature = "lima-parser-types")]
pub use features::LimaParserBookingTypes;

mod interpolate;
pub(crate) use interpolate::{Interpolation, interpolate_from_costed};

mod internal_types;
pub(crate) use internal_types::*;

mod public_types;
pub use public_types::{
    Booking, BookingTypes, Bookings, Cost, CostSpec, CostSpecCurrency, CostSpecDate, CostSpecLabel,
    CostSpecNumber, Interpolated, Inventory, Number, Position, Positions, PostingCost,
    PostingCosts, PostingSpec, PostingSpecAccount, PostingSpecCurrency, PostingSpecNumber, Price,
    PriceSpec, PriceSpecCurrency, PriceSpecNumber, Sign, Tolerance, ToleranceCurrency,
    ToleranceNumber,
};

mod reductions;
pub(crate) use reductions::{Reductions, book_reductions};

mod tolerance;
pub(crate) use tolerance::tolerance_residual;

#[cfg(test)]
mod tests;
