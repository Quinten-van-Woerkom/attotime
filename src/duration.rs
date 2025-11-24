//! Differences between two time points may be expressed as `Duration`s. These are little more than
//! a count of some unit (or multiple units) of time elapsed between those two time points. This
//! concept is similar to that applied in the C++ `chrono` library.

use core::{
    fmt::{Debug, Display},
    ops::{Div, Mul},
};

use derive_more::*;
use num_traits::{Bounded, ConstZero, Signed, Zero};

use crate::{
    Days, Femto, FractionalDigitsIterator, Micro, Milli, Nano, Pico, Second, SecondsPerDay,
    SecondsPerHour, SecondsPerMinute, SecondsPerMonth, SecondsPerWeek, SecondsPerYear, UnitRatio,
};

/// A `Duration` represents the difference between two time points. It has an associated
/// `Representation`, which determines how the count of elapsed ticks is stored. The `Period`
/// determines the integer (!) ratio of each tick to seconds. This may be used to convert between
/// `Duration`s of differing time units.
///
/// The accuracy of a `Duration` is one attosecond. This makes for a representable range of about
/// 10 trillion years, or about 700 times the age of the universe; should be sufficient for most
/// purposes. Note that this type is explicitly intended for calculations only: when storing large
/// numbers of durations, it might be more efficient to use a more tailor-made representation.
#[derive(
    Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Add, AddAssign, Sub, SubAssign, Neg,
)]
#[cfg_attr(kani, derive(kani::Arbitrary))]
pub struct Duration {
    count: i128,
}

impl Duration {
    /// Constructs a new `Duration` from a given number of attoseconds.
    pub const fn attoseconds(count: i128) -> Self {
        Self { count }
    }

    /// Constructs a new `Duration` from a given number of femtoseconds.
    pub const fn femtoseconds(count: i128) -> Self {
        Self {
            count: count * Femto::ATTOSECONDS,
        }
    }

    /// Constructs a new `Duration` from a given number of picoseconds.
    pub const fn picoseconds(count: i128) -> Self {
        Self {
            count: count * Pico::ATTOSECONDS,
        }
    }

    /// Constructs a new `Duration` from a given number of nanoseconds.
    pub const fn nanoseconds(count: i128) -> Self {
        Self {
            count: count * Nano::ATTOSECONDS,
        }
    }

    /// Constructs a new `Duration` from a given number of microseconds.
    pub const fn microseconds(count: i128) -> Self {
        Self {
            count: count * Micro::ATTOSECONDS,
        }
    }

    /// Constructs a new `Duration` from a given number of milliseconds.
    pub const fn milliseconds(count: i128) -> Self {
        Self {
            count: count * Milli::ATTOSECONDS,
        }
    }

    /// Constructs a new `Duration` from a given number of seconds.
    pub const fn seconds(count: i128) -> Self {
        Self {
            count: count * Second::ATTOSECONDS,
        }
    }

    /// Constructs a new `Duration` from a given number of minutes.
    pub const fn minutes(count: i128) -> Self {
        Self {
            count: count * SecondsPerMinute::ATTOSECONDS,
        }
    }

    /// Constructs a new `Duration` from a given number of hours.
    pub const fn hours(count: i128) -> Self {
        Self {
            count: count * SecondsPerHour::ATTOSECONDS,
        }
    }

    /// Constructs a new `Duration` from a given number of days.
    pub const fn days(count: i128) -> Self {
        Self {
            count: count * SecondsPerDay::ATTOSECONDS,
        }
    }

    /// Constructs a new `Duration` from a given number of weeks.
    pub const fn weeks(count: i128) -> Self {
        Self {
            count: count * SecondsPerWeek::ATTOSECONDS,
        }
    }

    /// Constructs a new `Duration` from a given number of months. Expresses a month as 1/12 of an
    /// average Gregorian year.
    pub const fn months(count: i128) -> Self {
        Self {
            count: count * SecondsPerMonth::ATTOSECONDS,
        }
    }

    /// Constructs a new `Duration` from a given number of years. Uses an average Gregorian year as
    /// duration.
    pub const fn years(count: i128) -> Self {
        Self {
            count: count * SecondsPerYear::ATTOSECONDS,
        }
    }

    /// Returns the raw number of time units contained in this `Duration`. It is advised not to
    /// use this function unless absolutely necessary, as it effectively throws away all time unit
    /// information and safety.
    pub const fn count(&self) -> i128 {
        self.count
    }

    /// Returns an iterator over the fractional (sub-unit) digits of this duration. Useful as
    /// helper function when printing durations.
    pub fn fractional_digits(
        &self,
        precision: Option<usize>,
        base: u8,
    ) -> impl Iterator<Item = u8> {
        FractionalDigitsIterator::from_signed(
            self.count,
            1,
            1_000_000_000_000_000_000,
            precision,
            base,
        )
    }

    /// Returns an iterator over the fractional (sub-unit) digits of this duration, expressed in
    /// decimal. Useful as helper function when printing durations.
    pub fn decimal_digits(&self, precision: Option<usize>) -> impl Iterator<Item = u8> {
        self.fractional_digits(precision, 10)
    }

    /// Converts towards a different time unit, rounding towards the nearest whole unit.
    pub const fn round<Target>(self) -> Duration
    where
        Target: UnitRatio + ?Sized,
    {
        let unit_attoseconds = Target::ATTOSECONDS;
        let count = ((self.count + unit_attoseconds / 2) / unit_attoseconds) * unit_attoseconds;
        Self { count }
    }

    /// Converts towards a different time unit, rounding towards positive infinity if the unit is
    /// not entirely commensurate with the present unit.
    pub fn ceil<Target>(self) -> Duration
    where
        Target: UnitRatio + ?Sized,
    {
        let unit_attoseconds = Target::ATTOSECONDS;
        Self {
            count: (num_integer::div_ceil(self.count, unit_attoseconds)) * unit_attoseconds,
        }
    }

    /// Converts towards a different time unit, rounding towards negative infinity if the unit is
    /// not entirely commensurate with the present unit.
    pub fn floor<Target>(self) -> Duration
    where
        Target: UnitRatio + ?Sized,
    {
        let unit_attoseconds = Target::ATTOSECONDS;
        Self {
            count: (num_integer::div_floor(self.count, unit_attoseconds)) * unit_attoseconds,
        }
    }

    /// Converts towards a different time unit, rounding towards zero if the unit is not entirely
    /// commensurate with the present unit.
    pub const fn truncate<Target>(self) -> Duration
    where
        Target: UnitRatio + ?Sized,
    {
        let unit_attoseconds = Target::ATTOSECONDS;
        Self {
            count: (self.count / unit_attoseconds) * unit_attoseconds,
        }
    }

    /// Segments this `Duration` by factoring out the largest possible number of whole multiples of
    /// a given unit. Returns this whole number as well as the remainder.
    ///
    /// An example would be factoring out the number of whole days from some elapsed time: then,
    /// `self.factor_out()` would return a tuple of the number of whole days and the fractional
    /// day part that remains.
    pub fn factor_out<Unit>(self) -> (i128, Duration)
    where
        Unit: UnitRatio + ?Sized,
    {
        let factored = self.truncate::<Unit>();
        let remainder = self - factored;
        let factored = factored.count() / Unit::ATTOSECONDS;
        (factored, remainder)
    }

    /// Divides by an `i128`, rounding to the nearest result.
    pub const fn div_round(self, other: i128) -> Self {
        let count = (self.count + other / 2) / other;
        Self { count }
    }

    /// Converts into a float approximation of the stored duration, expressed in the desired units.
    /// For maximum numerical precision, first reduces the magnitude of the fraction by computing
    /// the integer quotient: in this manner, only the computation of the fractional part loses
    /// numerical precision.
    pub fn as_float<T: num_traits::Float + Display, Unit: UnitRatio>(self) -> T {
        let numerator = self.count;
        let denominator = Unit::ATTOSECONDS;
        let quotient = T::from(numerator / denominator).unwrap();
        let remainder = T::from(numerator % denominator).unwrap();
        let fraction = remainder / T::from(denominator).unwrap();
        quotient + fraction
    }
}

/// Verifies that approximation of equivalent float values results in the correct values. For some
/// of these values, we look for an exact match, since we know that the value may be represented
/// exactly as a float.
#[test]
fn approximate_floats() {
    let millisecond = Duration::milliseconds(1);
    let seconds = millisecond.as_float::<f64, Second>();
    assert_eq!(seconds, 0.001);

    let attosecond = Duration::attoseconds(1);
    let seconds = attosecond.as_float::<f64, Second>();
    assert_eq!(seconds, 1e-18);

    let day = Duration::days(1);
    let seconds = day.as_float::<f32, Second>();
    assert_eq!(seconds, 86400.);
    let hours = day.as_float::<f32, SecondsPerHour>();
    assert_eq!(hours, 24.);
    let weeks = day.as_float::<f64, SecondsPerWeek>();
    assert!((weeks - 1. / 7.).abs() < 0.1);

    let year = Duration::years(1);
    let days = year.as_float::<f64, SecondsPerDay>();
    assert_eq!(days, 365.2425);
    let months = year.as_float::<f64, SecondsPerMonth>();
    assert_eq!(months, 12.);
}

impl core::fmt::Display for Duration {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.is_negative() {
            write!(f, "-")?;
        }

        let (days, remainder) = self.factor_out::<SecondsPerDay>();
        let (hours, remainder) = remainder.factor_out::<SecondsPerHour>();
        let (minutes, remainder) = remainder.factor_out::<SecondsPerMinute>();
        let (seconds, remainder) = remainder.factor_out::<Second>();
        write!(f, "P")?;
        if days != 0 {
            write!(f, "{}D", days.abs())?;
        }
        write!(f, "T")?;
        if hours != 0 {
            write!(f, "{}H", hours.abs())?;
        }
        if minutes != 0 {
            write!(f, "{}M", minutes.abs())?;
        }
        if seconds != 0 || !remainder.is_zero() {
            write!(f, "{}", seconds.abs())?;
            if !remainder.is_zero() {
                write!(f, ".")?;
                // Set maximum number of digits after the decimal point printed based on precision
                // argument given to the formatter.
                let max_digits_printed = f.precision();
                for digit in remainder.decimal_digits(max_digits_printed) {
                    write!(f, "{digit}")?;
                }
            }
            write!(f, "S")?;
        }
        Ok(())
    }
}

impl From<Days> for Duration {
    fn from(value: Days) -> Self {
        value.into_duration()
    }
}

impl<T> Mul<T> for Duration
where
    T: Into<i128>,
{
    type Output = Duration;

    /// A `Duration` may not be multiplied with another `Duration` (as that is undefined), but it may
    /// be multiplied with unitless numbers.
    fn mul(self, rhs: T) -> Self::Output {
        Self::Output {
            count: self.count * rhs.into(),
        }
    }
}

impl Div<Duration> for Duration {
    type Output = i128;

    fn div(self, rhs: Duration) -> Self::Output {
        self.count / rhs.count
    }
}

impl<T> Div<T> for Duration
where
    T: Into<i128>,
{
    type Output = Duration;

    /// A `Duration` may may be divided by unitless numbers to obtain a new `Duration`.
    fn div(self, rhs: T) -> Self::Output {
        Self::Output {
            count: self.count / rhs.into(),
        }
    }
}

impl Bounded for Duration {
    /// Returns the `Duration` value that is nearest to negative infinity.
    fn min_value() -> Self {
        Self { count: i128::MIN }
    }

    /// Returns the `Duration` value that is nearest to positive infinity.
    fn max_value() -> Self {
        Self { count: i128::MAX }
    }
}

impl Zero for Duration {
    /// Returns a `Duration` value that represents no time passed.
    fn zero() -> Self {
        Self {
            count: i128::zero(),
        }
    }

    /// Whether this `Duration` has any duration.
    fn is_zero(&self) -> bool {
        self.count.is_zero()
    }
}

impl ConstZero for Duration {
    const ZERO: Self = Self { count: i128::ZERO };
}

impl Duration {
    pub fn abs(&self) -> Self {
        Self {
            count: self.count.abs(),
        }
    }

    pub fn abs_sub(&self, other: &Self) -> Self {
        Self {
            count: self.count.abs_sub(&other.count),
        }
    }

    pub fn signum(&self) -> Self {
        Self {
            count: self.count.signum(),
        }
    }

    pub fn is_positive(&self) -> bool {
        self.count.is_positive()
    }

    pub fn is_negative(&self) -> bool {
        self.count.is_negative()
    }
}
