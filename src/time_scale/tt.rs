//! Implementation of Terrestrial Time (TT).

use crate::{
    Date, Duration, FromTimeScale, IntoTimeScale, Month, TimePoint,
    time_scale::{AbsoluteTimeScale, TerrestrialTime, TimeScale, datetime::UniformDateTimeScale},
};

pub type TtTime = TimePoint<Tt>;

/// Time scale representing the Terrestrial Time (TT) scale. This scale is a constant 32.184
/// seconds ahead of TAI, but otherwise completely synchronized. It is used primarily as
/// independent variable in the context of planetary ephemerides.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Tt;

impl TimeScale for Tt {
    const NAME: &'static str = "Terrestrial Time";

    const ABBREVIATION: &'static str = "TT";
}

impl AbsoluteTimeScale for Tt {
    const EPOCH: Date = match Date::from_historic_date(1977, Month::January, 1) {
        Ok(epoch) => epoch,
        Err(_) => unreachable!(),
    };
}

impl UniformDateTimeScale for Tt {}

impl<Scale: ?Sized> TimePoint<Scale> {
    pub fn from_tt(time_point: TtTime) -> Self
    where
        Self: FromTimeScale<Tt>,
    {
        Self::from_time_scale(time_point)
    }

    pub fn into_tt(self) -> TtTime
    where
        Self: IntoTimeScale<Tt>,
    {
        self.into_time_scale()
    }
}

impl TerrestrialTime for Tt {
    const TAI_OFFSET: Duration = Duration::milliseconds(32_184);
}

/// Compares with a known timestamp as obtained from Vallado and McClain's "Fundamentals of
/// Astrodynamics".
#[test]
fn known_timestamps() {
    use crate::{Month, TaiTime};
    let tai = TaiTime::from_historic_datetime(2004, Month::May, 14, 16, 43, 32).unwrap();
    let tt = TtTime::from_fine_historic_datetime(
        2004,
        Month::May,
        14,
        16,
        44,
        4,
        crate::Duration::milliseconds(184),
    )
    .unwrap();
    assert_eq!(tai, tt.into_tai());
}

#[test]
fn date_decomposition() {
    use num_traits::ConstZero;

    let time = TtTime::from_historic_datetime(2004, Month::May, 14, 16, 44, 4).unwrap();
    let (date, hour, minute, second, subseconds) = time.into_fine_historic_datetime();
    assert_eq!(date.year(), 2004);
    assert_eq!(date.month(), Month::May);
    assert_eq!(date.day(), 14);
    assert_eq!(hour, 16);
    assert_eq!(minute, 44);
    assert_eq!(second, 4);
    assert_eq!(subseconds, Duration::ZERO);
}

#[cfg(kani)]
mod proof_harness {
    use super::*;
    use crate::TaiTime;

    /// Verifies that construction of a terrestrial time from a date and time stamp never
    /// panics, even for invalid date-time inputs.
    #[kani::proof]
    fn from_datetime_never_panics() {
        use crate::FromDateTime;
        let date: Date = kani::any();
        let hour: u8 = kani::any();
        let minute: u8 = kani::any();
        let second: u8 = kani::any();
        let _ = TtTime::from_datetime(date, hour, minute, second);
    }

    /// Verifies that all valid terrestrial time datetimes can be losslessly converted to and from
    /// the equivalent TAI time.
    #[kani::proof]
    fn datetime_tai_roundtrip() {
        use crate::{FromDateTime, IntoTimeScale};
        let date: Date = kani::any();
        let hour: u8 = kani::any();
        let minute: u8 = kani::any();
        let second: u8 = kani::any();
        kani::assume(hour < 24);
        kani::assume(minute < 60);
        kani::assume(second < 60);
        let time1 = TtTime::from_datetime(date, hour, minute, second);
        if let Ok(time1) = time1 {
            let tai: TaiTime = time1.into_tai();
            let time2: TtTime = tai.into_tt();
            assert_eq!(time1, time2);
        }
    }
}
