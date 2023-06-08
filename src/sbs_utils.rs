use crate::bitslice::{BitDecoder, BitEncoder};
use crate::utils::IncompleteInt;

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

#[inline(always)]
const fn masked_bit(bits: u64, index: usize) -> bool {
    (bits & (1_u64 << index)) != 0
}

pub(crate) mod encode {
    use super::*;

    #[inline]
    pub(super) const fn vlen_indicator(flen: usize, next_bit: bool) -> usize {
        flen + (next_bit as usize) - 3
    }

    #[inline]
    pub(super) const fn suffix_len(flen: usize) -> usize {
        flen - 2
    }

    pub(crate) fn write(
        bitstream: &mut BitEncoder,
        value: NonZeroU64,
    ) -> Result<(), IncompleteInt<NonZeroU64>> {
        let value = value.get();
        let flen = 8 * core::mem::size_of::<u64>() - (value.leading_zeros() as usize);

        let flen_next_bit = (value & (1 << (flen - 2))) != 0;
        let vlen_next_bit = !flen_next_bit;
        let vlen_prefix = vlen_indicator(flen, flen_next_bit);
        let suffix_len = suffix_len(flen);
        let suffix_bits = masked_suffix(value, suffix_len);

        // Write vlen prefix
        for sublen in 0..vlen_prefix {
            if let Err(_e) = bitstream.write_bit(true) {
                return Err(IncompleteInt::Unbounded(
                    decode::from_partial_length_indicator(sublen),
                ));
            }
        }

        if let Err(_e) = bitstream.write_bit(false) {
            return Err(IncompleteInt::Unbounded(
                decode::from_partial_length_indicator(vlen_prefix),
            ));
        }

        // Write next bit
        if let Err(_e) = bitstream.write_bit(vlen_next_bit) {
            return Err(IncompleteInt::Bounded(
                decode::from_full_length_indicator(vlen_prefix),
                NonZeroUsize::new(1 + decode::suffix_len(vlen_prefix, true))
                    .expect("known to be non-zero"),
            ));
        }

        // Write suffix bits
        for sublen in (0..suffix_len).rev() {
            if let Err(_e) = bitstream.write_bit(masked_bit(suffix_bits, sublen)) {
                return Err(IncompleteInt::Bounded(
                    decode::from_full_prefix(
                        vlen_prefix,
                        vlen_next_bit,
                        (
                            suffix_bits >> (suffix_len - sublen),
                            (suffix_len - sublen - 1),
                        ),
                    ),
                    NonZeroUsize::new(sublen + 1).expect("known to be non-zero"),
                ));
            }
        }

        assert_eq!(
            value,
            decode::from_full(vlen_prefix, vlen_next_bit, suffix_bits).get()
        );
        Ok(())
    }

    pub fn write_inf(bitstream: &mut BitEncoder) -> IncompleteInt<NonZeroU64> {
        for sublen in 0.. {
            if let Err(_e) = bitstream.write_bit(true) {
                return IncompleteInt::Unbounded(decode::from_partial_length_indicator(sublen));
            }
        }
        unreachable!()
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
        (3 ^ (next_bit as u64)) << suffix_len(vlen_prefix, next_bit)
    }

    #[inline]
    pub(super) fn from_partial_length_indicator(min_vlen_prefix: usize) -> RangeFrom<NonZeroU64> {
        RangeFrom {
            start: NonZeroU64::new(flen_prefix_bits(min_vlen_prefix, false))
                .expect("known to be non-zero"),
        }
    }

    #[inline]
    pub(super) fn from_full_length_indicator(vlen_prefix: usize) -> RangeInclusive<NonZeroU64> {
        RangeInclusive::new(
            NonZeroU64::new(flen_prefix_bits(vlen_prefix, false)).expect("known to be non-zero"),
            NonZeroU64::new(flen_prefix_bits(vlen_prefix + 1, false) - 1)
                .expect("known to be non-zero"),
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
            NonZeroU64::new(flen_prefix_bits | (partial_bits << needed_len))
                .expect("known to be non-zero"),
            NonZeroU64::new(flen_prefix_bits | (((partial_bits + 1) << needed_len) - 1))
                .expect("known to be non-zero"),
        )
    }

    #[inline]
    pub(super) fn from_full(vlen_prefix: usize, next_bit: bool, suffix_bits: u64) -> NonZeroU64 {
        let suffix_len = suffix_len(vlen_prefix, next_bit);
        let flen_prefix_bits = flen_prefix_bits(vlen_prefix, next_bit);
        let suffix_bits = masked_suffix(suffix_bits, suffix_len);

        NonZeroU64::new(flen_prefix_bits | suffix_bits).expect("known to be non-zero")
    }

    pub(crate) fn read(
        bitstream: &mut BitDecoder,
    ) -> Result<NonZeroU64, IncompleteInt<NonZeroU64>> {
        // Get prefixing ones stream
        let mut vlen_prefix = 0;
        while bitstream
            .read_bit()
            .map_err(|_| IncompleteInt::Unbounded(from_partial_length_indicator(vlen_prefix)))?
        {
            vlen_prefix += 1;
        }

        // Get first literal digit bit -> determines result's MSBs
        let first_digit = bitstream.read_bit().map_err(|_| {
            IncompleteInt::Bounded(
                from_full_length_indicator(vlen_prefix),
                NonZeroUsize::new(1 + suffix_len(vlen_prefix, true)).expect("known to be non-zero"),
            )
        })?;

        let suffix_len = suffix_len(vlen_prefix, first_digit);

        let mut suffix_bits: u64 = 0;
        for sublen in 0..suffix_len {
            let bit = bitstream.read_bit().map_err(|_| {
                IncompleteInt::Bounded(
                    from_full_prefix(vlen_prefix, first_digit, (suffix_bits, sublen)),
                    NonZeroUsize::new(suffix_len - sublen)
                        .expect("checked that suffix_len > sublen"),
                )
            })?;
            suffix_bits <<= 1;
            suffix_bits |= bit as u64;
        }

        Ok(from_full(vlen_prefix, first_digit, suffix_bits))
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
            assert!(
                decode::from_partial_length_indicator(min_vlen_prefix)
                    .contains(&decode::from_partial_length_indicator(min_vlen_prefix).start)
            );
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
        fn test_from_full_prefix(vlen_prefix in 0_usize..62) {
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

    #[rstest(stream, _read_len, expt_result,
        case([0b00111111], 2, Ok(NonZeroU64::new(3).unwrap())),
        case([0b01011111], 3, Ok(NonZeroU64::new(4).unwrap())),
        case([0b01111111], 3, Ok(NonZeroU64::new(5).unwrap())),
        case([0b10001111], 4, Ok(NonZeroU64::new(6).unwrap())),
        case([0b10011111], 4, Ok(NonZeroU64::new(7).unwrap())),
        case([0b10100111], 5, Ok(NonZeroU64::new(8).unwrap())),
        case([0b10111111], 5, Ok(NonZeroU64::new(11).unwrap())),
        case([0b11000011], 6, Ok(NonZeroU64::new(12).unwrap())),
        case([0b11001111], 6, Ok(NonZeroU64::new(15).unwrap())),
        case([0b11010001], 7, Ok(NonZeroU64::new(16).unwrap())),
        case([0b11011111], 7, Ok(NonZeroU64::new(23).unwrap())),
        case([0b11100000], 8, Ok(NonZeroU64::new(24).unwrap())),
        case([0b11100111], 8, Ok(NonZeroU64::new(31).unwrap())),
        case([0b11101000], 8, Err(
            IncompleteInt::new_bounded(
                (NonZeroU64::new(32).unwrap(), NonZeroU64::new(33).unwrap()),
                NonZeroUsize::new(1).unwrap(),
            ),
        )),
        case([0b11111111], 8, Err(
            IncompleteInt::new_unbounded(NonZeroU64::new(0x300).unwrap()),
        )),
    )]
    fn test_read(
        stream: [u8; 1],
        _read_len: usize,
        expt_result: Result<NonZeroU64, IncompleteInt<NonZeroU64>>,
    ) {
        let mut bitstream = BitDecoder::new(&stream[..]);
        let calc_result = decode::read(&mut bitstream);
        assert_eq!(calc_result, expt_result);
    }

    #[rstest(value, expt_stream, expt_result,
        case(NonZeroU64::new(3).unwrap(), [0b00111111], Ok(())),
        case(NonZeroU64::new(4).unwrap(), [0b01011111], Ok(())),
        case(NonZeroU64::new(5).unwrap(), [0b01111111], Ok(())),
        case(NonZeroU64::new(6).unwrap(), [0b10001111], Ok(())),
        case(NonZeroU64::new(7).unwrap(), [0b10011111], Ok(())),
        case(NonZeroU64::new(8).unwrap(), [0b10100111], Ok(())),
        case(NonZeroU64::new(11).unwrap(), [0b10111111], Ok(())),
        case(NonZeroU64::new(12).unwrap(), [0b11000011], Ok(())),
        case(NonZeroU64::new(15).unwrap(), [0b11001111], Ok(())),
        case(NonZeroU64::new(16).unwrap(), [0b11010001], Ok(())),
        case(NonZeroU64::new(23).unwrap(), [0b11011111], Ok(())),
        case(NonZeroU64::new(24).unwrap(), [0b11100000], Ok(())),
        case(NonZeroU64::new(31).unwrap(), [0b11100111], Ok(())),
        case(NonZeroU64::new(32).unwrap(), [0b11101000], Err(
            IncompleteInt::new_bounded(
                (NonZeroU64::new(32).unwrap(), NonZeroU64::new(33).unwrap()),
                NonZeroUsize::new(1).unwrap(),
            ),
        )),
        case(NonZeroU64::new(0x300).unwrap(), [0b11111111], Err(
            IncompleteInt::new_unbounded(NonZeroU64::new(0x300).unwrap()),
        )),
    )]
    fn test_write(
        value: NonZeroU64,
        expt_stream: [u8; 1],
        expt_result: Result<(), IncompleteInt<NonZeroU64>>, // <- need to use `InputStream` for immutabile static references
    ) {
        let mut stream = [0xFF_u8];
        let mut bitstream = BitEncoder::new(&mut stream[..]);
        let calc_result = encode::write(&mut bitstream, value);
        encode::write_inf(&mut bitstream);
        assert_eq!(calc_result, expt_result);
        assert_eq!(stream, expt_stream);
    }
}
