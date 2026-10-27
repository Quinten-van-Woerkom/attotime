//! Implementation of parsing functionality. Largely separate from the actual time logic itself, so
//! kept in a separate module for isolation.
//!
//! Primarily, a subset of ISO 8601 is supported.

mod duration;
mod gregorian_date;
mod historic_date;
mod julian_date;
pub use duration::*;
mod time_of_day;
pub use time_of_day::*;
mod time_point;

use crate::errors::IntegerParsingError;

/// Parses an integer from the start of the given bytes, returning it together with the number of
/// bytes consumed.
fn parse_partial<N: lexical_core::FromLexical>(
    bytes: &[u8],
) -> Result<(N, usize), IntegerParsingError> {
    lexical_core::parse_partial(bytes).map_err(IntegerParsingError)
}
