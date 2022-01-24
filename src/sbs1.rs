use crate::sbs_utils;
use crate::utils::{IncompleteInt, InputStream};
use core::num::NonZeroU64;
use core::ops::RangeFrom;
use nom::bits::streaming::take;

/*
mod encode {
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
        stream: (&[u8], usize),
    ) -> Result<(InputStream, NonZeroU64), IncompleteInt<NonZeroU64>> {
        let (stream, first_bit) = take::<_, u8, _, ()>(1_usize)(stream).map_err(|_| {
            IncompleteInt::Unbounded(RangeFrom {
                start: NonZeroU64::new(1).unwrap(),
            })
        })?;
        if first_bit == 0 {
            Ok((stream, unsafe { NonZeroU64::new_unchecked(1) }))
        } else {
            match sbs_utils::decode::read(stream) {
                Ok((stream, result)) => Ok((stream, NonZeroU64::new(result.get() - 1).unwrap())),
                Err(IncompleteInt::Bounded(range, bits_left)) => Err(IncompleteInt::new_bounded(
                    (
                        NonZeroU64::new(range.start().get() - 1).unwrap(),
                        NonZeroU64::new(range.end().get() - 1).unwrap(),
                    ),
                    bits_left,
                )),
                Err(IncompleteInt::Unbounded(RangeFrom { start: b0 })) => Err(
                    IncompleteInt::new_unbounded(NonZeroU64::new(b0.get() - 1).unwrap()),
                ),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest(stream, expt_result,
        case((&[0b01111111][..], 0), Ok(((&[0b01111111][..], 1), NonZeroU64::new(1).unwrap()))),
        case((&[0b10011111][..], 0), Ok(((&[0b10011111][..], 3), NonZeroU64::new(2).unwrap()))),
        case((&[0b10101111][..], 0), Ok(((&[0b10101111][..], 4), NonZeroU64::new(3).unwrap()))),
        case((&[0b10111111][..], 0), Ok(((&[0b10111111][..], 4), NonZeroU64::new(4).unwrap()))),
        case((&[0b11000111][..], 0), Ok(((&[0b11000111][..], 5), NonZeroU64::new(5).unwrap()))),
        case((&[0b11001111][..], 0), Ok(((&[0b11001111][..], 5), NonZeroU64::new(6).unwrap()))),
        case((&[0b11010011][..], 0), Ok(((&[0b11010011][..], 6), NonZeroU64::new(7).unwrap()))),
        case((&[0b11011111][..], 0), Ok(((&[0b11011111][..], 6), NonZeroU64::new(10).unwrap()))),
        case((&[0b11100001][..], 0), Ok(((&[0b11100001][..], 7), NonZeroU64::new(11).unwrap()))),
        case((&[0b11100111][..], 0), Ok(((&[0b11100111][..], 7), NonZeroU64::new(14).unwrap()))),
        case((&[0b11101000][..], 0), Ok(((&[][..], 0), NonZeroU64::new(15).unwrap()))),
        case((&[0b11101111][..], 0), Ok(((&[][..], 0), NonZeroU64::new(22).unwrap()))),
    )]
    fn test_read(
        stream: InputStream,
        expt_result: Result<(InputStream, NonZeroU64), IncompleteInt<NonZeroU64>>,
    ) {
        let calc_result = decode::read(stream);
        assert_eq!(calc_result, expt_result);
    }
}
