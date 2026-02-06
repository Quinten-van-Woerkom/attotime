//! Implementation of Barycentric Dynamical Time (TDB), describing the proper time experienced by
//! a clock at rest in a coordinate frame co-moving with the barycentre of the Solar system.

use crate::{
    Date, Duration, FromTimeScale, IntoTimeScale, Month, Tcb, TcbTime, TimePoint, TtTime,
    time_scale::{AbsoluteTimeScale, TimeScale, datetime::UniformDateTimeScale},
};

pub type TdbTime = TimePoint<Tdb>;

/// Barycentric dynamical time scale
///
/// Time scale representing the Barycentric Dynamical Time (TDB). This scale is equivalent to the
/// proper time as experienced by an (idealistic) clock located at and co-moving with the SSB. The
/// resulting proper time is useful as independent variable for high-accuracy ephemerides for Solar
/// system objects.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Tdb;

impl TimeScale for Tdb {
    const NAME: &'static str = "Barycentric Dynamical Time";

    const ABBREVIATION: &'static str = "TDB";
}

impl AbsoluteTimeScale for Tdb {
    const EPOCH: Date = match Date::from_historic_date(1977, Month::January, 1) {
        Ok(epoch) => epoch,
        Err(_) => unreachable!(),
    };
}

impl UniformDateTimeScale for Tdb {}

impl<Scale: ?Sized> TimePoint<Scale> {
    #[must_use]
    pub fn from_tdb(time_point: TdbTime) -> Self
    where
        Self: FromTimeScale<Tdb>,
    {
        Self::from_time_scale(time_point)
    }

    #[must_use]
    pub fn into_tdb(self) -> TdbTime
    where
        Self: IntoTimeScale<Tdb>,
    {
        self.into_time_scale()
    }
}

impl FromTimeScale<Tcb> for TdbTime {
    #[allow(clippy::similar_names, reason = "Subsecond annotation needed")]
    fn from_time_scale(tcb_time: TcbTime) -> Self {
        const TDB0: Duration = Duration::nanoseconds(-65_500);
        const EPOCH_OFFSET: Duration = Duration::milliseconds(32_184);
        let tcb_since_1977_01_01 = tcb_time.time_since_epoch();
        let tcb_since_1977_01_01_00_00_32_184 = tcb_since_1977_01_01 - EPOCH_OFFSET;
        let rate_difference =
            (tcb_since_1977_01_01_00_00_32_184 * 193_814_971).div_round(12_500_000_000_000_000);
        let tdb_since_1977_01_01_00_00_32_184 = tcb_since_1977_01_01_00_00_32_184 - rate_difference;
        let tdb_since_1977_01_01 = tdb_since_1977_01_01_00_00_32_184 + EPOCH_OFFSET;
        Self::from_time_since_epoch(tdb_since_1977_01_01) + TDB0
    }
}

impl FromTimeScale<Tdb> for TcbTime {
    fn from_time_scale(time_point: TdbTime) -> Self {
        const TCB0: Duration = Duration::nanoseconds(65_500 * 12_500_000_000_000_000)
            .div_round(12_499_999_806_185_029);
        const EPOCH_OFFSET: Duration = Duration::milliseconds(32_184);
        let tdb_since_1977_01_01 = time_point.time_since_epoch();
        let tdb_since_1977_01_01_00_00_32_184 = tdb_since_1977_01_01 - EPOCH_OFFSET;
        let rate_difference =
            (tdb_since_1977_01_01_00_00_32_184 * 193_814_971).div_round(12_499_999_806_185_029);
        let difference = tdb_since_1977_01_01_00_00_32_184 + rate_difference + TCB0 + EPOCH_OFFSET;
        Self::from_time_since_epoch(difference)
    }
}

impl TtTime {
    /// Approximates Barycentric Dynamical Time (BDT) from TT using a simplified expression
    /// following the IAU SOFA estimate `TDB = TT + 0.001657 * sin(g)` where `g` is an estimate of
    /// the Earth's mean anomaly. The resulting estimate is accurate to 50 microseconds from 1980
    /// to 2100.
    ///
    /// See "SOFA Time Scale and Calendar Tools", 2023 May 31, version for the C programming
    /// language. Section 4.3.4 "TDB minus TT".
    #[allow(clippy::cast_precision_loss, reason = "Intended")]
    #[allow(clippy::cast_possible_truncation, reason = "Intended")]
    #[allow(clippy::missing_panics_doc, reason = "Infallible")]
    #[must_use]
    pub fn approximate_tdb(self) -> TdbTime {
        let j2000: Self = Self::from_historic_datetime(2000, Month::January, 1, 12, 0, 0).unwrap();
        let mean_anomaly_per_attosecond = 0.017_202 / (24. * 60. * 60.);
        let attoseconds_since_j2000 = (self - j2000).count();
        let mean_anomaly = 6.24 + mean_anomaly_per_attosecond * (attoseconds_since_j2000 as f64);
        let tdb_tt_offset = 0.001_657 * mean_anomaly.sin();
        let tdb_tt_attoseconds = tdb_tt_offset * 1e18;
        let tdb_tt_attoseconds = tdb_tt_attoseconds.round() as i128;
        let count = self.count() + tdb_tt_attoseconds;
        let time_since_epoch = Duration::attoseconds(count);
        TdbTime::from_time_since_epoch(time_since_epoch)
    }
}

#[cfg(kani)]
mod proof_harness {
    use super::*;

    /// Verifies that construction of a TDB from a date and time stamp never panics.
    #[kani::proof]
    fn from_datetime_never_panics() {
        use crate::FromDateTime;
        let date: Date = kani::any();
        let hour: u8 = kani::any();
        let minute: u8 = kani::any();
        let second: u8 = kani::any();
        let _ = TdbTime::from_datetime(date, hour, minute, second);
    }
}

/// Checks the TCB-TDB conversion using a known value from the SOFA Time Scale and Calendar Tools
/// documentation.
#[test]
fn known_tcb_to_tdb_conversion() {
    let tdb = TdbTime::from_fine_historic_datetime(
        2006,
        Month::January,
        15,
        21,
        25,
        42,
        Duration::microseconds(684_373),
    )
    .unwrap();
    let tcb = TcbTime::from_fine_historic_datetime(
        2006,
        Month::January,
        15,
        21,
        25,
        56,
        Duration::microseconds(893_952),
    )
    .unwrap();
    let difference = (tdb - tcb.into_tdb()).abs();
    assert!(difference < Duration::microseconds(1));
}

/// Checks the TDB-TCB conversion using a known value from the SOFA Time Scale and Calendar Tools
/// documentation.
#[test]
fn known_tdb_to_tcb_conversion() {
    let tdb = TdbTime::from_fine_historic_datetime(
        2006,
        Month::January,
        15,
        21,
        25,
        42,
        Duration::microseconds(684_373),
    )
    .unwrap();
    let tcb = TcbTime::from_fine_historic_datetime(
        2006,
        Month::January,
        15,
        21,
        25,
        56,
        Duration::microseconds(893_952),
    )
    .unwrap();
    let difference = (tcb - tdb.into_tcb()).abs();
    assert!(difference < Duration::microseconds(1));
}

/// Checks that roundtrip conversion to/from TCB/TDB is near-identity. Bar rounding errors, the
/// transformations should be each others inverse.
#[test]
fn roundtrip_tdb_tcb_conversion() {
    use rand::prelude::*;
    let mut rng = rand_chacha::ChaCha12Rng::seed_from_u64(44);
    for _ in 0..10_000 {
        let attoseconds_since_epoch = rng.random::<i32>();
        let time_since_epoch = Duration::attoseconds(attoseconds_since_epoch.into());
        let tdb = TdbTime::from_time_since_epoch(time_since_epoch);
        let tcb: TcbTime = TcbTime::from_tdb(tdb);
        let tdb2 = tcb.into_tdb();
        let difference = (tdb2 - tdb).abs();
        assert!(difference < Duration::attoseconds(10));
    }
}
