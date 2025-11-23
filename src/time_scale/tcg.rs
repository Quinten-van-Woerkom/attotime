//! Implementation of Geocentric Coordinate Time (TCG), describing the proper time experienced by a
//! clock at rest in a coordinate frame co-moving with the center of the Earth.

use crate::{
    Date, Duration, FromTimeScale, IntoTimeScale, Month, TerrestrialTime, TimePoint, TtTime,
    time_scale::{AbsoluteTimeScale, TimeScale, datetime::UniformDateTimeScale},
};

pub type TcgTime = TimePoint<Tcg>;

/// Time scale representing the Geocentric Coordinate Time (TCG). This scale is equivalent to the
/// proper time as experienced by an (idealistic) clock outside of Earth's gravity well, but
/// co-moving with the Earth. The resulting proper time is useful as independent variable for
/// high-accuracy ephemerides for Earth satellites, and as intermediate variable when transforming
/// into barycentric coordinate time.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Tcg;

impl TimeScale for Tcg {
    const NAME: &'static str = "Geocentric Coordinate Time";

    const ABBREVIATION: &'static str = "TCG";
}

impl AbsoluteTimeScale for Tcg {
    const EPOCH: Date = match Date::from_historic_date(1977, Month::January, 1) {
        Ok(epoch) => epoch,
        Err(_) => unreachable!(),
    };
}

impl UniformDateTimeScale for Tcg {}

impl TcgTime {
    fn into_tt(self) -> TtTime {
        let epoch_offset = Duration::milliseconds(32_184);
        let tcg_since_1977_01_01 = self.time_since_epoch();
        let tcg_since_1977_01_01_00_00_32_184 = tcg_since_1977_01_01 - epoch_offset;
        let rate_difference = (tcg_since_1977_01_01_00_00_32_184 * 3_484_645_067i128)
            .div_round(5_000_000_000_000_000_000);
        let tt_since_1977_01_01_00_00_32_184 = tcg_since_1977_01_01_00_00_32_184 - rate_difference;
        TtTime::from_time_since_epoch(epoch_offset) + tt_since_1977_01_01_00_00_32_184
    }

    fn from_tt(tt_time: TtTime) -> Self {
        let epoch_offset = Duration::milliseconds(32_184);
        let tt_since_1977_01_01 = tt_time.time_since_epoch();
        let tt_since_1977_01_01_00_00_32_184 = tt_since_1977_01_01 - epoch_offset;
        let rate_difference = (tt_since_1977_01_01_00_00_32_184 * 3_484_645_067i128)
            .div_round(4_999_999_996_515_354_933);
        let tcg_since_1977_01_01_00_00_32_184 = tt_since_1977_01_01_00_00_32_184 + rate_difference;
        TcgTime::from_time_since_epoch(epoch_offset) + tcg_since_1977_01_01_00_00_32_184
    }
}

impl<Scale> FromTimeScale<Scale> for TcgTime
where
    Scale: TerrestrialTime,
{
    fn from_time_scale(time_point: TimePoint<Scale>) -> Self {
        let tt_time = TtTime::from_time_scale(time_point);
        Self::from_tt(tt_time)
    }
}

impl<Scale> FromTimeScale<Tcg> for TimePoint<Scale>
where
    Scale: TerrestrialTime,
{
    fn from_time_scale(tcg_time: TcgTime) -> Self {
        let tt_time = tcg_time.into_tt();
        tt_time.into_time_scale()
    }
}

/// Compares with a known timestamp as obtained from the definition of TCG.
#[test]
fn known_timestamps() {
    use crate::{IntoTimeScale, Month, TaiTime};
    let tai = TaiTime::from_historic_datetime(1977, Month::January, 1, 0, 0, 0).unwrap();
    let tcg = TcgTime::from_fine_historic_datetime(
        1977,
        Month::January,
        1,
        0,
        0,
        32,
        Duration::milliseconds(184),
    )
    .unwrap();
    let tai_tt: TtTime = tai.into_time_scale();
    let tcg_tt: TtTime = tcg.into_time_scale();
    assert_eq!(tai_tt, tcg_tt);

    let tt = TtTime::from_fine_historic_datetime(
        1977,
        Month::January,
        1,
        0,
        0,
        32,
        Duration::milliseconds(184),
    )
    .unwrap();
    let tcg = TcgTime::from_fine_historic_datetime(
        1977,
        Month::January,
        1,
        0,
        0,
        32,
        Duration::milliseconds(184),
    )
    .unwrap();
    assert_eq!(tt, tcg.into_time_scale());
}

/// Verifies that conversion to and from TCG/TAI preserves identity.
#[test]
fn check_roundtrip() {
    use crate::IntoTimeScale;
    use rand::prelude::*;
    let mut rng = rand_chacha::ChaCha12Rng::seed_from_u64(44);
    for _ in 0..10_000 {
        let attoseconds_since_epoch = rng.random::<i64>();
        let time_since_epoch = Duration::attoseconds(attoseconds_since_epoch.into());
        let tt = TtTime::from_time_since_epoch(time_since_epoch);
        let tcg: TcgTime = TcgTime::from_time_scale(tt);
        let tt2 = tcg.into_time_scale();
        assert_eq!(tt, tt2);
    }
}

#[cfg(kani)]
mod proof_harness {
    use super::*;

    /// Verifies that construction of a TCG from a date and time stamp never panics.
    #[kani::proof]
    fn from_datetime_never_panics() {
        use crate::FromDateTime;
        let date: Date = kani::any();
        let hour: u8 = kani::any();
        let minute: u8 = kani::any();
        let second: u8 = kani::any();
        let _ = TcgTime::from_datetime(date, hour, minute, second);
    }
}
