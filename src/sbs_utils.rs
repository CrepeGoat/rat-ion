use crate::nom_ext::take_ones;
use crate::nom_mod::take_partial;
use nom::{bits::streaming::take, sequence::terminated};

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

    pub(crate) fn skip() -> bool {
        unimplemented!()
    }

    pub(crate) fn read(
        stream: (&[u8], usize),
    ) -> Result<((&[u8], usize), NonZeroU64), Option<(u64, NonZeroUsize)>> {
        // Get prefixing ones stream
        let (stream, ones_len) =
            terminated(take_ones(usize::MAX), take::<_, u8, _, ()>(1_usize))(stream)
                .or(Err(None))?;

        // Get first literal digit bit -> determines result's MSBs
        let digits_len = ones_len + 2;
        let (_, first_digit) = take::<_, u8, _, ()>(1_usize)(stream)
            // if no more bits left, assume largest (why largest? to be conservative?)
            .or(Err(Some((0, NonZeroUsize::new(digits_len).unwrap()))))?;
        let second_msb = 1 - first_digit;
        let digits_len = digits_len - (second_msb as usize);
        let leading_bits = (2 + (second_msb as u64)) << digits_len;

        match take_partial::<u64>(digits_len)(stream) {
            Ok((stream, result)) => Ok((stream, NonZeroU64::new(result + leading_bits).unwrap())),
            Err((partial, needed)) => Err(Some((partial + leading_bits, needed))),
        }
    }
}
