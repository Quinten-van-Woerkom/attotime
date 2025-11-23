//! Implementation of the time broadcast by the Global Positioning System (GPS).

use crate::{
    Date, Duration, FromTimeScale, IntoTimeScale, Month, TerrestrialTime, TimePoint,
    UniformDateTimeScale,
    time_scale::{AbsoluteTimeScale, TimeScale},
};

pub type GpsTime = TimePoint<Gpst>;

/// Time scale representing the Global Positioning System Time (GPST). GPST has no leap seconds
/// and increases monotonically at a constant rate. It is distributed as part of the GPS broadcast
/// messages, making it useful in a variety of high-accuracy situations.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Gpst;

impl TimeScale for Gpst {
    const NAME: &'static str = "Global Positioning System Time";

    const ABBREVIATION: &'static str = "GPST";
}

impl AbsoluteTimeScale for Gpst {
    const EPOCH: Date = match Date::from_historic_date(1980, Month::January, 6) {
        Ok(epoch) => epoch,
        Err(_) => unreachable!(),
    };
}

impl UniformDateTimeScale for Gpst {}

impl<Scale: ?Sized> TimePoint<Scale> {
    pub fn from_gpst(time_point: GpsTime) -> Self
    where
        Self: FromTimeScale<Gpst>,
    {
        Self::from_time_scale(time_point)
    }

    pub fn into_gpst(self) -> GpsTime
    where
        Self: IntoTimeScale<Gpst>,
    {
        self.into_time_scale()
    }
}

impl TerrestrialTime for Gpst {
    const TAI_OFFSET: Duration = Duration::seconds(-19);
}

/// Compares with a known timestamp as obtained from Vallado and McClain's "Fundamentals of
/// Astrodynamics".
#[test]
fn known_timestamps() {
    use crate::TaiTime;
    let tai = TaiTime::from_historic_datetime(2004, Month::May, 14, 16, 43, 32).unwrap();
    let gpst = GpsTime::from_historic_datetime(2004, Month::May, 14, 16, 43, 13).unwrap();
    assert_eq!(tai, gpst.into_tai());
}
