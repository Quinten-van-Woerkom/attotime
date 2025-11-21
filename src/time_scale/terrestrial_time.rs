//! This file implements the concept of a "terrestrial time", referring to any time scale which
//! represents the Platonic ideal of a time scale representing the elapsed time on the Earth geoid.

use crate::{Duration, FromTimeScale, TimePoint, time_scale::AbsoluteTimeScale};

/// In general, "terrestrial time" refers not just to the specific realization TT, but to an
/// idealized clock on the Earth geoid. It turns out that a lot of time scales are simply a variant
/// on terrestrial time (or, equivalently, TAI). All these time scales may easily be converted into
/// one another through a simple epoch offset: their internal clock rates are identical.
pub trait TerrestrialTime: AbsoluteTimeScale {
    const TAI_OFFSET: Duration;
}

impl<ScaleFrom, ScaleInto> FromTimeScale<ScaleFrom> for TimePoint<ScaleInto>
where
    ScaleFrom: TerrestrialTime,
    ScaleInto: TerrestrialTime,
{
    fn from_time_scale(time_point: TimePoint<ScaleFrom>) -> Self {
        let epoch_offset = ScaleFrom::EPOCH
            .elapsed_calendar_days_since(ScaleInto::EPOCH)
            .into();
        let from_offset: Duration = ScaleFrom::TAI_OFFSET;
        let into_offset: Duration = ScaleInto::TAI_OFFSET;
        // Depending on the sign, we flip the subtraction order. This is useful to ensure that we
        // do not overflow past zero for unsigned integers, and to keep the integer range needed as
        // small as possible in general.
        let time_since_epoch = if from_offset >= into_offset {
            let scale_offset = from_offset - into_offset;
            time_point.time_since_epoch() - scale_offset + epoch_offset
        } else {
            let scale_offset = into_offset - from_offset;
            time_point.time_since_epoch() + scale_offset + epoch_offset
        };
        Self::from_time_since_epoch(time_since_epoch)
    }
}
