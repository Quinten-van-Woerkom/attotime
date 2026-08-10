//! Implementation of Terrestrial Time (TT).

use crate::{
    Bdt, Date, Duration, FromTimeScale, Glonasst, Gpst, Gst, IntoTimeScale, Month, Qzsst, Tai, Tdb,
    TdbTime, TimePoint, Utc,
    time_scale::{
        AbsoluteTimeScale, TerrestrialTime, TimeScale, datetime::UniformDateTimeScale,
        tdb::tdb_tt_offset, terrestrial_time::impl_terrestrial_time_conversion,
    },
};

pub type TtTime = TimePoint<Tt>;

/// Terrestrial time scale
///
/// Time scale representing the Terrestrial Time (TT) scale. This scale is a constant 32.184
/// seconds ahead of TAI, but otherwise completely synchronized. It is used primarily as
/// independent variable in the context of planetary ephemerides.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Tt;

impl TimeScale for Tt {
    const NAME: &'static str = "Terrestrial Time";

    const ABBREVIATION: &'static str = "TT";
}

impl_terrestrial_time_conversion!(Tt, Bdt);
impl_terrestrial_time_conversion!(Tt, Glonasst);
impl_terrestrial_time_conversion!(Tt, Gpst);
impl_terrestrial_time_conversion!(Tt, Gst);
impl_terrestrial_time_conversion!(Tt, Qzsst);
impl_terrestrial_time_conversion!(Tt, Tai);
impl_terrestrial_time_conversion!(Tt, Utc);

impl AbsoluteTimeScale for Tt {
    const EPOCH: Date = match Date::from_historic_date(1977, Month::January, 1) {
        Ok(epoch) => epoch,
        Err(_) => unreachable!(),
    };
}

impl UniformDateTimeScale for Tt {}

impl<Scale: ?Sized> TimePoint<Scale> {
    #[must_use]
    pub fn from_tt(time_point: TtTime) -> Self
    where
        Self: FromTimeScale<Tt>,
    {
        Self::from_time_scale(time_point)
    }

    #[must_use]
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

impl TtTime {
    /// Approximates Barycentric Dynamical Time (TDB) from TT using a simplified expression
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
        const MEAN_ANOMALY_PER_ATTOSECOND: f64 = 0.017_202 / (24. * 60. * 60. * 1e18);
        let j2000: Self = Self::from_historic_datetime(2000, Month::January, 1, 12, 0, 0).unwrap();
        let attoseconds_since_j2000 = (self - j2000).count();
        let mean_anomaly =
            MEAN_ANOMALY_PER_ATTOSECOND.mul_add(attoseconds_since_j2000 as f64, 6.24);
        let tdb_tt_offset = 0.001_657 * mean_anomaly.sin();
        let tdb_tt_attoseconds = tdb_tt_offset * 1e18;
        let tdb_tt_attoseconds = tdb_tt_attoseconds.round() as i128;
        let count = self.count() + tdb_tt_attoseconds;
        let time_since_epoch = Duration::attoseconds(count);
        TdbTime::from_time_since_epoch(time_since_epoch)
    }
}

impl FromTimeScale<Tdb> for TtTime {
    #[allow(clippy::cast_possible_truncation, reason = "Intended")]
    fn from_time_scale(time_point: TimePoint<Tdb>) -> Self {
        let tdb_tt_offset = tdb_tt_offset(time_point);
        let tdb_tt_attoseconds = (tdb_tt_offset * 1e18).round() as i128;
        let count = time_point.count() - tdb_tt_attoseconds;
        Self::from_time_since_epoch(Duration::attoseconds(count))
    }
}

#[allow(clippy::doc_markdown, reason = "False positive on McClain")]
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
        use crate::FromDateTime;
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
