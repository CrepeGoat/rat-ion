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
