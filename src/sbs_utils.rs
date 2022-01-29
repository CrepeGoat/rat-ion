use crate::nom_ext::take_ones;
use crate::nom_mod::take_partial;
use crate::utils::{IncompleteInt, InputStream};
use nom::bits::streaming::take;

use core::num::{NonZeroU64, NonZeroUsize};

/*
pub(crate) mod encode {
    use super::*;

    fn fits_next(value: u64) -> bool {
        unimplemented!()
    }

    fn write(value: u64) {
        unimplemented!()
    }
}
*/

pub(crate) mod decode {
    use super::*;

    pub(crate) fn read(
        stream: InputStream,
    ) -> Result<(InputStream, NonZeroU64), IncompleteInt<NonZeroU64>> {
        // Get prefixing ones stream
        let (stream, min_digits_len) = take_ones::<_, _, ()>(usize::MAX)(stream).unwrap();
        let (stream, _) = take::<_, u8, _, ()>(1_usize)(stream).map_err(|_| {
            IncompleteInt::new_unbounded(NonZeroU64::new(3 << min_digits_len).unwrap())
        })?;

        // Get first literal digit bit -> determines result's MSBs
        let (stream, first_digit) = take::<_, u8, _, ()>(1_usize)(stream).map_err(|_| {
            IncompleteInt::new_bounded(
                (
                    NonZeroU64::new(3 << min_digits_len).unwrap(),
                    NonZeroU64::new((6 << min_digits_len) - 1).unwrap(),
                ),
                NonZeroUsize::new(min_digits_len + 2).unwrap(),
            )
        })?;

        let digits_len = min_digits_len + (first_digit as usize);
        let leading_bits = (3 - (first_digit as u64)) << digits_len;

        match take_partial::<u64>(digits_len)(stream) {
            Ok((stream, result)) => Ok((stream, NonZeroU64::new(result + leading_bits).unwrap())),
            Err((partial, needed)) => Err(IncompleteInt::new_bounded(
                (
                    NonZeroU64::new(leading_bits + (partial << needed.get())).unwrap(),
                    NonZeroU64::new(
                        leading_bits + (partial << needed.get()) + (1 << needed.get()) - 1,
                    )
                    .unwrap(),
                ),
                NonZeroUsize::new(digits_len).unwrap(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

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
