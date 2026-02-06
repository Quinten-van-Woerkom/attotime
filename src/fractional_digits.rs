//! Implementation of the ability to obtain an iterator over the fractional digits of some duration
//! representation. Used to represent subseconds when printing.

use num_traits::Zero;

// Back-up limit that is used to prevent infinite loops while printing. The value of this constant
// should not be relied upon for any practical reasons other than preventing infinite loops.
const ABSOLUTE_MAX_DIGITS: usize = 64;

/// Wrapper struct that implements `FractionalDigits` for all integers.
pub struct FractionalDigitsIterator {
    remainder: i128,
    denominator: i128,
    base: u8,
    precision: Option<usize>,
    current_digits: usize,
}

impl FractionalDigitsIterator {
    #[must_use]
    pub const fn from_signed(
        count: i128,
        numerator: i128,
        denominator: i128,
        precision: Option<usize>,
        base: u8,
    ) -> Self {
        let count = if count >= 0 { count } else { -count };
        let numerator = numerator * count;
        Self {
            remainder: numerator % denominator,
            denominator,
            base,
            precision,
            current_digits: 0,
        }
    }
}

impl Iterator for FractionalDigitsIterator {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        let keep_going = if let Some(precision) = self.precision {
            self.current_digits < precision
        } else {
            !self.remainder.is_zero()
        };

        if keep_going && self.current_digits < ABSOLUTE_MAX_DIGITS {
            self.current_digits += 1;
            self.remainder *= i128::from(self.base);
            let digit: u8 = (self.remainder / self.denominator)
                .try_into()
                .unwrap_or_else(|_| panic!());
            self.remainder %= self.denominator;
            Some(digit)
        } else {
            None
        }
    }
}

#[cfg(feature = "std")]
#[test]
fn integer_fractions() {
    let fraction: Vec<_> =
        FractionalDigitsIterator::from_signed(7854i128, 1, 1_000, Some(8), 10).collect();
    assert_eq!(fraction, vec![8, 5, 4, 0, 0, 0, 0, 0]);

    let fraction: Vec<_> = FractionalDigitsIterator::from_signed(
        1_234_567_890_123i128,
        1,
        1_000_000_000_000,
        Some(9),
        10,
    )
    .collect();
    assert_eq!(fraction, vec![2, 3, 4, 5, 6, 7, 8, 9, 0]);
}
