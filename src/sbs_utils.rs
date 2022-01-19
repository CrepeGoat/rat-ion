use crate::nom_ext::*;
use nom::{bits::streaming::take, sequence::terminated, IResult};

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
    use core::num::NonZeroUsize;

    pub(crate) fn skip() -> bool {
        unimplemented!()
    }

    pub(crate) fn read(
        stream: (&[u8], usize),
    ) -> IResult<(&[u8], usize), Option<(u64, NonZeroUsize)>, ()> {
        // Get prefixing ones stream
        let (stream, ones_len) =
            terminated(take_ones(usize::MAX), take(1_usize))(stream).map_err(unimplemented!())?;

        // Get first literal digit bit -> determines result's MSBs
        let (_, first_digit): (_, u8) = take(1_usize)(stream).map_err(unimplemented!())?;
        let second_msb = 1 - first_digit;
        let digits_len = ones_len + (first_digit as usize) + 1;

        let (stream, mut result): (_, u64) = take(digits_len)(stream).map_err(unimplemented!())?;
        result += (2 + (second_msb as u64)) << digits_len;

        Ok((stream, result))
    }
}
