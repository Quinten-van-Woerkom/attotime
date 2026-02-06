//! Implementation of the "generic" date representation, which is largely agnostic of any specific
//! calendar representation: including phenomena such as months, weeks, years, and leap years.
//! Rather, it is a simple day count since the Unix epoch.

use core::ops::{Add, AddAssign, Sub, SubAssign};

use crate::{
    GregorianDate, HistoricDate, JulianDate, Month, WeekDay,
    calendar::Days,
    errors::{InvalidGregorianDate, InvalidHistoricDate, InvalidJulianDate},
};

/// Generic date representation
///
/// Generic representation of date. Identifies an exact individual date within the calendar, in
/// terms of days before (negative) or after (positive) 1970-01-01. This makes it useful as
/// universal type that can be converted to and from other calendrical types.
///
/// Note that this type is not associated with a time zone: rather, it represents the local time in
/// some implicit time zone.
///
/// It is explicitly not possible to subtract one `Date` from another to obtain a duration. This
/// choice is made to prevent errors due to leap seconds, which cannot be incorporated in a
/// purely calendrical type. Rather, a date must be mapped towards a proper time scale first,
/// before such arithmetic is possible. It is possible to add full days to a `Date`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, derive_more::Constructor)]
#[cfg_attr(kani, derive(kani::Arbitrary))]
pub struct Date {
    days: Days,
}

impl Date {
    /// Creates a date from the given number of days since 1970-01-01.
    #[must_use]
    pub const fn from_time_since_epoch(days: Days) -> Self {
        Self { days }
    }

    #[must_use]
    /// The number of days since the epoch of this representation - midnight 1970.
    pub const fn time_since_epoch(&self) -> Days {
        self.days
    }

    /// Returns the number of elapsed calendar days since the passed date. Beware: the returned
    /// value represents strictly the number of elapsed calendar (!) days. While it is expressed as
    /// a duration, the possibility of leap seconds is ignored. Only interpret the returned value
    /// as an exact duration if no leap seconds occurred between both days.
    #[must_use]
    pub fn elapsed_calendar_days_since(self, other: Self) -> Days {
        self.days - other.days
    }
}

impl Date {
    /// Creates a `Date` based on a year-month-day date in the historic calendar.
    ///
    /// # Errors
    /// Will raise an error if the provided combination of year, month, and day is not a valid day
    /// in the historic calendar.
    pub const fn from_historic_date(
        year: i32,
        month: Month,
        day: u8,
    ) -> Result<Self, InvalidHistoricDate> {
        match HistoricDate::new(year, month, day) {
            Ok(historic_date) => Ok(historic_date.into_date()),
            Err(error) => Err(error),
        }
    }

    /// Creates a `Date` based on a year-month-day date in the proleptic Gregorian calendar.
    ///
    /// # Errors
    /// Will raise an error if the provided combination of year, month, and day is not a valid day
    /// in the proleptic Gregorian calendar.
    pub const fn from_gregorian_date(
        year: i32,
        month: Month,
        day: u8,
    ) -> Result<Self, InvalidGregorianDate> {
        match GregorianDate::new(year, month, day) {
            Ok(gregorian_date) => Ok(gregorian_date.into_date()),
            Err(error) => Err(error),
        }
    }

    /// Creates a `Date` based on a year-month-day date in the proleptic Julian calendar.
    ///
    /// # Errors
    /// Will raise an error if the provided combination of year, month, and day is not a valid day
    /// in the proleptic Julian calendar.
    pub const fn from_julian_date(
        year: i32,
        month: Month,
        day: u8,
    ) -> Result<Self, InvalidJulianDate> {
        match JulianDate::new(year, month, day) {
            Ok(julian_date) => Ok(julian_date.into_date()),
            Err(error) => Err(error),
        }
    }

    /// Returns the day-of-the-week of this date.
    #[must_use]
    pub const fn week_day(&self) -> WeekDay {
        let z = self.time_since_epoch().count();
        let day = if z >= 0 { z % 7 } else { (z + 1) % 7 + 6 };
        match day {
            0 => WeekDay::Thursday,
            1 => WeekDay::Friday,
            2 => WeekDay::Saturday,
            3 => WeekDay::Sunday,
            4 => WeekDay::Monday,
            5 => WeekDay::Tuesday,
            6 => WeekDay::Wednesday,
            _ => unreachable!(),
        }
    }
}

impl Add<Days> for Date {
    type Output = Self;

    fn add(self, rhs: Days) -> Self {
        Self {
            days: self.days + rhs,
        }
    }
}

impl AddAssign<Days> for Date
where
    Days: AddAssign,
{
    fn add_assign(&mut self, rhs: Days) {
        self.days += rhs;
    }
}

impl Sub<Days> for Date {
    type Output = Self;

    fn sub(self, rhs: Days) -> Self {
        Self {
            days: self.days - rhs,
        }
    }
}

impl SubAssign<Days> for Date
where
    Days: SubAssign,
{
    fn sub_assign(&mut self, rhs: Days) {
        self.days -= rhs;
    }
}

/// Verifies that the epoch of `Date` is found at 1970-01-01 (historic calendar).
#[test]
fn epoch_at_1970_01_01() {
    let epoch = Date::from_historic_date(1970, Month::January, 1).unwrap();
    assert_eq!(epoch.time_since_epoch(), Days::new(0));

    let historic_date = HistoricDate::new(1970, Month::January, 1).unwrap();
    let historic_date2 = HistoricDate::from_date(epoch);
    assert_eq!(historic_date, historic_date2);
}

/// Testing function that simply verifies whether a given historic date corresponds with a provided
/// week day. If not, panics.
#[cfg(test)]
fn check_week_day(year: i32, month: Month, day: u8, week_day: crate::WeekDay) {
    assert_eq!(
        Date::from_historic_date(year, month, day)
            .unwrap()
            .week_day(),
        week_day
    );
}

/// Tests some known week day values.
#[test]
fn week_days() {
    check_week_day(1969, Month::December, 25, WeekDay::Thursday);
    check_week_day(1969, Month::December, 26, WeekDay::Friday);
    check_week_day(1969, Month::December, 27, WeekDay::Saturday);
    check_week_day(1969, Month::December, 28, WeekDay::Sunday);
    check_week_day(1969, Month::December, 29, WeekDay::Monday);
    check_week_day(1969, Month::December, 30, WeekDay::Tuesday);
    check_week_day(1969, Month::December, 31, WeekDay::Wednesday);
    check_week_day(1970, Month::January, 1, WeekDay::Thursday);
    check_week_day(1970, Month::January, 2, WeekDay::Friday);
    check_week_day(1970, Month::January, 3, WeekDay::Saturday);
    check_week_day(1970, Month::January, 4, WeekDay::Sunday);
    check_week_day(1970, Month::January, 5, WeekDay::Monday);
    check_week_day(1970, Month::January, 6, WeekDay::Tuesday);
    check_week_day(1970, Month::January, 7, WeekDay::Wednesday);
    check_week_day(1970, Month::January, 8, WeekDay::Thursday);
    check_week_day(1998, Month::December, 17, WeekDay::Thursday);
}

#[cfg(kani)]
mod infallibility {
    use super::*;

    #[kani::proof]
    fn week_day() {
        let date: Date = kani::any();
        let _week_day = date.week_day();
    }

    #[kani::proof]
    fn historic_date_roundtrip() {
        let date: Date = kani::any();
        let historic_date = HistoricDate::from_date(date);
        let year = historic_date.year();
        let month = historic_date.month();
        let day = historic_date.day();
        let date2 = Date::from_historic_date(year, month, day).unwrap();
        assert_eq!(date, date2);
    }

    #[kani::proof]
    fn gregorian_date_roundtrip() {
        let date: Date = kani::any();
        let gregorian_date = GregorianDate::from_date(date);
        let year = gregorian_date.year();
        let month = gregorian_date.month();
        let day = gregorian_date.day();
        let date2 = Date::from_gregorian_date(year, month, day).unwrap();
        assert_eq!(date, date2);
    }

    #[kani::proof]
    fn julian_date_roundtrip() {
        let date: Date = kani::any();
        let julian_date = JulianDate::from_date(date);
        let year = julian_date.year();
        let month = julian_date.month();
        let day = julian_date.day();
        let date2 = Date::from_julian_date(year, month, day).unwrap();
        assert_eq!(date, date2);
    }
}
