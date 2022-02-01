use crate::nom_ext::take_ones;
use crate::nom_mod::take_partial;
use crate::utils::{IncompleteInt, InputStream, OutputStream};
use nom::bits::streaming::take;
use nom::combinator::map;

use core::num::{NonZeroU64, NonZeroUsize};
use core::ops::{RangeFrom, RangeInclusive};

#[inline(always)]
const fn masked_suffix(bits: u64, len: usize) -> u64 {
    if len >= 8 * core::mem::size_of::<u64>() {
        bits
    } else {
        bits & !(u64::MAX << len)
    }
}

pub(crate) mod encode {
    use super::*;
    use crate::nom_mod::give8;

    #[inline]
    pub(super) const fn vlen_indicator(flen: usize, next_bit: bool) -> usize {
        flen + (next_bit as usize) - 3
    }

    #[inline]
    pub(super) const fn suffix_len(flen: usize) -> usize {
        flen - 2
    }

    pub(crate) fn write(
        stream: OutputStream,
        value: NonZeroU64,
    ) -> Result<OutputStream, IncompleteInt<NonZeroU64>> {
        const TYPE_BITS: usize = 8 * core::mem::size_of::<u64>();

        let value = value.get();
        let flen = 8 * core::mem::size_of::<u64>() - (value.leading_zeros() as usize);

        let flen_next_bit = (value & (1 << (flen - 2))) != 0;
        let vlen_next_bit = !flen_next_bit;
        let vlen_prefix = vlen_indicator(flen, flen_next_bit);
        let suffix_len = suffix_len(flen);
        let suffix_bits = masked_suffix(value, suffix_len);

        println!(
            "{:?}x<1> <0, {:?}> <{:b}; {:?}>",
            vlen_prefix, vlen_next_bit as u8, suffix_bits, suffix_len
        );

        // Write vlen prefix
        let mut stream = stream;
        let mut source_left = vlen_prefix;
        while source_left > 0 {
            let (_stream, (_, _source_left)) = give8(stream, (0xFF, source_left))
                .map_err(|_| decode::from_partial_length_indicator(vlen_prefix - source_left))
                .map_err(IncompleteInt::Unbounded)?;

            stream = _stream;
            source_left = _source_left;

            println!("{:?}, left = {}", stream, source_left);
        }
        let (stream, _) = give8(stream, (0x00, 1))
            .map_err(|_| decode::from_partial_length_indicator(vlen_prefix))
            .map_err(IncompleteInt::Unbounded)?;

        // Write next bit
        let (stream, _) = give8(stream, (vlen_next_bit as u8, 1)).map_err(|_| {
            IncompleteInt::Bounded(
                decode::from_full_length_indicator(vlen_prefix),
                NonZeroUsize::new(decode::suffix_len(vlen_prefix, true)).unwrap(),
            )
        })?;

        // Write suffix bits
        let suffix_bytes = suffix_bits.to_be_bytes();
        let source = (&suffix_bytes[..], 0_usize);
        let (source, _) = take::<_, u64, _, ()>(TYPE_BITS - suffix_len)(source).unwrap();

        let mut stream = stream;
        let mut source = source;
        let mut bits_left = suffix_len;
        while !source.0.is_empty() {
            let take_len = 8 - source.1;
            let (_source, source_val) = take::<_, u8, _, ()>(take_len)(source).unwrap();
            let (_stream, _) = give8(stream, (source_val, take_len)).map_err(|_| {
                IncompleteInt::Bounded(
                    decode::from_full_prefix(
                        vlen_prefix,
                        vlen_next_bit,
                        (suffix_bits >> bits_left, suffix_len - bits_left),
                    ),
                    NonZeroUsize::new(decode::suffix_len(vlen_prefix, true)).unwrap(),
                )
            })?;

            source = _source;
            stream = _stream;
            bits_left -= take_len;
        }
        assert_eq!(bits_left, 0);

        Ok(stream)
    }
}

pub(crate) mod decode {
    use super::*;

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
    #[trace]
    fn test_read(
        stream: InputStream,
        expt_result: Result<(InputStream, NonZeroU64), IncompleteInt<NonZeroU64>>,
    ) {
        let calc_result = decode::read(stream);
        assert_eq!(calc_result, expt_result);
    }

    #[rstest(value, expt_result,
        case(NonZeroU64::new(3).unwrap(), Ok((&[0b00111111][..], 2))),
        case(NonZeroU64::new(4).unwrap(), Ok((&[0b01011111][..], 3))),
        case(NonZeroU64::new(5).unwrap(), Ok((&[0b01111111][..], 3))),
        case(NonZeroU64::new(6).unwrap(), Ok((&[0b10001111][..], 4))),
        case(NonZeroU64::new(7).unwrap(), Ok((&[0b10011111][..], 4))),
        case(NonZeroU64::new(8).unwrap(), Ok((&[0b10100111][..], 5))),
        case(NonZeroU64::new(11).unwrap(), Ok((&[0b10111111][..], 5))),
        case(NonZeroU64::new(12).unwrap(), Ok((&[0b11000011][..], 6))),
        case(NonZeroU64::new(15).unwrap(), Ok((&[0b11001111][..], 6))),
        case(NonZeroU64::new(16).unwrap(), Ok((&[0b11010001][..], 7))),
        case(NonZeroU64::new(23).unwrap(), Ok((&[0b11011111][..], 7))),
        case(NonZeroU64::new(24).unwrap(), Ok((&[][..], 0))),
        case(NonZeroU64::new(31).unwrap(), Ok((&[][..], 0))),
    )]
    #[trace]
    fn test_write(
        value: NonZeroU64,
        expt_result: Result<InputStream, IncompleteInt<NonZeroU64>>, // <- need to use `InputStream` for immutabile static references
    ) {
        let stream = (&mut [0xFF_u8; 1][..], 0_usize);
        let calc_result = encode::write(stream, value);
        assert_eq!(
            calc_result.map(|(bits, bit_offset)| (&*bits, bit_offset)),
            expt_result,
        );
    }
}
