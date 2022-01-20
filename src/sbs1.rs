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
    use crate::sbs_utils;

    pub(crate) fn skip() -> bool {
        unimplemented!()
    }

    pub(crate) fn read(
        stream: (&[u8], usize),
    ) -> Result<((&[u8], usize), NonZeroU64), Option<(u64, NonZeroUsize)>> {
        let (stream, first_bit) = take::<_, u8, _, ()>(1_usize)(stream).or(Err(None))?;
        if first_bit == 0 {
            Ok((stream, unsafe { NonZeroU64::new_unchecked(1) }))
        } else {
            match sbs_utils::decode::read(stream) {
                Ok((stream, result)) => Ok((stream, NonZeroU64::new(result.get() - 1).unwrap())),
                Err(Some((partial, needed))) => Err(Some((unimplemented!(), needed))),
                Err(None) => Err(None),
            }
        }
    }
}
