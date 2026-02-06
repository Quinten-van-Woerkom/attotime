//! Leap seconds are applied when converting date-time pairs to underlying time scales, to better
//! align those time scales with the human-centric time based on the Earth's rotation (UT1).

use crate::{Date, Duration, FromDateTime, IntoDateTime, UtcTime};

/// Provider of leap second information
///
/// Since leap seconds are hard to predict in advance (due to irregular variations in the Earth's
/// rotation), their insertion and deletion is based on short-term predictions. This means that
/// it is not possible to develop a leap second table that holds "for all eternity" without
/// external influence. Different applications may have different manners of obtaining these
/// updates from external sources - if at all possible. To accommodate all such applications, we
/// support a generic manner of introducing leap seconds, via the `LeapSecondProvider` interface.
///
/// Any type that implements this trait may be used to determine when leap seconds occur, and how
/// often they do. In this manner, one may opt for a static leap second table but also easily swap
/// it for a table that updates based on the published IANA list, on GNSS constellation navigation
/// messages, or custom telecommands (for spacecraft, for example).
pub trait LeapSecondProvider {
    /// For any given date (expressed in UTC), determines whether a leap second was inserted at the
    /// end of that day. In tandem, returns the accumulated number of leap seconds before (!) that
    /// date.
    fn leap_seconds_on_date(&self, utc_date: Date) -> (bool, i32);

    /// Given some UTC time, returns the number of leap seconds that apply, and whether the
    /// requested date-time is a leap second (exactly).
    fn leap_seconds_at_time(&self, utc_time: UtcTime) -> (bool, i32);
}

/// This trait is the leap second equivalent of `FromDateTime`. It permits the creation of time
/// points from date-times when a non-standard leap second provider must be used.
pub trait FromLeapSecondDateTime: Sized {
    type Error: core::error::Error;

    /// Maps a given combination of date and time-of-day to an instant on this time scale.
    ///
    /// Takes a leap second provider as additional argument, which is used to determine at which
    /// times leap seconds are inserted or deleted.
    ///
    /// # Errors
    /// Will return an error if the input does not represent a valid combination of date and
    /// time-of-day.
    fn from_datetime(
        date: Date,
        hour: u8,
        minute: u8,
        second: u8,
        leap_second_provider: &impl LeapSecondProvider,
    ) -> Result<Self, Self::Error>;
}

/// We provide a default implementation that uses the static leap second provider.
impl<TimePoint> FromDateTime for TimePoint
where
    TimePoint: FromLeapSecondDateTime,
{
    type Error = <TimePoint as FromLeapSecondDateTime>::Error;

    fn from_datetime(date: Date, hour: u8, minute: u8, second: u8) -> Result<Self, Self::Error> {
        FromLeapSecondDateTime::from_datetime(
            date,
            hour,
            minute,
            second,
            &StaticLeapSecondProvider {},
        )
    }
}

/// This trait is the leap second equivalent of `IntoDateTime`. It permits the retrieval of
/// date-times from time points when a non-standard leap second provider must be used.
pub trait IntoLeapSecondDateTime: IntoDateTime {
    /// Maps a time point back to the date and time-of-day that it represents. Returns a tuple of
    /// date, hour, minute, and second. This function shall not fail, unless overflow occurs in the
    /// underlying integer arithmetic.
    ///
    /// Takes a leap second provider as additional argument, which is used to determine at which
    /// times leap seconds are inserted or deleted.
    fn into_datetime(self, leap_second_provider: &impl LeapSecondProvider) -> (Date, u8, u8, u8);
}

/// We provide a default implementation that uses the static leap second provider.
impl<TimePoint> IntoDateTime for TimePoint
where
    TimePoint: IntoLeapSecondDateTime,
{
    fn into_datetime(self) -> (Date, u8, u8, u8) {
        IntoLeapSecondDateTime::into_datetime(self, &StaticLeapSecondProvider {})
    }
}

/// Static leap second provider, baking in leap second information at build time
///
/// Default leap second provider that uses a pre-compiled table to obtain the leap seconds. Will
/// suffice for most non-critical applications and is useful in testing, but cannot be updated
/// after compilation. This makes it unsuitable for long-running applications.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct StaticLeapSecondProvider {}

/// Convenience constant that may be used to directly obtain a `StaticLeapSecondProvider` object.
pub const STATIC_LEAP_SECOND_PROVIDER: StaticLeapSecondProvider = StaticLeapSecondProvider {};

impl LeapSecondProvider for StaticLeapSecondProvider {
    /// For the static leap seconds provider, we just use a generated jump table that maps from
    /// days (expressed as `Date`, i.e., `Days` since 1970-01-01) to whether that day
    /// contains a leap second and what the total leap second count is. It is sorted in reverse,
    /// because it is more likely for users to work with dates in the present or future than in the
    /// past.
    fn leap_seconds_on_date(&self, utc_date: Date) -> (bool, i32) {
        let days_since_1970_01_01 = utc_date.time_since_epoch().count();
        let (is_leap_second, leap_seconds) = match days_since_1970_01_01 {
            17167.. => (false, 37),
            17166 => (true, 36),
            16617.. => (false, 36),
            16616 => (true, 35),
            15522.. => (false, 35),
            15521 => (true, 34),
            14245.. => (false, 34),
            14244 => (true, 33),
            13149.. => (false, 33),
            13148 => (true, 32),
            10592.. => (false, 32),
            10591 => (true, 31),
            10043.. => (false, 31),
            10042 => (true, 30),
            9496.. => (false, 30),
            9495 => (true, 29),
            8947.. => (false, 29),
            8946 => (true, 28),
            8582.. => (false, 28),
            8581 => (true, 27),
            8217.. => (false, 27),
            8216 => (true, 26),
            7670.. => (false, 26),
            7669 => (true, 25),
            7305.. => (false, 25),
            7304 => (true, 24),
            6574.. => (false, 24),
            6573 => (true, 23),
            5660.. => (false, 23),
            5659 => (true, 22),
            4929.. => (false, 22),
            4928 => (true, 21),
            4564.. => (false, 21),
            4563 => (true, 20),
            4199.. => (false, 20),
            4198 => (true, 19),
            3652.. => (false, 19),
            3651 => (true, 18),
            3287.. => (false, 18),
            3286 => (true, 17),
            2922.. => (false, 17),
            2921 => (true, 16),
            2557.. => (false, 16),
            2556 => (true, 15),
            2191.. => (false, 15),
            2190 => (true, 14),
            1826.. => (false, 14),
            1825 => (true, 13),
            1461.. => (false, 13),
            1460 => (true, 12),
            1096.. => (false, 12),
            1095 => (true, 11),
            912.. => (false, 11),
            911 => (true, 10),
            730.. => (false, 10),
            729 => (true, 9),
            _ => (false, 9),
        };
        (is_leap_second, leap_seconds)
    }

    /// To determine the leap second offset applicable at a given time, we just use a generated
    /// jump table, similar to the date-to-leap-seconds conversion. Note that leap seconds are
    /// applied only after the leap second itself: during a leap second, the count is still the
    /// same as before.
    fn leap_seconds_at_time(&self, utc_time: UtcTime) -> (bool, i32) {
        let seconds_since_1972_01_01 = utc_time.time_since_epoch() / Duration::seconds(1);
        let (is_leap_second, leap_seconds) = match seconds_since_1972_01_01 {
            1_420_156_837.. => (false, 37),
            1_420_156_836 => (true, 36),
            1_372_636_836.. => (false, 36),
            1_372_636_835 => (true, 35),
            1_278_028_835.. => (false, 35),
            1_278_028_834 => (true, 34),
            1_167_696_034.. => (false, 34),
            1_167_696_033 => (true, 33),
            1_073_001_633.. => (false, 33),
            1_073_001_632 => (true, 32),
            852_076_832.. => (false, 32),
            852_076_831 => (true, 31),
            804_643_231.. => (false, 31),
            804_643_230 => (true, 30),
            757_382_430.. => (false, 30),
            757_382_429 => (true, 29),
            709_948_829.. => (false, 29),
            709_948_828 => (true, 28),
            678_412_828.. => (false, 28),
            678_412_827 => (true, 27),
            646_876_827.. => (false, 27),
            646_876_826 => (true, 26),
            599_616_026.. => (false, 26),
            599_616_025 => (true, 25),
            568_080_025.. => (false, 25),
            568_080_024 => (true, 24),
            504_921_624.. => (false, 24),
            504_921_623 => (true, 23),
            425_952_023.. => (false, 23),
            425_952_022 => (true, 22),
            362_793_622.. => (false, 22),
            362_793_621 => (true, 21),
            331_257_621.. => (false, 21),
            331_257_620 => (true, 20),
            299_721_620.. => (false, 20),
            299_721_619 => (true, 19),
            252_460_819.. => (false, 19),
            252_460_818 => (true, 18),
            220_924_818.. => (false, 18),
            220_924_817 => (true, 17),
            189_388_817.. => (false, 17),
            189_388_816 => (true, 16),
            157_852_816.. => (false, 16),
            157_852_815 => (true, 15),
            126_230_415.. => (false, 15),
            126_230_414 => (true, 14),
            94_694_414.. => (false, 14),
            94_694_413 => (true, 13),
            63_158_413.. => (false, 13),
            63_158_412 => (true, 12),
            31_622_412.. => (false, 12),
            31_622_411 => (true, 11),
            15_724_811.. => (false, 11),
            15_724_810 => (true, 10),
            10.. => (false, 10),
            9 => (true, 9),
            _ => (false, 9),
        };
        (is_leap_second, leap_seconds)
    }
}
