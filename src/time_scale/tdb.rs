//! Implementation of Barycentric Dynamical Time (TDB), describing the proper time experienced by
//! a clock at rest in a coordinate frame co-moving with the barycentre of the Solar system.

use crate::{
    Date, Duration, Month, TimePoint, TtTime,
    time_scale::{AbsoluteTimeScale, TimeScale, datetime::UniformDateTimeScale},
};

pub type TdbTime = TimePoint<Tdb>;

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

impl TtTime {
    /// Approximates Barycentric Dynamical Time (BDT) from TT using a simplified expression
    /// following the IAU SOFA estimate `TDB = TT + 0.001657 * sin(g)` where `g` is an estimate of
    /// the Earth's mean anomaly. The resulting estimate is accurate to 50 microseconds from 1980
    /// to 2100.
    ///
    /// See "SOFA Time Scale and Calendar Tools", 2023 May 31, version for the C programming
    /// language. Section 4.3.4 "TDB minus TT".
    pub fn approximate_tdb(&self) -> TdbTime {
        let j2000: Self =
            TtTime::from_historic_datetime(2000, Month::January, 1, 12, 0, 0).unwrap();
        let mean_anomaly_per_attosecond = 0.017202 / (24. * 60. * 60.);
        let attoseconds_since_j2000 = (*self - j2000).count();
        let mean_anomaly = 6.24 + mean_anomaly_per_attosecond * (attoseconds_since_j2000 as f64);
        let tdb_tt_offset = 0.001657 * mean_anomaly.sin();
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
