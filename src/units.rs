//! Definitions of the different units that may be used to express `Duration`s. In essence, these
//! types are little more than labels that are associated with a given ratio to SI seconds, as may
//! be used to convert between arbitrary time periods.

/// Trait representing the fact that something is a unit ratio.
pub trait UnitRatio {
    const ATTOSECONDS: i128;
}

/// Unit that is described as an exact ratio with respect to unity.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LiteralRatio<const ATTOSECONDS: i128> {}

impl<const ATTOSECONDS: i128> UnitRatio for LiteralRatio<ATTOSECONDS> {
    const ATTOSECONDS: i128 = ATTOSECONDS;
}

// SI time unit qualifiers
pub type Atto = LiteralRatio<1>;
pub type Femto = LiteralRatio<1_000>;
pub type Pico = LiteralRatio<1_000_000>;
pub type Nano = LiteralRatio<1_000_000_000>;
pub type Micro = LiteralRatio<1_000_000_000_000>;
pub type Milli = LiteralRatio<1_000_000_000_000_000>;
pub type Second = LiteralRatio<1_000_000_000_000_000_000>;
pub type SecondsPerMinute = LiteralRatio<{ 1_000_000_000_000_000_000 * 60 }>;
pub type SecondsPerHour = LiteralRatio<{ 1_000_000_000_000_000_000 * 3600 }>;
pub type SecondsPerDay = LiteralRatio<{ 1_000_000_000_000_000_000 * 3600 * 24 }>;
pub type SecondsPerWeek = LiteralRatio<{ 1_000_000_000_000_000_000 * 3600 * 24 * 7 }>;
/// The number of seconds in 1/12 the average Gregorian year.
pub type SecondsPerMonth = LiteralRatio<{ 1_000_000_000_000_000_000 * 2629746 }>;
/// The number of seconds in an average Gregorian year.
pub type SecondsPerYear = LiteralRatio<{ 1_000_000_000_000_000_000 * 31556952 }>;
