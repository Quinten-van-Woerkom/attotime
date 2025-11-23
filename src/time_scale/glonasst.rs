//! Implementation of the GLONASS Time (GLONASST) time scale.

use crate::{
    Days, Duration, FromLeapSecondDateTime, IntoLeapSecondDateTime, IntoTimeScale,
    LeapSecondProvider, Second, TerrestrialTime, TimePoint,
    calendar::{Date, Month},
    errors::{InvalidGlonassDateTime, InvalidTimeOfDay},
    time_scale::{AbsoluteTimeScale, TimeScale},
    units::{SecondsPerDay, SecondsPerHour, SecondsPerMinute},
};

/// `GlonassTime` is a time point that is expressed according to the GLONASS Time time
/// scale.
pub type GlonassTime = TimePoint<Glonasst>;

/// The GLONASS Time (GLONASST) time scale is broadcast by GLONASS satellites. It follows UTC (or
/// rather, UTC(SU), which is a realization of UTC) and adds three hours (Moscow time). Indeed,
/// this means that it also incorporates leap seconds.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Glonasst;

impl TimeScale for Glonasst {
    const NAME: &'static str = "Glonass Time";

    const ABBREVIATION: &'static str = "GLONASST";
}

impl AbsoluteTimeScale for Glonasst {
    const EPOCH: Date = match Date::from_historic_date(1996, Month::January, 1) {
        Ok(epoch) => epoch,
        Err(_) => unreachable!(),
    };
}

impl TerrestrialTime for Glonasst {
    /// GLONASS time is in line with Moscow time (MSK), which is 3 hours ahead of UTC. Since leap
    /// seconds are accounted for in the date-time constructor, this means that GLONASST is three
    /// hours ahead of TAI.
    const TAI_OFFSET: Duration = Duration::hours(3);
}

impl FromLeapSecondDateTime for GlonassTime {
    type Error = InvalidGlonassDateTime;

    fn from_datetime(
        date: Date,
        hour: u8,
        minute: u8,
        second: u8,
        leap_second_provider: &impl LeapSecondProvider,
    ) -> Result<Self, Self::Error> {
        if hour > 23 || minute > 59 || second > 60 {
            return Err(InvalidGlonassDateTime::InvalidTimeOfDay(InvalidTimeOfDay {
                hour,
                minute,
                second,
            }));
        }

        let utc_date = if hour < 3 { date - Days::new(1) } else { date };
        let (is_leap_second, total_leap_seconds) =
            leap_second_provider.leap_seconds_on_date(utc_date);
        if second == 60 && !is_leap_second {
            return Err(InvalidGlonassDateTime::NonLeapSecondDateTime {
                date,
                hour,
                minute,
                second,
            });
        }

        let days_since_scale_epoch = {
            let days_since_1970 = date.time_since_epoch();
            let epoch_days_since_1970 = Glonasst::EPOCH.time_since_epoch();

            // First we try to compute the difference by subtracting first and then converting into
            // the target representation.
            days_since_1970 - epoch_days_since_1970
        };

        let hours = Duration::hours(hour.into());
        let minutes = Duration::minutes(minute.into());
        let seconds = Duration::seconds(second.into());
        let time_since_epoch = hours
            + minutes
            + seconds
            + Duration::seconds(total_leap_seconds.into())
            + days_since_scale_epoch.into();
        Ok(TimePoint::from_time_since_epoch(time_since_epoch))
    }
}

impl IntoLeapSecondDateTime for GlonassTime {
    fn into_datetime(self, leap_second_provider: &impl LeapSecondProvider) -> (Date, u8, u8, u8) {
        // Step-by-step factoring of the time since epoch into days, hours, minutes, and seconds.
        let seconds_since_scale_epoch = self.time_since_epoch();

        let utc_time = self.into_time_scale();
        let (is_leap_second, leap_seconds) = leap_second_provider.leap_seconds_at_time(utc_time);
        let leap_seconds = Duration::seconds(leap_seconds.into());

        let seconds_since_scale_epoch = seconds_since_scale_epoch - leap_seconds;
        let (days_since_scale_epoch, seconds_in_day) = {
            let factored = seconds_since_scale_epoch.floor::<SecondsPerDay>();
            let remainder = seconds_since_scale_epoch - factored;
            let factored = factored.count() / <SecondsPerDay as crate::UnitRatio>::ATTOSECONDS;
            (factored, remainder)
        };
        let days_since_scale_epoch = Days::new(
            days_since_scale_epoch
                .try_into()
                .unwrap_or_else(|_| panic!()),
        );

        let days_since_scale_epoch: Days = days_since_scale_epoch;
        let (hour, seconds_in_hour) = seconds_in_day.factor_out::<SecondsPerHour>();
        let (minute, second) = seconds_in_hour.factor_out::<SecondsPerMinute>();
        let second = second.floor::<Second>();
        let days_since_universal_epoch =
            Glonasst::EPOCH.time_since_epoch() + days_since_scale_epoch;
        let date = Date::from_time_since_epoch(days_since_universal_epoch);

        if is_leap_second {
            let date = date - Days::new(1);
            (date, 23, 59, 60)
        } else {
            (
            // We must narrow-cast all results, but only the cast of `date` may fail. The rest will
            // always succeed by construction: hour < 24, minute < 60, second < 60, so all fit in `u8`.
            date,
            hour.try_into().unwrap_or_else(|_| panic!("Call of `datetime_from_time_point` results in hour value that cannot be expressed as `u8`")),
            minute.try_into().unwrap_or_else(|_| panic!("Call of `datetime_from_time_point` results in minute value that cannot be expressed as `u8`")),
            (second / Duration::seconds(1)).try_into().unwrap_or_else(|_| panic!("Call of `datetime_from_time_point` results in second value that cannot be expressed as `u8`")),
        )
        }
    }
}

/// Compares with a known timestamp as obtained from the definition of the GLONASS time: the
/// epoch itself of the system. For GLONASST, two times could be considered as epoch:
/// 1996-01-01T00:00:00 UTC(SU), which is defined as start of the time scale, and
/// 1996-01-01T00:00:00 MSK, which is the epoch at which the broadcast time is 0. We just check
/// both times, and we also verify that the second is really the zero-duration point of this type.
#[test]
fn known_timestamps() {
    use crate::{IntoTimeScale, UtcTime};
    let utc = UtcTime::from_historic_datetime(1996, Month::January, 1, 0, 0, 0).unwrap();
    let glonasst = GlonassTime::from_historic_datetime(1996, Month::January, 1, 3, 0, 0).unwrap();
    assert_eq!(utc.into_time_scale(), glonasst);

    let utc = UtcTime::from_historic_datetime(1995, Month::December, 31, 21, 0, 0).unwrap();
    let glonasst = GlonassTime::from_historic_datetime(1996, Month::January, 1, 0, 0, 0).unwrap();
    assert_eq!(utc, glonasst.into_time_scale());
    // At the epoch time, 29 leap seconds are applied - this is the only offset that remains.
    assert_eq!(glonasst.time_since_epoch(), Duration::seconds(29));
}

#[cfg(test)]
fn date_roundtrip(year: i32, month: Month, day: u8, hour: u8, minute: u8, second: u8) {
    let time = GlonassTime::from_historic_datetime(year, month, day, hour, minute, second).unwrap();
    let (date, hour2, minute2, second2) = time.into_gregorian_datetime();
    assert_eq!(date.year(), year);
    assert_eq!(date.month(), month);
    assert_eq!(date.day(), day);
    assert_eq!(hour2, hour);
    assert_eq!(minute2, minute);
    assert_eq!(second2, second);
}

#[test]
fn date_decomposition() {
    date_roundtrip(1999, Month::August, 22, 0, 0, 0);
    date_roundtrip(1958, Month::January, 1, 0, 0, 0);
    date_roundtrip(1958, Month::January, 2, 0, 0, 0);
    date_roundtrip(1960, Month::January, 1, 0, 0, 0);
    date_roundtrip(1961, Month::January, 1, 0, 0, 0);
    date_roundtrip(1970, Month::January, 1, 0, 0, 0);
    date_roundtrip(1976, Month::January, 1, 0, 0, 0);
    date_roundtrip(2025, Month::July, 16, 16, 23, 24);
    date_roundtrip(2034, Month::December, 26, 8, 2, 37);
    date_roundtrip(2760, Month::April, 1, 21, 59, 58);
    date_roundtrip(1643, Month::January, 4, 1, 1, 33);
    date_roundtrip(1996, Month::January, 1, 3, 0, 0);
}
