//! All errors that may be returned by public functions of this library are defined in this module.
//! This is useful in reducing the number of "unnecessary" inter-module dependencies, by ensuring
//! that using the results/error of a function does not require importing its entire module.

use thiserror::Error;

use crate::{Date, HistoricDate, Month};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Error)]
#[error("{day} {month} {year} does not exist in the historic calendar")]
pub struct InvalidHistoricDate {
    pub year: i32,
    pub month: Month,
    pub day: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Error)]
#[error("{day} {month} {year} does not exist in the proleptic Gregorian calendar")]
pub struct InvalidGregorianDate {
    pub year: i32,
    pub month: Month,
    pub day: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Error)]
#[error("{day} {month} {year} does not exist in the proleptic Julian calendar")]
pub struct InvalidJulianDate {
    pub year: i32,
    pub month: Month,
    pub day: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Error)]
#[error("invalid combination of year and day-of-year")]
pub enum InvalidDayOfYear {
    #[error(transparent)]
    InvalidDayOfYearCount(#[from] InvalidDayOfYearCount),
    #[error(transparent)]
    InvalidHistoricDate(#[from] InvalidHistoricDate),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Error)]
#[error("{day_of_year} is not a valid day in {year}")]
pub struct InvalidDayOfYearCount {
    pub day_of_year: u16,
    pub year: i32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Error)]
#[error("invalid month number {month}")]
pub struct InvalidMonthNumber {
    pub month: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Error)]
#[error("invalid week day number {week_day}")]
pub struct InvalidWeekDayNumber {
    pub week_day: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Error)]
#[error("invalid time-of-day {hour:02}-{minute:02}-{second:02}")]
pub struct InvalidTimeOfDay {
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Error)]
#[error("invalid historic date-time")]
pub enum InvalidHistoricDateTime<InvalidDateTime: core::error::Error> {
    #[error(transparent)]
    InvalidHistoricDate(#[from] InvalidHistoricDate),
    InvalidDateTime(#[source] InvalidDateTime),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Error)]
#[error("invalid Gregorian date-time")]
pub enum InvalidGregorianDateTime<InvalidDateTime: core::error::Error> {
    #[error(transparent)]
    InvalidGregorianDate(#[from] InvalidGregorianDate),
    InvalidDateTime(#[source] InvalidDateTime),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Error)]
#[error("invalid Julian date-time")]
pub enum InvalidJulianDateTime<InvalidDateTime: core::error::Error> {
    #[error(transparent)]
    InvalidJulianDate(#[from] InvalidJulianDate),
    InvalidDateTime(#[source] InvalidDateTime),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Error)]
pub enum InvalidUtcDateTime {
    #[error("invalid time-of-day")]
    InvalidTimeOfDay(#[from] InvalidTimeOfDay),
    #[error("not a valid UTC leap second date-time: {}T{hour:02}-{minute:02}-{second:02}", <Date as Into<HistoricDate>>::into(*date))]
    NonLeapSecondDateTime {
        date: Date,
        hour: u8,
        minute: u8,
        second: u8,
    },
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Error)]
pub enum InvalidGlonassDateTime {
    #[error("invalid time-of-day")]
    InvalidTimeOfDay(#[from] InvalidTimeOfDay),
    #[error("not a valid GLONASST leap second date-time: {}T{hour:02}-{minute:02}-{second:02}", <Date as Into<HistoricDate>>::into(*date))]
    NonLeapSecondDateTime {
        date: Date,
        hour: u8,
        minute: u8,
        second: u8,
    },
}
