//! Implementation of Coordinated Universal Time (UTC).

use num_traits::ConstZero;

use crate::{
    Date, Days, Duration, FromDateTime, FromTimeScale, IntoDateTime, IntoTimeScale,
    LeapSecondProvider, Month, Second, StaticLeapSecondProvider, TerrestrialTime, TimePoint,
    errors::{InvalidTimeOfDay, InvalidUtcDateTime},
    time_scale::{AbsoluteTimeScale, TimeScale},
    units::{SecondsPerDay, SecondsPerHour, SecondsPerMinute},
};

pub type UtcTime = TimePoint<Utc>;

/// Time scale representing Coordinated Universal Time (UTC). This scale is adjusted using leap
/// seconds to closely match the rotation of the Earth. This makes it useful as civil time scale,
/// but also means that external, discontinuous synchronization is required.
///
/// The synchronization based on leap seconds is implemented to occur at the date-time boundary.
/// This means that it is only done when a UTC time point is created based on a date-time pair,
/// after which it is converted into a time-since-epoch representation. This makes arithmetic over
/// UTC time points much more efficient and entirely correct over all possible leap second
/// boundaries.
///
/// This choice does also mean that introduction of new leap seconds does not "shift" any UTC time
/// stamps that were created to be after the point of introduction of this leap second. Generally,
/// this is desired behaviour, but in human communication it might not be. In such cases, users are
/// better off storing their UTC timestamps as date-time pairs and only converting them into
/// `UtcTime` at the point of use.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Utc;

impl TimeScale for Utc {
    const NAME: &'static str = "Coordinated Universal Time";

    const ABBREVIATION: &'static str = "UTC";
}

impl AbsoluteTimeScale for Utc {
    /// This epoch is the exact date at which the modern definition of UTC started. This makes it
    /// useful, because users may choose to permit "proleptic" UTC dates before 1972 by using a
    /// signed representation, but may also choose to forbid it by using unsigned arithmetic, which
    /// leads to easy-to-detect underflows whenever an ambiguous pre-1972 UTC date-time is created.
    const EPOCH: Date = match Date::from_historic_date(1972, Month::January, 1) {
        Ok(epoch) => epoch,
        Err(_) => unreachable!(),
    };
}

impl<Scale: ?Sized> TimePoint<Scale> {
    pub fn from_utc(time_point: UtcTime) -> Self
    where
        Self: FromTimeScale<Utc>,
    {
        Self::from_time_scale(time_point)
    }

    pub fn into_utc(self) -> UtcTime
    where
        Self: IntoTimeScale<Utc>,
    {
        self.into_time_scale()
    }
}

impl TerrestrialTime for Utc {
    /// Perhaps confusingly, we define UTC as coinciding with TAI. This is entirely possible
    /// because we handle leap seconds at the date-time boundary: after converting UTC into its
    /// time-since-epoch variation, there are no leap seconds to speak of anymore.
    const TAI_OFFSET: Duration = Duration::ZERO;
}

impl FromDateTime for UtcTime {
    type Error = InvalidUtcDateTime;

    fn from_datetime(date: Date, hour: u8, minute: u8, second: u8) -> Result<Self, Self::Error> {
        if hour > 23 || minute > 59 || second > 60 {
            return Err(InvalidUtcDateTime::InvalidTimeOfDay(InvalidTimeOfDay {
                hour,
                minute,
                second,
            }));
        }

        let (is_leap_second, leap_seconds) = StaticLeapSecondProvider {}.leap_seconds_on_date(date);
        if second == 60 && !is_leap_second {
            return Err(InvalidUtcDateTime::NonLeapSecondDateTime {
                date,
                hour,
                minute,
                second,
            });
        }

        let days_since_scale_epoch = {
            let days_since_1970 = date.time_since_epoch();
            let epoch_days_since_1970 = Utc::EPOCH.time_since_epoch();
            days_since_1970 - epoch_days_since_1970
        };

        let hours = Duration::hours(hour.into());
        let minutes = Duration::minutes(minute.into());
        let seconds = Duration::seconds(second.into());
        let time_since_epoch = hours
            + minutes
            + seconds
            + Duration::seconds(leap_seconds.into())
            + days_since_scale_epoch.into();
        Ok(TimePoint::from_time_since_epoch(time_since_epoch))
    }
}

impl IntoDateTime for UtcTime {
    fn into_datetime(self) -> (Date, u8, u8, u8) {
        // Step-by-step factoring of the time since epoch into days, hours, minutes, and seconds.
        let seconds_since_scale_epoch = self.time_since_epoch();

        let (is_leap_second, leap_seconds) = StaticLeapSecondProvider {}.leap_seconds_at_time(self);

        let seconds_since_scale_epoch =
            seconds_since_scale_epoch - Duration::seconds(leap_seconds.into());
        let (days_since_scale_epoch, seconds_in_day) = {
            let factored = seconds_since_scale_epoch.floor::<SecondsPerDay>();
            let remainder = seconds_since_scale_epoch - factored;
            let factored = factored.count() / <SecondsPerDay as crate::UnitRatio>::ATTOSECONDS;
            (factored, remainder)
        };
        let days_since_scale_epoch: Days = Days::new(days_since_scale_epoch
            .try_into()
            .unwrap_or_else(|_| panic!("Call of `datetime_from_time_point` results in days since scale epoch outside of `i32` range")));
        let (hour, seconds_in_hour) = seconds_in_day.factor_out::<SecondsPerHour>();
        let (minute, second) = seconds_in_hour.factor_out::<SecondsPerMinute>();
        let second = second.floor::<Second>();
        let days_since_universal_epoch = Utc::EPOCH.time_since_epoch() + days_since_scale_epoch;
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

/// Tests the creation of UTC time points from calendar dates for some known values. We explicitly
/// try out times near leap second insertions to see if those are handled properly, including:
/// - Durations should be handled correctly before, during, and after a leap second.
/// - If a leap second format (61 seconds) datetime is given for a non-leap second datetime, this
///   shall be caught and indicated.
#[test]
fn calendar_dates_near_insertion() {
    use crate::Month::*;
    // Leap second insertion of June 2015.
    let date = Date::from_historic_date(2015, June, 30).unwrap();
    let regular_second1 = UtcTime::from_datetime(date, 23, 59, 58).unwrap();
    let regular_second2 = UtcTime::from_datetime(date, 23, 59, 59).unwrap();
    assert_eq!(regular_second2 - regular_second1, Duration::seconds(1));
    let leap_second = UtcTime::from_datetime(date, 23, 59, 60).unwrap();
    assert_eq!(leap_second - regular_second2, Duration::seconds(1));
    assert_eq!(leap_second - regular_second1, Duration::seconds(2));
    let date2 = Date::from_historic_date(2015, July, 1).unwrap();
    let regular_second3 = UtcTime::from_datetime(date2, 0, 0, 0).unwrap();
    assert_eq!(regular_second3 - leap_second, Duration::seconds(1));

    // Leap second insertion of December 2016.
    let date = Date::from_historic_date(2016, December, 31).unwrap();
    let regular_second1 = UtcTime::from_datetime(date, 23, 59, 58).unwrap();
    let regular_second2 = UtcTime::from_datetime(date, 23, 59, 59).unwrap();
    assert_eq!(regular_second2 - regular_second1, Duration::seconds(1));
    let leap_second = UtcTime::from_datetime(date, 23, 59, 60).unwrap();
    assert_eq!(leap_second - regular_second2, Duration::seconds(1));
    assert_eq!(leap_second - regular_second1, Duration::seconds(2));
    let date2 = Date::from_historic_date(2017, January, 1).unwrap();
    let regular_second3 = UtcTime::from_datetime(date2, 0, 0, 0).unwrap();
    assert_eq!(regular_second3 - leap_second, Duration::seconds(1));

    // Non-leap second date: June 2016
    let date = Date::from_historic_date(2016, June, 30).unwrap();
    let regular_second1 = UtcTime::from_datetime(date, 23, 59, 58).unwrap();
    let regular_second2 = UtcTime::from_datetime(date, 23, 59, 59).unwrap();
    assert_eq!(regular_second2 - regular_second1, Duration::seconds(1));
    let leap_second = UtcTime::from_datetime(date, 23, 59, 60);
    assert_eq!(
        leap_second,
        Err(InvalidUtcDateTime::NonLeapSecondDateTime {
            date,
            hour: 23,
            minute: 59,
            second: 60
        })
    );
}

#[test]
fn trivial_times() {
    let epoch = UtcTime::from_historic_datetime(1972, Month::January, 1, 0, 0, 0).unwrap();
    assert_eq!(epoch.time_since_epoch(), Duration::seconds(10));
    let epoch = UtcTime::from_historic_datetime(1971, Month::December, 31, 23, 59, 60).unwrap();
    assert_eq!(epoch.time_since_epoch(), Duration::seconds(9));
}

#[test]
fn tai_roundtrip_near_leap_seconds() {
    use crate::Month::*;
    use crate::{FromTimeScale, HistoricDate, TaiTime};
    // Leap second insertion of June 2015.
    let date = HistoricDate::new(2015, June, 30).unwrap().into();
    let date2 = HistoricDate::new(2015, July, 1).unwrap().into();
    let date3 = HistoricDate::new(2016, December, 31).unwrap().into();
    let date4 = HistoricDate::new(2017, January, 1).unwrap().into();
    let date5 = HistoricDate::new(2016, June, 30).unwrap().into();

    let times = [
        UtcTime::from_datetime(date, 23, 59, 58).unwrap(),
        UtcTime::from_datetime(date, 23, 59, 59).unwrap(),
        UtcTime::from_datetime(date2, 0, 0, 0).unwrap(),
        UtcTime::from_datetime(date2, 0, 0, 1).unwrap(),
        UtcTime::from_datetime(date3, 23, 59, 58).unwrap(),
        UtcTime::from_datetime(date3, 23, 59, 59).unwrap(),
        UtcTime::from_datetime(date4, 0, 0, 0).unwrap(),
        UtcTime::from_datetime(date5, 23, 59, 58).unwrap(),
        UtcTime::from_datetime(date5, 23, 59, 59).unwrap(),
    ];

    for &time in times.iter() {
        let tai = TaiTime::from_time_scale(time);
        let time2 = tai.into_utc();
        assert_eq!(time, time2);
    }
}

#[test]
fn datetime_roundtrip_near_leap_seconds() {
    use crate::Month::*;
    use crate::{HistoricDate, IntoDateTime};

    // Leap second insertion of June 2015.
    let dates = [
        HistoricDate::new(2015, June, 30).unwrap().into(),
        HistoricDate::new(2015, July, 1).unwrap().into(),
        HistoricDate::new(2016, December, 31).unwrap().into(),
        HistoricDate::new(2017, January, 1).unwrap().into(),
        HistoricDate::new(2016, June, 30).unwrap().into(),
    ];

    let times_of_day = [(23, 59, 58), (23, 59, 59), (0, 0, 0), (0, 0, 1)];

    for date in dates.iter() {
        for time_of_day in times_of_day.iter() {
            let hour = time_of_day.0;
            let minute = time_of_day.1;
            let second = time_of_day.2;
            let utc_time = UtcTime::from_datetime(*date, hour, minute, second).unwrap();
            let datetime = utc_time.into_datetime();

            assert_eq!(datetime.0, *date);
            assert_eq!(datetime.1, hour);
            assert_eq!(datetime.2, minute);
            assert_eq!(datetime.3, second);
        }
    }
}
