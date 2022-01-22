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

    pub(crate) fn skip() -> bool {
        unimplemented!()
    }

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
