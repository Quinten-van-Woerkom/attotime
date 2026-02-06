//! Definition of a `Days`, but expressed as a number of days. Useful in calendrical
//! calculations - opposed to the (in this context) wasteful `Days`.

use core::{
    fmt::Debug,
    ops::{Div, Mul},
};

use num_traits::{Bounded, ConstZero, Signed, Zero};

use crate::Duration;

/// Representation of a duration to an accuracy of `Days`. Useful whenever some duration is known
/// or needs to be known only to an accuracy of one day - for example, in calendrical computations.
#[derive(
    Copy,
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    derive_more::Add,
    derive_more::AddAssign,
    derive_more::Sub,
    derive_more::SubAssign,
    derive_more::Neg,
)]
#[cfg_attr(kani, derive(kani::Arbitrary))]
pub struct Days {
    count: i32,
}

impl Days {
    /// Constructs a new `Days` from a given number of days.
    #[must_use]
    pub const fn new(count: i32) -> Self {
        Self { count }
    }

    /// Constructs a new `Days` from a given number of weeks.
    #[must_use]
    pub const fn weeks(count: i32) -> Self {
        Self { count: count * 7 }
    }

    /// Returns the raw number of time units contained in this `Days`. It is advised not to
    /// use this function unless absolutely necessary, as it effectively throws away all time unit
    /// information and safety.
    #[must_use]
    pub const fn count(&self) -> i32 {
        self.count
    }

    /// Returns the `Duration` equivalent to this number of days.
    #[must_use]
    pub const fn into_duration(&self) -> Duration {
        Duration::days(self.count as i128)
    }
}

impl<T> Mul<T> for Days
where
    T: Into<i32>,
{
    type Output = Self;

    /// A `Days` may not be multiplied with another `Days` (as that is undefined), but it may
    /// be multiplied with unitless numbers.
    fn mul(self, rhs: T) -> Self::Output {
        Self::Output {
            count: self.count * rhs.into(),
        }
    }
}

impl<T> Div<T> for Days
where
    T: Into<i32>,
{
    type Output = Self;

    /// A `Days` may may be divided by unitless numbers to obtain a new `Days`.
    fn div(self, rhs: T) -> Self::Output {
        Self::Output {
            count: self.count / rhs.into(),
        }
    }
}

impl Bounded for Days {
    /// Returns the `Days` value that is nearest to negative infinity.
    fn min_value() -> Self {
        Self { count: i32::MIN }
    }

    /// Returns the `Days` value that is nearest to positive infinity.
    fn max_value() -> Self {
        Self { count: i32::MAX }
    }
}

impl Zero for Days {
    /// Returns a `Days` value that represents no time passed.
    fn zero() -> Self {
        Self { count: i32::zero() }
    }

    /// Whether this `Days` has any duration.
    fn is_zero(&self) -> bool {
        self.count.is_zero()
    }
}

impl ConstZero for Days {
    const ZERO: Self = Self { count: i32::ZERO };
}

impl Days {
    #[must_use]
    pub const fn abs(&self) -> Self {
        Self {
            count: self.count.abs(),
        }
    }

    #[must_use]
    pub fn abs_sub(&self, other: &Self) -> Self {
        Self {
            count: self.count.abs_sub(&other.count),
        }
    }

    #[must_use]
    pub const fn signum(&self) -> Self {
        Self {
            count: self.count.signum(),
        }
    }

    #[must_use]
    pub const fn is_positive(&self) -> bool {
        self.count.is_positive()
    }

    #[must_use]
    pub const fn is_negative(&self) -> bool {
        self.count.is_negative()
    }
}
