//! Representation of Quasi-Zenith Satellite System Time (QZSST), which is broadcast by the
//! Quasi-Zenith Satellite System constellation.

use crate::{
    Date, Duration, Month, TerrestrialTime, TimePoint, UniformDateTimeScale,
    time_scale::{AbsoluteTimeScale, TimeScale},
};

pub type QzssTime = TimePoint<Qzsst>;

/// Time scale representing the Quasi-Zenith Satellite System Time (QZSST). QZSST has no leap
/// seconds and increases monotonically at a constant rate. It is distributed as part of the QZSST
/// broadcast messages, making it useful in a variety of high-accuracy situations.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Qzsst;

impl TimeScale for Qzsst {
    const NAME: &'static str = "Quasi-Zenith Satellite System Time";

    const ABBREVIATION: &'static str = "QZSST";
}

impl AbsoluteTimeScale for Qzsst {
    const EPOCH: Date = match Date::from_historic_date(1999, Month::August, 22) {
        Ok(epoch) => epoch,
        Err(_) => unreachable!(),
    };
}

impl UniformDateTimeScale for Qzsst {}

impl TerrestrialTime for Qzsst {
    const TAI_OFFSET: Duration = Duration::seconds(-19);
}

/// Compares with a known timestamp as obtained from Vallado and McClain's "Fundamentals of
/// Astrodynamics". Note that that timestamp is given for GPS time: QZSS time is always aligned
/// with GPS.
#[test]
fn known_timestamps() {
    use crate::{IntoTimeScale, TaiTime};
    let tai = TaiTime::from_historic_datetime(2004, Month::May, 14, 16, 43, 32).unwrap();
    let qzsst = QzssTime::from_historic_datetime(2004, Month::May, 14, 16, 43, 13).unwrap();
    assert_eq!(tai, qzsst.into_time_scale());
}
