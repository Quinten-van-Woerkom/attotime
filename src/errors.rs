//! Error types
//!
//! All errors that may be returned by public functions of this library are defined in this module.
//! This is useful in reducing the number of "unnecessary" inter-module dependencies, by ensuring
//! that using the results/error of a function does not require importing its entire module.

use thiserror::Error;

use crate::{Date, DurationDesignator, HistoricDate, Month};

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
pub enum InvalidHistoricDateTime<InvalidDateTime> {
    #[error(transparent)]
    InvalidHistoricDate(#[from] InvalidHistoricDate),
    InvalidDateTime(#[source] InvalidDateTime),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Error)]
#[error("invalid Gregorian date-time")]
pub enum InvalidGregorianDateTime<InvalidDateTime> {
    #[error(transparent)]
    InvalidGregorianDate(#[from] InvalidGregorianDate),
    InvalidDateTime(#[source] InvalidDateTime),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Error)]
#[error("invalid Julian date-time")]
pub enum InvalidJulianDateTime<InvalidDateTime> {
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
#[error("error parsing `HistoricDate`")]
pub enum HistoricDateParsingError {
    #[error(transparent)]
    IntegerParsingError(#[from] lexical_core::Error),
    #[error(transparent)]
    InvalidHistoricDate(#[from] InvalidHistoricDate),
    #[error(transparent)]
    InvalidMonthNumber(#[from] InvalidMonthNumber),
    #[error("expected but did not find year-month delimiter '-'")]
    ExpectedYearMonthDelimiter,
    #[error("month representation must be exactly two digits")]
    MonthRepresentationNotTwoDigits,
    #[error("expected but did not find month-day delimiter '-'")]
    ExpectedMonthDayDelimiter,
    #[error("day representation must be exactly two digits")]
    DayRepresentationNotTwoDigits,
    #[error("could not parse entire string: data remains after historic date")]
    UnexpectedRemainder,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
#[error("error parsing `GregorianDate`")]
pub enum GregorianDateParsingError {
    #[error(transparent)]
    IntegerParsingError(#[from] lexical_core::Error),
    #[error(transparent)]
    InvalidGregorianDate(#[from] InvalidGregorianDate),
    #[error(transparent)]
    InvalidMonthNumber(#[from] InvalidMonthNumber),
    #[error("expected but did not find year-month delimiter '-'")]
    ExpectedYearMonthDelimiter,
    #[error("month representation must be exactly two digits")]
    MonthRepresentationNotTwoDigits,
    #[error("expected but did not find month-day delimiter '-'")]
    ExpectedMonthDayDelimiter,
    #[error("day representation must be exactly two digits")]
    DayRepresentationNotTwoDigits,
    #[error("could not parse entire string: data remains after Gregorian date")]
    UnexpectedRemainder,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
#[error("error parsing `JulianDate`")]
pub enum JulianDateParsingError {
    #[error(transparent)]
    IntegerParsingError(#[from] lexical_core::Error),
    #[error(transparent)]
    InvalidJulianDate(#[from] InvalidJulianDate),
    #[error(transparent)]
    InvalidMonthNumber(#[from] InvalidMonthNumber),
    #[error("expected but did not find year-month delimiter '-'")]
    ExpectedYearMonthDelimiter,
    #[error("month representation must be exactly two digits")]
    MonthRepresentationNotTwoDigits,
    #[error("expected but did not find month-day delimiter '-'")]
    ExpectedMonthDayDelimiter,
    #[error("day representation must be exactly two digits")]
    DayRepresentationNotTwoDigits,
    #[error("could not parse entire string: data remains after Julian date")]
    UnexpectedRemainder,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
#[error("error parsing `TimeOfDay`")]
pub enum TimeOfDayParsingError {
    #[error(transparent)]
    IntegerParsingError(#[from] lexical_core::Error),
    #[error("hour representation must be exactly two digits")]
    HourRepresentationNotTwoDigits,
    #[error("expected but did not find hour-minute delimiter ':'")]
    ExpectedHourMinuteDelimiter,
    #[error("minute representation must be exactly two digits")]
    MinuteRepresentationNotTwoDigits,
    #[error("expected but did not find minute-second delimiter ':'")]
    ExpectedMinuteSecondDelimiter,
    #[error("integer part of the second representation must be exactly two digits")]
    IntegerSecondRepresentationNotTwoDigits,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
#[error("error parsing `Duration`")]
pub enum DurationParsingError {
    #[error(transparent)]
    IntegerParsingError(#[from] lexical_core::Error),
    #[error("input string did not start with \'P\'")]
    ExpectedDurationPrefix,
    #[error("expected duration designator")]
    ExpectedDurationDesignator,
    #[error("could not parse entire string: data remains after duration")]
    UnexpectedRemainder,
    #[error("unit designators must be provided in decreasing error, but found {current}")]
    NonDecreasingDesignators { current: DurationDesignator },
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
#[error("error parsing `TimePoint`")]
pub enum TimePointParsingError<DateTimeError> {
    #[error(transparent)]
    DateParsingError(#[from] HistoricDateParsingError),
    #[error(transparent)]
    TimeOfDayParsingError(#[from] TimeOfDayParsingError),
    #[error("expected but did not find time designator 'T'")]
    ExpectedTimeDesignator,
    #[error("expected but did not find space between time-of-day and time scale designator")]
    ExpectedSpace,
    #[error("expected but did not find time scale designator")]
    ExpectedTimeScaleDesignator,
    #[error("could not parse entire string: data remains after time point")]
    UnexpectedRemainder,

    DateTimeError(#[source] DateTimeError),
}
