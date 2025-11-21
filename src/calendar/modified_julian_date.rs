//! The Modified Julian Day (MJD) is nothing more than the number of days since 1858 November 17
//! at 0h UT. Effectively, this makes it a constant offset from the Julian Day (JD); however, the
//! MJD is useful because it is not fractional for time points at midnight.

use crate::{
    Date, Days, Month,
    errors::{InvalidGregorianDate, InvalidHistoricDate, InvalidJulianDate},
};

/// The Modified Julian Day (MJD) representation of any given date.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModifiedJulianDate {
    time_since_epoch: Days,
}

/// The modified Julian date of the Unix epoch is useful as constant in some calculations.
const MODIFIED_JULIAN_DATE_UNIX_EPOCH: Days = Days::new(40587);

impl ModifiedJulianDate {
    /// Constructs a new MJD directly from some duration since the MJD epoch, November 17 1858.
    pub const fn from_time_since_epoch(time_since_epoch: Days) -> Self {
        Self { time_since_epoch }
    }

    /// Returns the time since the MJD epoch of this day.
    pub const fn time_since_epoch(&self) -> Days {
        self.time_since_epoch
    }

    /// Constructs a modified Julian date from some given calendar date.
    pub fn from_date(date: Date) -> Self {
        Self {
            time_since_epoch: date.time_since_epoch() + MODIFIED_JULIAN_DATE_UNIX_EPOCH,
        }
    }

    /// Converts this modified Julian date into the equivalent "universal" calendar date.
    pub fn into_date(&self) -> Date {
        Date::from_time_since_epoch(self.time_since_epoch - MODIFIED_JULIAN_DATE_UNIX_EPOCH)
    }

    /// Creates a `Date` based on a year-month-day date in the historic calendar.
    pub fn from_historic_date(
        year: i32,
        month: Month,
        day: u8,
    ) -> Result<Self, InvalidHistoricDate> {
        match Date::from_historic_date(year, month, day) {
            Ok(date) => Ok(Self::from_date(date)),
            Err(error) => Err(error),
        }
    }

    /// Creates a `Date` based on a year-month-day date in the proleptic Gregorian calendar.
    pub fn from_gregorian_date(
        year: i32,
        month: Month,
        day: u8,
    ) -> Result<Self, InvalidGregorianDate> {
        match Date::from_gregorian_date(year, month, day) {
            Ok(date) => Ok(Self::from_date(date)),
            Err(error) => Err(error),
        }
    }

    /// Creates a `Date` based on a year-month-day date in the proleptic Julian calendar.
    pub fn from_julian_date(year: i32, month: Month, day: u8) -> Result<Self, InvalidJulianDate> {
        match Date::from_julian_date(year, month, day) {
            Ok(date) => Ok(Self::from_date(date)),
            Err(error) => Err(error),
        }
    }
}

impl From<Date> for ModifiedJulianDate {
    fn from(value: Date) -> Self {
        Self::from_date(value)
    }
}

impl From<ModifiedJulianDate> for Date {
    fn from(value: ModifiedJulianDate) -> Self {
        value.into_date()
    }
}

/// Verifies this implementation by computing the `ModifiedJulianDate` for some known (computed
/// manually or obtained elsewhere) time stamp. If it doesn't match the given `time_since_epoch`,
/// panics.
#[cfg(test)]
fn check_historic_modified_julian_date(year: i32, month: Month, day: u8, time_since_epoch: Days) {
    assert_eq!(
        ModifiedJulianDate::from_historic_date(year, month, day)
            .unwrap()
            .time_since_epoch(),
        time_since_epoch,
    );
}

/// Compares some computed MJD values with known values from Meeus' Astronomical Algorithms.
/// Includes all historic dates, including those from before the Gregorian reform: indeed, the
/// historic date structure should be able to capture that.
#[test]
fn historic_dates_from_meeus() {
    use crate::Month::*;
    check_historic_modified_julian_date(2000, January, 1, Days::new(51544));
    check_historic_modified_julian_date(1999, January, 1, Days::new(51179));
    check_historic_modified_julian_date(1987, January, 27, Days::new(46822));
    check_historic_modified_julian_date(1987, June, 19, Days::new(46965));
    check_historic_modified_julian_date(1988, January, 27, Days::new(47187));
    check_historic_modified_julian_date(1988, June, 19, Days::new(47331));
    check_historic_modified_julian_date(1900, January, 1, Days::new(15020));
    check_historic_modified_julian_date(1600, January, 1, Days::new(-94553));
    check_historic_modified_julian_date(1600, December, 31, Days::new(-94188));
    check_historic_modified_julian_date(837, April, 10, Days::new(-373129));
    check_historic_modified_julian_date(-123, December, 31, Days::new(-723504));
    check_historic_modified_julian_date(-122, January, 1, Days::new(-723503));
    check_historic_modified_julian_date(-1000, July, 12, Days::new(-1044000));
    check_historic_modified_julian_date(-1000, February, 29, Days::new(-1044134));
    check_historic_modified_julian_date(-1001, August, 17, Days::new(-1044330));
    check_historic_modified_julian_date(-4712, January, 1, Days::new(-2400001));
}

#[cfg(kani)]
mod proof_harness {
    use super::*;

    /// Verifies that construction of a MJD based on a date never panics, assuming that the input
    /// date is between validity bounds based on `i32` limits.
    #[kani::proof]
    fn from_date_never_panics() {
        let date: Date = kani::any();
        kani::assume(
            date.time_since_epoch().count() <= i32::MAX - MODIFIED_JULIAN_DATE_UNIX_EPOCH.count(),
        );
        let _ = ModifiedJulianDate::from_date(date);
    }
}
