//! Definition of the `TimePoint` type (and associated types and methods), which implements the
//! fundamental timekeeping logic of this library.

use core::{
    fmt::{Debug, Display},
    hash::Hash,
    ops::{Add, AddAssign, Sub, SubAssign},
};

use num_traits::{Bounded, Zero};

use crate::{
    Date, Days, Duration, FromDateTime, FromFineDateTime, GregorianDate, HistoricDate,
    IntoDateTime, IntoFineDateTime, JulianDate, ModifiedJulianDate, Month, Second, UnitRatio,
    errors::{InvalidGregorianDateTime, InvalidHistoricDateTime, InvalidJulianDateTime},
    time_scale::{AbsoluteTimeScale, TimeScale, UniformDateTimeScale},
};

/// Instant in time
///
/// A `TimePoint` identifies a specific instant in time. It is templated on a `Representation` and
/// `Period`, which the define the characteristics of the `Duration` type used to represent the
/// time elapsed since the epoch of the underlying time scale `Scale`.
pub struct TimePoint<Scale: ?Sized> {
    time_since_epoch: Duration,
    time_scale: core::marker::PhantomData<Scale>,
}

impl<Scale: ?Sized> TimePoint<Scale> {
    /// Constructs a new `TimePoint` from a known time since epoch.
    #[must_use]
    pub const fn from_time_since_epoch(time_since_epoch: Duration) -> Self {
        Self {
            time_since_epoch,
            time_scale: core::marker::PhantomData,
        }
    }

    /// Returns the time elapsed since the epoch of the time scale associated with this instant.
    #[must_use]
    pub const fn time_since_epoch(&self) -> Duration {
        self.time_since_epoch
    }

    /// Returns the raw underlying representation of this time point.
    #[must_use]
    pub const fn count(&self) -> i128 {
        self.time_since_epoch().count()
    }

    /// Converts towards a different time unit, rounding towards the nearest whole unit.
    #[must_use]
    pub const fn round<Target>(self) -> Self
    where
        Target: UnitRatio,
    {
        Self::from_time_since_epoch(self.time_since_epoch.round::<Target>())
    }

    /// Converts towards a different time unit, rounding towards positive infinity if the unit is
    /// not entirely commensurate with the present unit.
    #[must_use]
    pub fn ceil<Target>(self) -> Self
    where
        Target: UnitRatio,
    {
        Self::from_time_since_epoch(self.time_since_epoch.ceil::<Target>())
    }

    /// Converts towards a different time unit, rounding towards negative infinity if the unit is
    /// not entirely commensurate with the present unit.
    #[must_use]
    pub fn floor<Target>(self) -> Self
    where
        Target: UnitRatio,
    {
        Self::from_time_since_epoch(self.time_since_epoch.floor::<Target>())
    }

    /// Constructs a `TimePoint` in the given time scale, based on a historic date-time.
    ///
    /// # Errors
    /// Will raise an error if the requested combination of year-month-day does not exist in the
    /// historic calendar, or if the requested time-of-day does not exist for the given date.
    pub fn from_historic_datetime(
        year: i32,
        month: Month,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> Result<Self, InvalidHistoricDateTime<<Self as FromDateTime>::Error>>
    where
        Self: FromDateTime,
    {
        let date = Date::from_historic_date(year, month, day)?;
        match Self::from_datetime(date, hour, minute, second) {
            Ok(time_point) => Ok(time_point),
            Err(error) => Err(InvalidHistoricDateTime::InvalidDateTime(error)),
        }
    }

    /// Constructs a `TimePoint` in the given time scale, based on a Gregorian date-time.
    ///
    /// # Errors
    /// Will raise an error if the requested combination of year-month-day does not exist in the
    /// proleptic Gregorian calendar, or if the requested time-of-day does not exist for the given
    /// date.
    pub fn from_gregorian_datetime(
        year: i32,
        month: Month,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> Result<Self, InvalidGregorianDateTime<<Self as FromDateTime>::Error>>
    where
        Self: FromDateTime,
    {
        let date = Date::from_gregorian_date(year, month, day)?;
        match Self::from_datetime(date, hour, minute, second) {
            Ok(time_point) => Ok(time_point),
            Err(error) => Err(InvalidGregorianDateTime::InvalidDateTime(error)),
        }
    }

    /// Constructs a `TimePoint` in the given time scale, based on a Julian date-time.
    ///
    /// # Errors
    /// Will raise an error if the requested combination of year-month-day does not exist in the
    /// proleptic Julian calendar, or if the requested time-of-day does not exist for the given
    /// date.
    pub fn from_julian_datetime(
        year: i32,
        month: Month,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> Result<Self, InvalidJulianDateTime<<Self as FromDateTime>::Error>>
    where
        Self: FromDateTime,
    {
        let date = Date::from_julian_date(year, month, day)?;
        match Self::from_datetime(date, hour, minute, second) {
            Ok(time_point) => Ok(time_point),
            Err(error) => Err(InvalidJulianDateTime::InvalidDateTime(error)),
        }
    }
}

impl<Scale> TimePoint<Scale>
where
    Scale: ?Sized + UniformDateTimeScale,
{
    /// Constructs a time point from a modified Julian date, expressed in the resulting time scale
    /// itself. The modified Julian date uses 17 November, 1858 (historic calendar) as epoch, or
    /// 2400000.5 days less than the Julian day.
    ///
    /// Conversions from modified Julian days into `TimePoint`s are supported only for uniform date
    /// time scales. For non-uniform time scales, leap second days result in ambiguous and
    /// difficult to implement interpretations of the fractional part of a day. Based on the
    /// "Resolution B1 on the use of Julian Dates" of the IAU, it is also not recommended to use
    /// such Julian date expressions: hence, we do not support it.
    #[must_use]
    pub fn from_modified_julian_date(mjd: ModifiedJulianDate) -> Self {
        const MODIFIED_JULIAN_EPOCH: Date =
            match Date::from_historic_date(1858, Month::November, 17) {
                Ok(epoch) => epoch,
                Err(_) => panic!("Internal error: start of modified Julian period found invalid"),
            };
        let epoch_julian_day = Scale::EPOCH.elapsed_calendar_days_since(MODIFIED_JULIAN_EPOCH);
        let days_since_epoch = mjd.time_since_epoch() - epoch_julian_day;
        Self::from_time_since_epoch(days_since_epoch.into())
    }
}

impl<Scale> TimePoint<Scale>
where
    Scale: ?Sized + AbsoluteTimeScale,
{
    /// Converts this time point into the equivalent Julian day representation.
    #[allow(clippy::missing_panics_doc, reason = "Infallible")]
    #[must_use]
    pub fn into_modified_julian_date(&self) -> ModifiedJulianDate {
        const MODIFIED_JULIAN_EPOCH: Date =
            match Date::from_historic_date(1858, Month::November, 17) {
                Ok(epoch) => epoch,
                Err(_) => panic!("Internal error: start of modified Julian period found invalid"),
            };
        let epoch_julian_day = Scale::EPOCH.elapsed_calendar_days_since(MODIFIED_JULIAN_EPOCH);
        let days_since_epoch = Days::new(
            (self.time_since_epoch() / Duration::days(1))
                .try_into()
                .unwrap_or_else(|_| panic!()),
        );
        let days_since_epoch = days_since_epoch + epoch_julian_day;
        ModifiedJulianDate::from_time_since_epoch(days_since_epoch)
    }
}

impl<Scale> FromFineDateTime for TimePoint<Scale>
where
    Scale: ?Sized,
    Self: FromDateTime,
{
    type Error = <Self as FromDateTime>::Error;

    fn from_fine_datetime(
        date: Date,
        hour: u8,
        minute: u8,
        second: u8,
        subseconds: Duration,
    ) -> Result<Self, Self::Error> {
        let coarse_time_point: Self = Self::from_datetime(date, hour, minute, second)?;
        Ok(coarse_time_point + subseconds)
    }
}

impl<Scale> TimePoint<Scale>
where
    Self: FromFineDateTime + FromDateTime,
    Scale: ?Sized,
{
    /// Constructs a `TimePoint` in the given time scale, based on a subsecond-accuracy historic
    /// date-time.
    ///
    /// # Errors
    /// Will raise an error if the requested combination of year-month-day does not exist in the
    /// historic calendar, or if the requested time-of-day does not exist for the given date.
    pub fn from_fine_historic_datetime(
        year: i32,
        month: Month,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        subseconds: Duration,
    ) -> Result<Self, InvalidHistoricDateTime<<Self as FromFineDateTime>::Error>> {
        let date = Date::from_historic_date(year, month, day)?;
        match Self::from_fine_datetime(date, hour, minute, second, subseconds) {
            Ok(time_point) => Ok(time_point),
            Err(error) => Err(InvalidHistoricDateTime::InvalidDateTime(error)),
        }
    }

    /// Constructs a `TimePoint` in the given time scale, based on a subsecond-accuracy Gregorian
    /// date-time.
    ///
    /// # Errors
    /// Will raise an error if the requested combination of year-month-day does not exist in the
    /// proleptic Gregorian calendar, or if the requested time-of-day does not exist for the given
    /// date.
    pub fn from_fine_gregorian_datetime(
        year: i32,
        month: Month,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        subseconds: Duration,
    ) -> Result<Self, InvalidGregorianDateTime<<Self as FromFineDateTime>::Error>> {
        let date = Date::from_gregorian_date(year, month, day)?;
        match Self::from_fine_datetime(date, hour, minute, second, subseconds) {
            Ok(time_point) => Ok(time_point),
            Err(error) => Err(InvalidGregorianDateTime::InvalidDateTime(error)),
        }
    }

    /// Constructs a `TimePoint` in the given time scale, based on a subsecond-accuracy Julian
    /// date-time.
    ///
    /// # Errors
    /// Will raise an error if the requested combination of year-month-day does not exist in the
    /// proleptic Julian calendar, or if the requested time-of-day does not exist for the given
    /// date.
    pub fn from_fine_julian_datetime(
        year: i32,
        month: Month,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        subseconds: Duration,
    ) -> Result<Self, InvalidJulianDateTime<<Self as FromFineDateTime>::Error>> {
        let date = Date::from_julian_date(year, month, day)?;
        match Self::from_fine_datetime(date, hour, minute, second, subseconds) {
            Ok(time_point) => Ok(time_point),
            Err(error) => Err(InvalidJulianDateTime::InvalidDateTime(error)),
        }
    }
}

impl<Scale> IntoFineDateTime for TimePoint<Scale>
where
    Scale: ?Sized,
    Self: IntoDateTime,
{
    fn into_fine_datetime(self) -> (Date, u8, u8, u8, Duration) {
        let coarse_time_point = self.floor::<Second>();
        let subseconds = self - coarse_time_point;
        let (date, hour, minute, second) = coarse_time_point.into_datetime();
        (date, hour, minute, second, subseconds)
    }
}

impl<Scale: ?Sized> TimePoint<Scale>
where
    Self: IntoDateTime,
{
    /// Maps a `TimePoint` towards the corresponding historic date and time-of-day.
    #[must_use]
    pub fn into_historic_datetime(self) -> (HistoricDate, u8, u8, u8) {
        let (date, hour, minute, second) = self.into_datetime();
        (date.into(), hour, minute, second)
    }

    /// Maps a `TimePoint` towards the corresponding proleptic Gregorian date and time-of-day.
    #[must_use]
    pub fn into_gregorian_datetime(self) -> (GregorianDate, u8, u8, u8) {
        let (date, hour, minute, second) = self.into_datetime();
        (date.into(), hour, minute, second)
    }

    /// Maps a `TimePoint` towards the corresponding Julian date and time-of-day.
    #[must_use]
    pub fn into_julian_datetime(self) -> (JulianDate, u8, u8, u8) {
        let (date, hour, minute, second) = self.into_datetime();
        (date.into(), hour, minute, second)
    }
}

impl<Scale: ?Sized> TimePoint<Scale>
where
    Self: IntoFineDateTime,
{
    #[must_use]
    pub fn into_fine_historic_datetime(self) -> (HistoricDate, u8, u8, u8, Duration) {
        let (date, hour, minute, second, subseconds) = self.into_fine_datetime();
        (date.into(), hour, minute, second, subseconds)
    }

    #[must_use]
    pub fn into_fine_gregorian_datetime(self) -> (GregorianDate, u8, u8, u8, Duration) {
        let (date, hour, minute, second, subseconds) = self.into_fine_datetime();
        (date.into(), hour, minute, second, subseconds)
    }

    #[must_use]
    pub fn into_fine_julian_datetime(self) -> (JulianDate, u8, u8, u8, Duration) {
        let (date, hour, minute, second, subseconds) = self.into_fine_datetime();
        (date.into(), hour, minute, second, subseconds)
    }
}

impl<Scale> Display for TimePoint<Scale>
where
    Scale: ?Sized + TimeScale,
    Duration: Zero,
    Self: IntoFineDateTime,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let (historic_date, hour, minute, second, subseconds) = self.into_fine_historic_datetime();
        write!(
            f,
            "{:04}-{:02}-{:02}T{hour:02}:{minute:02}:{second:02}",
            historic_date.year(),
            historic_date.month() as u8,
            historic_date.day(),
        )?;

        if !subseconds.is_zero() {
            write!(f, ".")?;

            // Set maximum number of digits after the decimal point printed based on precision
            // argument given to the formatter.
            let max_digits_printed = f.precision();
            for digit in subseconds.decimal_digits(max_digits_printed) {
                write!(f, "{digit}")?;
            }
        }

        write!(f, " {}", Scale::ABBREVIATION)
    }
}

#[cfg(feature = "std")]
#[cfg(test)]
#[allow(clippy::too_many_arguments)]
fn check_formatting_i64(
    string: &str,
    year: i32,
    month: Month,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    milliseconds: i128,
) {
    let time = crate::TaiTime::from_fine_historic_datetime(
        year,
        month,
        day,
        hour,
        minute,
        second,
        crate::Duration::milliseconds(milliseconds),
    )
    .unwrap();
    assert_eq!(time.to_string(), string);
}

/// Verifies formatting for some known values.
#[cfg(feature = "std")]
#[test]
fn formatting_i64() {
    use crate::Month::*;
    check_formatting_i64("1958-01-01T00:00:00.001 TAI", 1958, January, 1, 0, 0, 0, 1);
    check_formatting_i64("1958-01-02T00:00:00 TAI", 1958, January, 2, 0, 0, 0, 0);
    check_formatting_i64(
        "1960-01-01T12:34:56.789 TAI",
        1960,
        January,
        1,
        12,
        34,
        56,
        789,
    );
    check_formatting_i64("1961-01-01T00:00:00 TAI", 1961, January, 1, 0, 0, 0, 0);
    check_formatting_i64("1970-01-01T00:00:00 TAI", 1970, January, 1, 0, 0, 0, 0);
    check_formatting_i64(
        "1976-01-01T23:59:59.999 TAI",
        1976,
        January,
        1,
        23,
        59,
        59,
        999,
    );
    check_formatting_i64("2025-07-16T16:23:24 TAI", 2025, July, 16, 16, 23, 24, 0);
    check_formatting_i64(
        "2034-12-26T08:02:37.123 TAI",
        2034,
        December,
        26,
        8,
        2,
        37,
        123,
    );
    check_formatting_i64("2760-04-01T21:59:58 TAI", 2760, April, 1, 21, 59, 58, 0);
    check_formatting_i64("1643-01-04T01:01:33 TAI", 1643, January, 4, 1, 1, 33, 0);
}

/// Verifies that truncation is properly applied when the underlying fraction exceeds the number of
/// digits specified in the formatting precision (or 9 by default, if none is specified).
#[cfg(feature = "std")]
#[test]
fn truncated_format() {
    let time = crate::UtcTime::from_fine_historic_datetime(
        1998,
        Month::December,
        17,
        23,
        21,
        58,
        crate::Duration::picoseconds(450_103_789_401_i128),
    )
    .unwrap();
    assert_eq!(format!("{time:.9}"), "1998-12-17T23:21:58.450103789 UTC");
}

/// Verifies that formatting does not panic for a large randomized range of values.
#[cfg(feature = "std")]
#[test]
fn random_formatting() {
    use crate::TaiTime;
    use core::str::FromStr;
    use rand::prelude::*;
    let mut rng = rand_chacha::ChaCha12Rng::seed_from_u64(76);
    for _ in 0..10_000 {
        let ticks_since_epoch = rng.random::<i64>();
        let time_since_epoch = crate::Duration::nanoseconds(ticks_since_epoch.into());
        let time = TaiTime::from_time_since_epoch(time_since_epoch);
        let string = format!("{time:.9}");
        let time2 = TaiTime::from_str(string.as_str()).unwrap();
        assert_eq!(time, time2);
    }
}

#[cfg(kani)]
impl<Scale> kani::Arbitrary for TimePoint<Scale>
where
    Scale: ?Sized,
{
    fn any() -> Self {
        TimePoint::from_time_since_epoch(kani::any())
    }
}

impl<Scale: ?Sized> Debug for TimePoint<Scale> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TimePoint")
            .field("time_since_epoch", &self.time_since_epoch)
            .finish()
    }
}

impl<Scale: ?Sized> Copy for TimePoint<Scale> {}

impl<Scale: ?Sized> Clone for TimePoint<Scale> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Scale: ?Sized> PartialEq for TimePoint<Scale> {
    fn eq(&self, other: &Self) -> bool {
        self.time_since_epoch == other.time_since_epoch && self.time_scale == other.time_scale
    }
}

impl<Scale: ?Sized> Eq for TimePoint<Scale> {}

impl<Scale: ?Sized> PartialOrd for TimePoint<Scale> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<Scale: ?Sized> Ord for TimePoint<Scale> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        match self.time_since_epoch.cmp(&other.time_since_epoch) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.time_scale.cmp(&other.time_scale)
    }
}

impl<Scale: ?Sized> Hash for TimePoint<Scale> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.time_since_epoch.hash(state);
        self.time_scale.hash(state);
    }
}

impl<Scale> Sub for TimePoint<Scale>
where
    Scale: ?Sized,
{
    type Output = Duration;

    fn sub(self, rhs: Self) -> Self::Output {
        self.time_since_epoch - rhs.time_since_epoch
    }
}

impl<Scale> Add<Duration> for TimePoint<Scale>
where
    Scale: ?Sized,
{
    type Output = Self;

    fn add(self, rhs: Duration) -> Self::Output {
        Self::from_time_since_epoch(self.time_since_epoch + rhs)
    }
}

impl<Scale> AddAssign<Duration> for TimePoint<Scale>
where
    Scale: ?Sized,
{
    fn add_assign(&mut self, rhs: Duration) {
        self.time_since_epoch += rhs;
    }
}

impl<Scale> Sub<Duration> for TimePoint<Scale>
where
    Scale: ?Sized,
{
    type Output = Self;

    fn sub(self, rhs: Duration) -> Self::Output {
        Self::from_time_since_epoch(self.time_since_epoch - rhs)
    }
}

impl<Scale> SubAssign<Duration> for TimePoint<Scale>
where
    Scale: ?Sized,
{
    fn sub_assign(&mut self, rhs: Duration) {
        self.time_since_epoch -= rhs;
    }
}

impl<Scale> Bounded for TimePoint<Scale>
where
    Scale: ?Sized,
{
    fn min_value() -> Self {
        Self::from_time_since_epoch(Duration::min_value())
    }

    fn max_value() -> Self {
        Self::from_time_since_epoch(Duration::max_value())
    }
}
