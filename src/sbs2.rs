use crate::sbs_utils;
use crate::utils::{IncompleteInt, InputStream};
use core::num::{NonZeroU64, NonZeroUsize};
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

    pub(crate) fn skip() -> bool {
        unimplemented!()
    }

    pub(crate) fn read(
        stream: (&[u8], usize),
    ) -> Result<(InputStream, NonZeroU64), IncompleteInt<NonZeroU64>> {
        let (stream, first_bit) = take::<_, u8, _, ()>(1_usize)(stream)
            .map_err(|_| IncompleteInt::new_unbounded(unsafe { NonZeroU64::new_unchecked(1) }))?;
        if first_bit == 0 {
            match take::<_, u64, _, ()>(1_usize)(stream) {
                Ok((stream, x)) => Ok((stream, unsafe { NonZeroU64::new_unchecked(x + 1) })),
                Err(_) => Err(unsafe {
                    IncompleteInt::new_bounded(
                        (NonZeroU64::new_unchecked(1), NonZeroU64::new_unchecked(2)),
                        NonZeroUsize::new_unchecked(1),
                    )
                }),
            }
        } else {
            sbs_utils::decode::read(stream)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest(stream, expt_result,
        case((&[0b00111111][..], 0), Ok(((&[0b00111111][..], 2), NonZeroU64::new(1).unwrap()))),
        case((&[0b01111111][..], 0), Ok(((&[0b01111111][..], 2), NonZeroU64::new(2).unwrap()))),
        case((&[0b10011111][..], 0), Ok(((&[0b10011111][..], 3), NonZeroU64::new(3).unwrap()))),
        case((&[0b10101111][..], 0), Ok(((&[0b10101111][..], 4), NonZeroU64::new(4).unwrap()))),
        case((&[0b10111111][..], 0), Ok(((&[0b10111111][..], 4), NonZeroU64::new(5).unwrap()))),
        case((&[0b11000111][..], 0), Ok(((&[0b11000111][..], 5), NonZeroU64::new(6).unwrap()))),
        case((&[0b11001111][..], 0), Ok(((&[0b11001111][..], 5), NonZeroU64::new(7).unwrap()))),
        case((&[0b11010011][..], 0), Ok(((&[0b11010011][..], 6), NonZeroU64::new(8).unwrap()))),
        case((&[0b11011111][..], 0), Ok(((&[0b11011111][..], 6), NonZeroU64::new(11).unwrap()))),
        case((&[0b11100001][..], 0), Ok(((&[0b11100001][..], 7), NonZeroU64::new(12).unwrap()))),
        case((&[0b11100111][..], 0), Ok(((&[0b11100111][..], 7), NonZeroU64::new(15).unwrap()))),
        case((&[0b11101000][..], 0), Ok(((&[][..], 0), NonZeroU64::new(16).unwrap()))),
        case((&[0b11101111][..], 0), Ok(((&[][..], 0), NonZeroU64::new(23).unwrap()))),
    )]
    fn test_read(
        stream: InputStream,
        expt_result: Result<(InputStream, NonZeroU64), IncompleteInt<NonZeroU64>>,
    ) {
        let calc_result = decode::read(stream);
        assert_eq!(calc_result, expt_result);
    }
}
