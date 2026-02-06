//! Implementation of parsing logic for `Duration` types.

use core::str::FromStr;

use num_traits::ConstZero;

use crate::{Duration, errors::DurationParsingError};

impl FromStr for Duration {
    type Err = DurationParsingError;

    /// Parses a `Duration` type based on some ISO 8601 duration string. However, we additionally
    /// impose that months may not be used as duration, to prevent confusion with minutes (and
    /// because their precise duration cannot be unambiguously defined). Furthermore, we do not
    /// support use of the time designator ('T') inside duration expressions. Finally, we support
    /// years, days, hours, minutes, and seconds with any number of digits.
    ///
    /// For years, following the rest of this library, a duration of 31556952 seconds is used, which
    /// corresponds with the exact average duration of a Gregorian year.
    fn from_str(mut string: &str) -> Result<Self, Self::Err> {
        // Parse the mandatory duration prefix 'P'.
        if string.starts_with('P') {
            string = string.get(1..).unwrap();
        } else {
            return Err(DurationParsingError::ExpectedDurationPrefix);
        }
        parse_years_duration(string)
    }
}

/// Parses the remainder of an ISO 8601 duration string after a 'P'.
#[inline]
fn parse_years_duration(mut string: &str) -> Result<Duration, DurationParsingError> {
    if string.starts_with('T') {
        string = string.get(1..).unwrap();
        return parse_hours_duration(string, Duration::ZERO);
    }

    let (count, consumed_bytes) = lexical_core::parse_partial(string.as_bytes())?;
    string = string.get(consumed_bytes..).unwrap();
    if string.starts_with('.') {
        parse_years_fractional_duration(string, count)
    } else {
        parse_years_duration_designator(string, count)
    }
}

/// Parses the fractional duration of an ISO 8601 duration string. Applied to the case where no
/// other components have been parsed yet: units of years, months, days, hours, and seconds are
/// possible.
#[inline]
fn parse_years_fractional_duration(
    mut string: &str,
    count: i128,
) -> Result<Duration, DurationParsingError> {
    string = string.get(1..).unwrap();
    let (subcount, fractional_digits) = lexical_core::parse_partial(string.as_bytes())?;
    string = string.get(fractional_digits..).unwrap();

    let denominator = 10i128.pow(fractional_digits.try_into().unwrap());
    let numerator: i128 = subcount;

    let duration_designator = string
        .chars()
        .next()
        .ok_or(DurationParsingError::ExpectedDurationDesignator)?;
    string = string.get(1..).unwrap();

    if !string.is_empty() {
        return Err(DurationParsingError::UnexpectedRemainder);
    }

    match duration_designator {
        'Y' => Ok(Duration::years(count) + Duration::years(numerator).div_round(denominator)),
        'M' => Ok(Duration::months(count) + Duration::months(numerator).div_round(denominator)),
        'D' => Ok(Duration::days(count) + Duration::days(numerator).div_round(denominator)),
        'H' => Ok(Duration::hours(count) + Duration::hours(numerator).div_round(denominator)),
        'S' => Ok(Duration::seconds(count) + Duration::seconds(numerator).div_round(denominator)),
        _ => Err(DurationParsingError::ExpectedDurationDesignator),
    }
}

/// Parses the duration designator part of an ISO 8601 duration string. Applied to the case where
/// no other components have been parsed yet: units of years, months, days, hours, and seconds are
/// possible.
fn parse_years_duration_designator(
    mut string: &str,
    count: i128,
) -> Result<Duration, DurationParsingError> {
    let duration_designator = string
        .chars()
        .next()
        .ok_or(DurationParsingError::ExpectedDurationDesignator)?;
    string = string.get(1..).unwrap();
    match duration_designator {
        'Y' => parse_months_duration(string, Duration::years(count)),
        'M' => parse_days_duration(string, Duration::months(count)),
        'D' => parse_hours_duration(string, Duration::days(count)),
        'H' => parse_minutes_duration(string, Duration::hours(count)),
        'S' => {
            if !string.is_empty() {
                return Err(DurationParsingError::UnexpectedRemainder);
            }
            Ok(Duration::seconds(count))
        }
        _ => Err(DurationParsingError::ExpectedDurationDesignator),
    }
}

/// Parses the remainder of an ISO 8601 duration string after the years component has already been
/// parsed.
#[inline]
fn parse_months_duration(
    mut string: &str,
    duration: Duration,
) -> Result<Duration, DurationParsingError> {
    if string.is_empty() {
        return Ok(duration);
    }

    if string.starts_with('T') {
        string = string.get(1..).unwrap();
        return parse_hours_duration(string, duration);
    }

    let (count, consumed_bytes) = lexical_core::parse_partial(string.as_bytes())?;
    string = string.get(consumed_bytes..).unwrap();
    if string.starts_with('.') {
        parse_months_fractional_duration(string, duration, count)
    } else {
        parse_months_duration_designator(string, duration, count)
    }
}

/// Parses the fractional duration of an ISO 8601 duration string. Applied to the case where the
/// years component has been parsed: units of months, days, hours, and seconds are possible.
#[inline]
fn parse_months_fractional_duration(
    mut string: &str,
    duration: Duration,
    count: i128,
) -> Result<Duration, DurationParsingError> {
    string = string.get(1..).unwrap();
    let (subcount, fractional_digits) = lexical_core::parse_partial(string.as_bytes())?;
    string = string.get(fractional_digits..).unwrap();

    let denominator = 10i128.pow(fractional_digits.try_into().unwrap());
    let numerator: i128 = subcount;

    let duration_designator = string
        .chars()
        .next()
        .ok_or(DurationParsingError::ExpectedDurationDesignator)?;
    string = string.get(1..).unwrap();

    if !string.is_empty() {
        return Err(DurationParsingError::UnexpectedRemainder);
    }

    match duration_designator {
        'Y' => Err(DurationParsingError::NonDecreasingDesignators {
            current: DurationDesignator::Years,
        }),
        'M' => Ok(duration
            + Duration::months(count)
            + Duration::months(numerator).div_round(denominator)),
        'D' => {
            Ok(duration + Duration::days(count) + Duration::days(numerator).div_round(denominator))
        }
        'H' => Ok(duration
            + Duration::hours(count)
            + Duration::hours(numerator).div_round(denominator)),
        'S' => Ok(duration
            + Duration::seconds(count)
            + Duration::seconds(numerator).div_round(denominator)),
        _ => Err(DurationParsingError::ExpectedDurationDesignator),
    }
}

/// Parses the duration designator part of an ISO 8601 duration string. Applied to the case where
/// the years component has been parsed: units of months, days, hours, and seconds are possible.
fn parse_months_duration_designator(
    mut string: &str,
    duration: Duration,
    count: i128,
) -> Result<Duration, DurationParsingError> {
    let duration_designator = string
        .chars()
        .next()
        .ok_or(DurationParsingError::ExpectedDurationDesignator)?;
    string = string.get(1..).unwrap();

    if string.starts_with('T') {
        string = string.get(1..).unwrap();
    }

    match duration_designator {
        'Y' => Err(DurationParsingError::NonDecreasingDesignators {
            current: DurationDesignator::Years,
        }),
        'M' => parse_days_duration(string, duration + Duration::months(count)),
        'D' => parse_hours_duration(string, duration + Duration::days(count)),
        'H' => parse_minutes_duration(string, duration + Duration::hours(count)),
        'S' => {
            if !string.is_empty() {
                return Err(DurationParsingError::UnexpectedRemainder);
            }
            Ok(duration + Duration::seconds(count))
        }
        _ => Err(DurationParsingError::ExpectedDurationDesignator),
    }
}

/// Parses the remainder of an ISO 8601 duration string after the months component has already been
/// parsed.
#[inline]
fn parse_days_duration(
    mut string: &str,
    duration: Duration,
) -> Result<Duration, DurationParsingError> {
    if string.is_empty() {
        return Ok(duration);
    }

    if string.starts_with('T') {
        string = string.get(1..).unwrap();
        return parse_hours_duration(string, duration);
    }

    let (count, consumed_bytes) = lexical_core::parse_partial(string.as_bytes())?;
    string = string.get(consumed_bytes..).unwrap();
    if string.starts_with('.') {
        parse_days_fractional_duration(string, duration, count)
    } else {
        parse_days_duration_designator(string, duration, count)
    }
}

/// Parses the fractional duration of an ISO 8601 duration string. Applied to the case where the
/// months component has been parsed: units of days, hours, minutes, and seconds are possible.
#[inline]
fn parse_days_fractional_duration(
    mut string: &str,
    duration: Duration,
    count: i128,
) -> Result<Duration, DurationParsingError> {
    string = string.get(1..).unwrap();
    let (subcount, fractional_digits) = lexical_core::parse_partial(string.as_bytes())?;
    string = string.get(fractional_digits..).unwrap();

    let denominator = 10i128.pow(fractional_digits.try_into().unwrap());
    let numerator: i128 = subcount;

    let duration_designator = string
        .chars()
        .next()
        .ok_or(DurationParsingError::ExpectedDurationDesignator)?;
    string = string.get(1..).unwrap();

    if !string.is_empty() {
        return Err(DurationParsingError::UnexpectedRemainder);
    }

    match duration_designator {
        'Y' => Err(DurationParsingError::NonDecreasingDesignators {
            current: DurationDesignator::Years,
        }),
        'D' => {
            Ok(duration + Duration::days(count) + Duration::days(numerator).div_round(denominator))
        }
        'H' => Ok(duration
            + Duration::hours(count)
            + Duration::hours(numerator).div_round(denominator)),
        'M' => Ok(duration
            + Duration::minutes(count)
            + Duration::minutes(numerator).div_round(denominator)),
        'S' => Ok(duration
            + Duration::seconds(count)
            + Duration::seconds(numerator).div_round(denominator)),
        _ => Err(DurationParsingError::ExpectedDurationDesignator),
    }
}

/// Parses the duration designator part of an ISO 8601 duration string. Applied to the case where
/// the months component has been parsed: units of days, hours, minutes, and seconds are possible.
fn parse_days_duration_designator(
    mut string: &str,
    duration: Duration,
    count: i128,
) -> Result<Duration, DurationParsingError> {
    let duration_designator = string
        .chars()
        .next()
        .ok_or(DurationParsingError::ExpectedDurationDesignator)?;
    string = string.get(1..).unwrap();

    if string.starts_with('T') {
        string = string.get(1..).unwrap();
    }

    match duration_designator {
        'Y' => Err(DurationParsingError::NonDecreasingDesignators {
            current: DurationDesignator::Years,
        }),
        'D' => parse_hours_duration(string, duration + Duration::days(count)),
        'H' => parse_minutes_duration(string, duration + Duration::hours(count)),
        'M' => parse_seconds_duration(string, duration + Duration::minutes(count)),
        'S' => {
            if !string.is_empty() {
                return Err(DurationParsingError::UnexpectedRemainder);
            }
            Ok(duration + Duration::seconds(count))
        }
        _ => Err(DurationParsingError::ExpectedDurationDesignator),
    }
}

/// Parses the remainder of an ISO 8601 duration string after the days component has already been
/// parsed.
#[inline]
fn parse_hours_duration(
    mut string: &str,
    duration: Duration,
) -> Result<Duration, DurationParsingError> {
    if string.is_empty() {
        return Ok(duration);
    }

    let (count, consumed_bytes) = lexical_core::parse_partial(string.as_bytes())?;
    string = string.get(consumed_bytes..).unwrap();
    if string.starts_with('.') {
        parse_hours_fractional_duration(string, duration, count)
    } else {
        parse_hours_duration_designator(string, duration, count)
    }
}

/// Parses the fractional duration of an ISO 8601 duration string. Applied to the case where the
/// days component has been parsed: hours, minutes, and seconds are possible.
#[inline]
fn parse_hours_fractional_duration(
    mut string: &str,
    duration: Duration,
    count: i128,
) -> Result<Duration, DurationParsingError> {
    string = string.get(1..).unwrap();
    let (subcount, fractional_digits) = lexical_core::parse_partial(string.as_bytes())?;
    string = string.get(fractional_digits..).unwrap();

    let denominator = 10i128.pow(fractional_digits.try_into().unwrap());
    let numerator: i128 = subcount;

    let duration_designator = string
        .chars()
        .next()
        .ok_or(DurationParsingError::ExpectedDurationDesignator)?;
    string = string.get(1..).unwrap();

    if !string.is_empty() {
        return Err(DurationParsingError::UnexpectedRemainder);
    }

    match duration_designator {
        'Y' => Err(DurationParsingError::NonDecreasingDesignators {
            current: DurationDesignator::Years,
        }),
        'D' => Err(DurationParsingError::NonDecreasingDesignators {
            current: DurationDesignator::Days,
        }),
        'H' => Ok(duration
            + Duration::hours(count)
            + Duration::hours(numerator).div_round(denominator)),
        'M' => Ok(duration
            + Duration::minutes(count)
            + Duration::minutes(numerator).div_round(denominator)),
        'S' => Ok(duration
            + Duration::seconds(count)
            + Duration::seconds(numerator).div_round(denominator)),
        _ => Err(DurationParsingError::ExpectedDurationDesignator),
    }
}

/// Parses the duration designator part of an ISO 8601 duration string. Applied to the case where
/// the days component has been parsed: units of hours, minutes, and seconds are possible.
fn parse_hours_duration_designator(
    mut string: &str,
    duration: Duration,
    count: i128,
) -> Result<Duration, DurationParsingError> {
    let duration_designator = string
        .chars()
        .next()
        .ok_or(DurationParsingError::ExpectedDurationDesignator)?;
    string = string.get(1..).unwrap();

    match duration_designator {
        'Y' => Err(DurationParsingError::NonDecreasingDesignators {
            current: DurationDesignator::Years,
        }),
        'D' => Err(DurationParsingError::NonDecreasingDesignators {
            current: DurationDesignator::Days,
        }),
        'H' => parse_minutes_duration(string, duration + Duration::hours(count)),
        'M' => parse_seconds_duration(string, duration + Duration::minutes(count)),
        'S' => {
            if !string.is_empty() {
                return Err(DurationParsingError::UnexpectedRemainder);
            }
            Ok(duration + Duration::seconds(count))
        }
        _ => Err(DurationParsingError::ExpectedDurationDesignator),
    }
}

/// Parses the remainder of an ISO 8601 duration string after the hours component has already been
/// parsed.
#[inline]
fn parse_minutes_duration(
    mut string: &str,
    duration: Duration,
) -> Result<Duration, DurationParsingError> {
    if string.is_empty() {
        return Ok(duration);
    }

    let (count, consumed_bytes) = lexical_core::parse_partial(string.as_bytes())?;
    string = string.get(consumed_bytes..).unwrap();
    if string.starts_with('.') {
        parse_minutes_fractional_duration(string, duration, count)
    } else {
        parse_minutes_duration_designator(string, duration, count)
    }
}

/// Parses the fractional duration of an ISO 8601 duration string. Applied to the case where the
/// hours component has been parsed: minutes and seconds are possible.
#[inline]
fn parse_minutes_fractional_duration(
    mut string: &str,
    duration: Duration,
    count: i128,
) -> Result<Duration, DurationParsingError> {
    string = string.get(1..).unwrap();
    let (subcount, fractional_digits) = lexical_core::parse_partial(string.as_bytes())?;
    string = string.get(fractional_digits..).unwrap();

    let denominator = 10i128.pow(fractional_digits.try_into().unwrap());
    let numerator: i128 = subcount;

    let duration_designator = string
        .chars()
        .next()
        .ok_or(DurationParsingError::ExpectedDurationDesignator)?;
    string = string.get(1..).unwrap();

    if !string.is_empty() {
        return Err(DurationParsingError::UnexpectedRemainder);
    }

    match duration_designator {
        'Y' => Err(DurationParsingError::NonDecreasingDesignators {
            current: DurationDesignator::Years,
        }),
        'D' => Err(DurationParsingError::NonDecreasingDesignators {
            current: DurationDesignator::Days,
        }),
        'H' => Err(DurationParsingError::NonDecreasingDesignators {
            current: DurationDesignator::Hours,
        }),
        'M' => Ok(duration
            + Duration::minutes(count)
            + Duration::minutes(numerator).div_round(denominator)),
        'S' => Ok(duration
            + Duration::seconds(count)
            + Duration::seconds(numerator).div_round(denominator)),
        _ => Err(DurationParsingError::ExpectedDurationDesignator),
    }
}

/// Parses the duration designator part of an ISO 8601 duration string. Applied to the case where
/// the hours component has been parsed: units of minutes and seconds are possible.
fn parse_minutes_duration_designator(
    mut string: &str,
    duration: Duration,
    count: i128,
) -> Result<Duration, DurationParsingError> {
    let duration_designator = string
        .chars()
        .next()
        .ok_or(DurationParsingError::ExpectedDurationDesignator)?;
    string = string.get(1..).unwrap();

    if string.starts_with('T') {
        string = string.get(1..).unwrap();
    }

    match duration_designator {
        'Y' => Err(DurationParsingError::NonDecreasingDesignators {
            current: DurationDesignator::Years,
        }),
        'D' => Err(DurationParsingError::NonDecreasingDesignators {
            current: DurationDesignator::Days,
        }),
        'H' => Err(DurationParsingError::NonDecreasingDesignators {
            current: DurationDesignator::Hours,
        }),
        'M' => parse_seconds_duration(string, duration + Duration::minutes(count)),
        'S' => {
            if !string.is_empty() {
                return Err(DurationParsingError::UnexpectedRemainder);
            }
            Ok(duration + Duration::seconds(count))
        }
        _ => Err(DurationParsingError::ExpectedDurationDesignator),
    }
}

/// Parses the remainder of an ISO 8601 duration string after the hours component has already been
/// parsed.
#[inline]
fn parse_seconds_duration(
    mut string: &str,
    duration: Duration,
) -> Result<Duration, DurationParsingError> {
    if string.is_empty() {
        return Ok(duration);
    }

    let (count, consumed_bytes) = lexical_core::parse_partial(string.as_bytes())?;
    string = string.get(consumed_bytes..).unwrap();
    if string.starts_with('.') {
        parse_seconds_fractional_duration(string, duration, count)
    } else {
        parse_seconds_duration_designator(string, duration, count)
    }
}

/// Parses the fractional duration of an ISO 8601 duration string. Applied to the case where the
/// minutes component has been parsed: only seconds remain.
#[inline]
fn parse_seconds_fractional_duration(
    mut string: &str,
    duration: Duration,
    count: i128,
) -> Result<Duration, DurationParsingError> {
    string = string.get(1..).unwrap();
    let (subcount, fractional_digits) = lexical_core::parse_partial(string.as_bytes())?;
    string = string.get(fractional_digits..).unwrap();

    let denominator = 10i128.pow(fractional_digits.try_into().unwrap());
    let numerator: i128 = subcount;

    let duration_designator = string
        .chars()
        .next()
        .ok_or(DurationParsingError::ExpectedDurationDesignator)?;
    string = string.get(1..).unwrap();

    if !string.is_empty() {
        return Err(DurationParsingError::UnexpectedRemainder);
    }

    match duration_designator {
        'Y' => Err(DurationParsingError::NonDecreasingDesignators {
            current: DurationDesignator::Years,
        }),
        'D' => Err(DurationParsingError::NonDecreasingDesignators {
            current: DurationDesignator::Days,
        }),
        'H' => Err(DurationParsingError::NonDecreasingDesignators {
            current: DurationDesignator::Hours,
        }),
        'M' => Err(DurationParsingError::NonDecreasingDesignators {
            current: DurationDesignator::Minutes,
        }),
        'S' => Ok(duration
            + Duration::seconds(count)
            + Duration::seconds(numerator).div_round(denominator)),
        _ => Err(DurationParsingError::ExpectedDurationDesignator),
    }
}

/// Parses the duration designator part of an ISO 8601 duration string. Applied to the case where
/// the minutes component has been parsed: only units of seconds are possible.
fn parse_seconds_duration_designator(
    mut string: &str,
    duration: Duration,
    count: i128,
) -> Result<Duration, DurationParsingError> {
    let duration_designator = string
        .chars()
        .next()
        .ok_or(DurationParsingError::ExpectedDurationDesignator)?;
    string = string.get(1..).unwrap();

    if string.starts_with('T') {
        string = string.get(1..).unwrap();
    }

    match duration_designator {
        'Y' => Err(DurationParsingError::NonDecreasingDesignators {
            current: DurationDesignator::Years,
        }),
        'D' => Err(DurationParsingError::NonDecreasingDesignators {
            current: DurationDesignator::Days,
        }),
        'H' => Err(DurationParsingError::NonDecreasingDesignators {
            current: DurationDesignator::Hours,
        }),
        'M' => Err(DurationParsingError::NonDecreasingDesignators {
            current: DurationDesignator::Minutes,
        }),
        'S' => {
            if !string.is_empty() {
                return Err(DurationParsingError::UnexpectedRemainder);
            }
            Ok(duration + Duration::seconds(count))
        }
        _ => Err(DurationParsingError::ExpectedDurationDesignator),
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Duration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let string = self.to_string();
        serializer.serialize_str(&string)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Duration
where
    Self: FromStr,
    <Self as FromStr>::Err: core::fmt::Display,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        Self::from_str(&string).map_err(serde::de::Error::custom)
    }
}

/// The set of duration symbols that are supported when expressing durations as strings.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, derive_more::Display)]
pub enum DurationDesignator {
    Seconds,
    Minutes,
    Hours,
    Days,
    Years,
}

/// Tests that "simple" durations, made up of only one unit, can correctly be constructed.
#[test]
fn simple_durations() {
    let second = Duration::from_str("P1S").unwrap();
    assert_eq!(second, Duration::seconds(1));
    let seconds = Duration::from_str("P42S").unwrap();
    assert_eq!(seconds, Duration::seconds(42));

    let minute = Duration::from_str("PT1M").unwrap();
    assert_eq!(minute, Duration::minutes(1));
    let minutes = Duration::from_str("PT1998M").unwrap();
    assert_eq!(minutes, Duration::minutes(1998));

    let hour = Duration::from_str("P1H").unwrap();
    assert_eq!(hour, Duration::hours(1));
    let hours = Duration::from_str("P76H").unwrap();
    assert_eq!(hours, Duration::hours(76));

    let day = Duration::from_str("P1D").unwrap();
    assert_eq!(day, Duration::days(1));
    let days = Duration::from_str("P31415D").unwrap();
    assert_eq!(days, Duration::days(31415));

    let month = Duration::from_str("P1M").unwrap();
    assert_eq!(month, Duration::months(1));
    let months = Duration::from_str("P1998M").unwrap();
    assert_eq!(months, Duration::months(1998));

    let year = Duration::from_str("P1Y").unwrap();
    assert_eq!(year, Duration::years(1));
    let years = Duration::from_str("P2000Y").unwrap();
    assert_eq!(years, Duration::years(2000));
}

/// Verifies that simple composite durations can be constructed.
#[test]
fn simple_composite_durations() {
    let duration = Duration::from_str("P1Y1S").unwrap();
    assert_eq!(duration, Duration::years(1) + Duration::seconds(1));

    let duration = Duration::from_str("P1YT1M1S").unwrap();
    assert_eq!(
        duration,
        Duration::years(1) + Duration::minutes(1) + Duration::seconds(1)
    );

    let duration = Duration::from_str("P1YT1H1M1S").unwrap();
    assert_eq!(
        duration,
        Duration::years(1) + Duration::hours(1) + Duration::minutes(1) + Duration::seconds(1)
    );

    let duration = Duration::from_str("P1Y1DT1H1M1S").unwrap();
    assert_eq!(
        duration,
        Duration::years(1)
            + Duration::days(1)
            + Duration::hours(1)
            + Duration::minutes(1)
            + Duration::seconds(1)
    );

    let duration = Duration::from_str("P1Y1M1DT1H1M1S").unwrap();
    assert_eq!(
        duration,
        Duration::years(1)
            + Duration::months(1)
            + Duration::days(1)
            + Duration::hours(1)
            + Duration::minutes(1)
            + Duration::seconds(1)
    );

    let duration = Duration::from_str("P1Y2D3H4M5S").unwrap();
    assert_eq!(
        duration,
        Duration::seconds(31_556_952 + 2 * 86400 + 3 * 3600 + 4 * 60 + 5)
    );
}

/// Verifies that composite durations can be constructed.
#[test]
fn composite_durations() {
    let duration = Duration::from_str("P5S").unwrap();
    assert_eq!(duration, Duration::seconds(5));

    let duration = Duration::from_str("PT4M5S").unwrap();
    assert_eq!(duration, Duration::seconds(4 * 60 + 5));

    let duration = Duration::from_str("PT3H4M5S").unwrap();
    assert_eq!(duration, Duration::seconds(3 * 3600 + 4 * 60 + 5));

    let duration = Duration::from_str("P2D3H4M5S").unwrap();
    assert_eq!(
        duration,
        Duration::seconds(2 * 86400 + 3 * 3600 + 4 * 60 + 5)
    );

    let duration = Duration::from_str("P1Y2D3H4M5S").unwrap();
    assert_eq!(
        duration,
        Duration::seconds(31_556_952 + 2 * 86400 + 3 * 3600 + 4 * 60 + 5)
    );

    let duration = Duration::from_str("P1Y11M2D3H4M5S").unwrap();
    assert_eq!(
        duration,
        Duration::seconds(31_556_952 + 11 * 2_629_746 + 2 * 86400 + 3 * 3600 + 4 * 60 + 5)
    );

    let duration = Duration::from_str("P1Y11M2DT3H4M5S").unwrap();
    assert_eq!(
        duration,
        Duration::seconds(31_556_952 + 11 * 2_629_746 + 2 * 86400 + 3 * 3600 + 4 * 60 + 5)
    );
}

/// Verifies that it is possible to construct durations from sub-unit duration components as long
/// as the components can exactly be converted into the representation unit (e.g., 60 minutes can
/// be converted into an hour, so "PT60M" is a valid representation for hours).
#[test]
fn sub_unit_durations() {
    let hour = Duration::from_str("PT60M").unwrap();
    assert_eq!(hour, Duration::hours(1));
}

/// Checks whether fractional duration representations can be constructed.
#[test]
fn fractional_durations() {
    let milliseconds = Duration::from_str("P5.123S").unwrap();
    assert_eq!(milliseconds, Duration::milliseconds(5123));

    let milliseconds = Duration::from_str("P23H59M58.123S").unwrap();
    assert_eq!(
        milliseconds,
        Duration::milliseconds(58123 + 59 * 60_000 + 23 * 3_600_000)
    );

    let seconds = Duration::from_str("P23H59.5M").unwrap();
    assert_eq!(seconds, Duration::seconds(23 * 3600 + 59 * 60 + 30));
}
