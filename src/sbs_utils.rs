use crate::nom_ext::take_ones;
use crate::nom_mod::take_partial;
use crate::utils::{IncompleteInt, InputStream};
use nom::bits::streaming::take;
use nom::combinator::map;

use core::num::{NonZeroU64, NonZeroUsize};
use core::ops::{RangeFrom, RangeInclusive};

pub(crate) mod decode {
    use super::*;

    #[inline(always)]
    const fn masked_suffix(bits: u64, len: usize) -> u64 {
        if len >= 8 * core::mem::size_of::<u64>() {
            bits
        } else {
            bits & !(u64::MAX << len)
        }
    }

    #[inline]
    pub(super) const fn suffix_len(vlen_prefix: usize, next_bit: bool) -> usize {
        vlen_prefix + (next_bit as usize)
    }

    #[inline]
    pub(super) const fn flen_prefix_bits(vlen_prefix: usize, next_bit: bool) -> u64 {
        (3 - (next_bit as u64)) << suffix_len(vlen_prefix, next_bit)
    }

    #[inline]
    pub(super) fn from_partial_length_indicator(min_vlen_prefix: usize) -> RangeFrom<NonZeroU64> {
        RangeFrom {
            start: NonZeroU64::new(flen_prefix_bits(min_vlen_prefix, false)).unwrap(),
        }
    }

    #[inline]
    pub(super) fn from_full_length_indicator(vlen_prefix: usize) -> RangeInclusive<NonZeroU64> {
        RangeInclusive::new(
            NonZeroU64::new(flen_prefix_bits(vlen_prefix, false)).unwrap(),
            NonZeroU64::new(flen_prefix_bits(vlen_prefix + 1, false) - 1).unwrap(),
        )
    }

    #[inline]
    pub(super) fn from_full_prefix(
        vlen_prefix: usize,
        next_bit: bool,
        partial_suffix: (u64, usize),
    ) -> RangeInclusive<NonZeroU64> {
        let (partial_bits, partial_len) = partial_suffix;
        let partial_bits = masked_suffix(partial_bits, partial_len);

        let suffix_len = suffix_len(vlen_prefix, next_bit);
        let flen_prefix_bits = flen_prefix_bits(vlen_prefix, next_bit);
        let needed_len = suffix_len - partial_len;

        RangeInclusive::new(
            NonZeroU64::new(flen_prefix_bits | (partial_bits << needed_len)).unwrap(),
            NonZeroU64::new(flen_prefix_bits | (((partial_bits + 1) << needed_len) - 1)).unwrap(),
        )
    }

    #[inline]
    pub(super) fn from_full(vlen_prefix: usize, next_bit: bool, suffix_bits: u64) -> NonZeroU64 {
        let suffix_len = suffix_len(vlen_prefix, next_bit);
        let flen_prefix_bits = flen_prefix_bits(vlen_prefix, next_bit);
        let suffix_bits = masked_suffix(suffix_bits, suffix_len);

        NonZeroU64::new(flen_prefix_bits | suffix_bits).unwrap()
    }

    pub(crate) fn read(
        stream: InputStream,
    ) -> Result<(InputStream, NonZeroU64), IncompleteInt<NonZeroU64>> {
        // Get prefixing ones stream
        let (stream, vlen_prefix) = take_ones::<_, _, ()>(usize::MAX)(stream).unwrap();
        let (stream, _) = take::<_, u8, _, ()>(1_usize)(stream)
            .map_err(|_| IncompleteInt::Unbounded(from_partial_length_indicator(vlen_prefix)))?;

        // Get first literal digit bit -> determines result's MSBs
        let (stream, first_digit) = map(take::<_, u8, _, ()>(1_usize), |fd| fd != 0)(stream)
            .map_err(|_| {
                IncompleteInt::Bounded(
                    from_full_length_indicator(vlen_prefix),
                    NonZeroUsize::new(1 + suffix_len(vlen_prefix, true)).unwrap(),
                )
            })?;

        let suffix_len = suffix_len(vlen_prefix, first_digit);

        match take_partial::<u64>(suffix_len)(stream) {
            Ok((stream, result)) => Ok((stream, from_full(vlen_prefix, first_digit, result))),
            Err((partial, needed)) => Err(IncompleteInt::Bounded(
                from_full_prefix(
                    suffix_len,
                    first_digit,
                    (partial, suffix_len - needed.get()),
                ),
                needed,
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::*;
    use rstest::*;

    proptest! {
        #[test]
        fn test_from_partial_length_indicator(min_vlen_prefix in 0_usize..64) {
            assert!(decode::from_partial_length_indicator(min_vlen_prefix).contains(&decode::from_partial_length_indicator(min_vlen_prefix).start));
        }

        #[test]
        fn test_from_full_length_indicator(vlen_prefix in 0_usize..63) {
            let result = decode::from_full_length_indicator(vlen_prefix);

            assert_eq!(
                result.start(),
                &decode::from_partial_length_indicator(vlen_prefix).start,
            );
            assert_eq!(
                result.end().get(),
                decode::from_partial_length_indicator(vlen_prefix+1).start.get() - 1,
            );
        }

        #[test]
        fn from_full_prefix(vlen_prefix in 0_usize..62) {
            let result1 = decode::from_full_prefix(vlen_prefix, false, (0, 0));
            let result2 = decode::from_full_prefix(vlen_prefix, true, (0, 0));

            assert_eq!(
                result1.start(),
                decode::from_full_length_indicator(vlen_prefix).start(),
            );
            assert_eq!(result1.end().get(), result2.start().get() - 1);
            assert_eq!(
                result2.end().get(),
                decode::from_full_length_indicator(vlen_prefix+1).start().get() - 1,
            );
        }
    }

    #[rstest(stream, expt_result,
        case((&[0b00111111][..], 0), Ok(((&[0b00111111][..], 2), NonZeroU64::new(3).unwrap()))),
        case((&[0b01011111][..], 0), Ok(((&[0b01011111][..], 3), NonZeroU64::new(4).unwrap()))),
        case((&[0b01111111][..], 0), Ok(((&[0b01111111][..], 3), NonZeroU64::new(5).unwrap()))),
        case((&[0b10001111][..], 0), Ok(((&[0b10001111][..], 4), NonZeroU64::new(6).unwrap()))),
        case((&[0b10011111][..], 0), Ok(((&[0b10011111][..], 4), NonZeroU64::new(7).unwrap()))),
        case((&[0b10100111][..], 0), Ok(((&[0b10100111][..], 5), NonZeroU64::new(8).unwrap()))),
        case((&[0b10111111][..], 0), Ok(((&[0b10111111][..], 5), NonZeroU64::new(11).unwrap()))),
        case((&[0b11000011][..], 0), Ok(((&[0b11000011][..], 6), NonZeroU64::new(12).unwrap()))),
        case((&[0b11001111][..], 0), Ok(((&[0b11001111][..], 6), NonZeroU64::new(15).unwrap()))),
        case((&[0b11010001][..], 0), Ok(((&[0b11010001][..], 7), NonZeroU64::new(16).unwrap()))),
        case((&[0b11011111][..], 0), Ok(((&[0b11011111][..], 7), NonZeroU64::new(23).unwrap()))),
        case((&[0b11100000][..], 0), Ok(((&[][..], 0), NonZeroU64::new(24).unwrap()))),
        case((&[0b11100111][..], 0), Ok(((&[][..], 0), NonZeroU64::new(31).unwrap()))),
    )]
    fn test_read(
        stream: InputStream,
        expt_result: Result<(InputStream, NonZeroU64), IncompleteInt<NonZeroU64>>,
    ) {
        let calc_result = decode::read(stream);
        assert_eq!(calc_result, expt_result);
    }
}
