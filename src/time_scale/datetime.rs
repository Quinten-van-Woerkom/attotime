//! Implementation of the concept of date and time-of-day within a time scale.

use crate::{
    Date, Days, Duration, Second, SecondsPerDay, SecondsPerHour, SecondsPerMinute, TimePoint,
    errors::InvalidTimeOfDay, time_scale::AbsoluteTimeScale,
};

/// Some time scales are uniform with respect to date-times: they do not apply leap seconds. In
/// such cases, their implementation of the `DateTime` mapping reduces to a simple add-and-multiply
/// of days, hours, minutes, and seconds with respect to the "arbitrary" measurement epoch in which
/// their resulting time points are measured.
///
/// This trait is only a marker trait.
pub trait UniformDateTimeScale: AbsoluteTimeScale {}

/// This trait may be implemented for time points that can be constructed based on a date-time
/// pair: they can connect a date and time-of-day to a specific time instant within their internal
/// scale and vice versa.
pub trait FromDateTime: Sized {
    /// This error may be returned whenever some input date-time is not valid. This may be the case
    /// when the time-of-day is not valid, but also when some date-time does not occur in a chosen
    /// time scale, for example due to leap seconds deletions or daylight saving time switches.
    type Error: core::error::Error;

    /// Maps a given combination of date and time-of-day to an instant on this time scale. May
    /// return an error if the input does not represent a valid combination of date and
    /// time-of-day.
    fn from_datetime(date: Date, hour: u8, minute: u8, second: u8) -> Result<Self, Self::Error>;
}

impl<Scale> FromDateTime for TimePoint<Scale>
where
    Scale: ?Sized + UniformDateTimeScale,
{
    type Error = InvalidTimeOfDay;

    fn from_datetime(date: Date, hour: u8, minute: u8, second: u8) -> Result<Self, Self::Error> {
        if hour >= 24 || minute >= 60 || second >= 60 {
            return Err(InvalidTimeOfDay {
                hour,
                minute,
                second,
            });
        }

        let days_since_scale_epoch = {
            let days_since_1970: Duration = date.time_since_epoch().into();
            let epoch_days_since_1970: Duration = Scale::EPOCH.time_since_epoch().into();
            days_since_1970 - epoch_days_since_1970
        };

        let hours = Duration::hours(hour.into());
        let minutes = Duration::minutes(minute.into());
        let seconds = Duration::seconds(second.into());
        let time_since_epoch = hours + minutes + seconds + days_since_scale_epoch;
        Ok(TimePoint::from_time_since_epoch(time_since_epoch))
    }
}

/// This trait may be implemented for time points that can be created based on "fine" date-time
/// pairs, which have subsecond accuracy.
pub trait FromFineDateTime: Sized {
    type Error: core::error::Error;

    /// Maps a given combination of date and fine time-of-day to an instant on this time scale. May
    /// return an error if the input does not represent a valid combination of date and
    /// time-of-day.
    fn from_fine_datetime(
        date: Date,
        hour: u8,
        minute: u8,
        second: u8,
        subseconds: Duration,
    ) -> Result<Self, Self::Error>;
}

/// This trait represents the fact that some time instant may be mapped back to the date-time pair
/// that it corresponds with, at an accuracy of seconds.
pub trait IntoDateTime: Sized {
    /// Maps a time point back to the date and time-of-day that it represents. Returns a tuple of
    /// date, hour, minute, and second. This function shall not fail, unless overflow occurs in the
    /// underlying integer arithmetic.
    fn into_datetime(self) -> (Date, u8, u8, u8);
}

impl<Scale> IntoDateTime for TimePoint<Scale>
where
    Scale: ?Sized + UniformDateTimeScale,
{
    fn into_datetime(self) -> (Date, u8, u8, u8) {
        // Step-by-step factoring of the time since epoch into days, hours, minutes, and seconds.
        let seconds_since_scale_epoch = self.time_since_epoch();
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
        let (hour, seconds_in_hour) = seconds_in_day.factor_out::<SecondsPerHour>();
        let (minute, second) = seconds_in_hour.factor_out::<SecondsPerMinute>();
        let second = second.floor::<Second>();
        let days_since_universal_epoch =
            <Scale as AbsoluteTimeScale>::EPOCH.time_since_epoch() + days_since_scale_epoch;
        let date = Date::from_time_since_epoch(days_since_universal_epoch);

        // We must narrow-cast all results, but only the cast of `date` may fail. The rest will
        // always succeed by construction: hour < 24, minute < 60, second < 60, so all fit in `u8`.
        (
            date,
            hour.try_into().unwrap_or_else(|_| panic!("Call of `datetime_from_time_point` results in hour value that cannot be expressed as `u8`")),
            minute.try_into().unwrap_or_else(|_| panic!("Call of `datetime_from_time_point` results in minute value that cannot be expressed as `u8`")),
            (second / Duration::seconds(1)).try_into().unwrap_or_else(|_| panic!("Call of `datetime_from_time_point` results in second value that cannot be expressed as `u8`")),
        )
    }
}

pub trait IntoFineDateTime {
    /// Convenience function that maps from a "fine" (subsecond-accuracy) time point to a date-time
    /// according to this time scale. Returns a tuple of date, hour, minute, second, and subsecond.
    /// Shall not fail, unless overflow occurs in the underlying integer arithmetic.
    fn into_fine_datetime(self) -> (Date, u8, u8, u8, Duration);
}
