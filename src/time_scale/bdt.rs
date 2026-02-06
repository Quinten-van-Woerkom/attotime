#![allow(clippy::doc_markdown, reason = "False positive on BeiDou")]
//! Representation of BeiDou Time (BDT), which is broadcast by the BeiDou constellation.

use crate::{
    Date, Duration, FromTimeScale, IntoTimeScale, Month, TerrestrialTime, TimePoint,
    UniformDateTimeScale,
    time_scale::{AbsoluteTimeScale, TimeScale},
};

pub type BeiDouTime = TimePoint<Bdt>;

/// BeiDou time scale
///
/// Time scale representing the BeiDou Time (BDT). BDT has no leap seconds and increases
/// monotonically at a constant rate. It is distributed as part of the BeiDou broadcast messages,
/// making it useful in a variety of high-accuracy situations.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Bdt;

impl TimeScale for Bdt {
    const NAME: &'static str = "BeiDou Time";

    const ABBREVIATION: &'static str = "BDT";
}

impl AbsoluteTimeScale for Bdt {
    const EPOCH: Date = match Date::from_historic_date(2006, Month::January, 1) {
        Ok(epoch) => epoch,
        Err(_) => unreachable!(),
    };
}

impl UniformDateTimeScale for Bdt {}

impl<Scale: ?Sized> TimePoint<Scale> {
    #[must_use]
    pub fn from_bdt(time_point: BeiDouTime) -> Self
    where
        Self: FromTimeScale<Bdt>,
    {
        Self::from_time_scale(time_point)
    }

    #[must_use]
    pub fn into_bdt(self) -> BeiDouTime
    where
        Self: IntoTimeScale<Bdt>,
    {
        self.into_time_scale()
    }
}

impl TerrestrialTime for Bdt {
    const TAI_OFFSET: Duration = Duration::seconds(-33);
}

/// Compares with a known timestamp as obtained from the definition of the BeiDou Time: the
/// epoch itself of the system.
#[test]
fn known_timestamps() {
    use crate::UtcTime;
    let utc = UtcTime::from_historic_datetime(2006, Month::January, 1, 0, 0, 0).unwrap();
    let bdt = BeiDouTime::from_historic_datetime(2006, Month::January, 1, 0, 0, 0).unwrap();
    assert_eq!(utc, bdt.into_utc());
}
