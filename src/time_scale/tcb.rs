//! Implementation of Barycentric Coordinate Time (TCB), describing the proper time experienced by
//! a clock at rest in a coordinate frame co-moving with the barycentre of the Solar system.

use crate::{
    Date, Duration, FromTimeScale, IntoTimeScale, Month, Tdb, TdbTime, TimePoint,
    time_scale::{AbsoluteTimeScale, TimeScale, datetime::UniformDateTimeScale},
};

pub type TcbTime = TimePoint<Tcb>;

/// Barycentric coordinate time scale
///
/// Time scale representing the Barycentric Coordinate Time (TCB). This scale is equivalent to the
/// proper time as experienced by an (idealistic) clock outside of Sun's gravity well, but
/// co-moving with the SSB. The resulting proper time is useful as independent variable for
/// high-accuracy ephemerides for Solar system objects, and as intermediate variable when
/// transforming into barycentric dynamical time.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Tcb;

impl TimeScale for Tcb {
    const NAME: &'static str = "Barycentric Coordinate Time";

    const ABBREVIATION: &'static str = "TCB";
}

impl AbsoluteTimeScale for Tcb {
    const EPOCH: Date = match Date::from_historic_date(1977, Month::January, 1) {
        Ok(epoch) => epoch,
        Err(_) => unreachable!(),
    };
}

impl UniformDateTimeScale for Tcb {}

impl<Scale: ?Sized> TimePoint<Scale> {
    #[must_use]
    pub fn from_tcb(time_point: TcbTime) -> Self
    where
        Self: FromTimeScale<Tcb>,
    {
        Self::from_time_scale(time_point)
    }

    #[must_use]
    pub fn into_tcb(self) -> TcbTime
    where
        Self: IntoTimeScale<Tcb>,
    {
        self.into_time_scale()
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

#[cfg(kani)]
mod proof_harness {
    use super::*;

    /// Verifies that construction of a TCB from a date and time stamp never panics.
    #[kani::proof]
    fn from_datetime_never_panics() {
        use crate::FromDateTime;
        let date: Date = kani::any();
        let hour: u8 = kani::any();
        let minute: u8 = kani::any();
        let second: u8 = kani::any();
        let _ = TcbTime::from_datetime(date, hour, minute, second);
    }
}
