use crate::nom_ext::take_ones;
use crate::nom_mod::take_partial;
use crate::utils::{IncompleteInt, InputStream};
use nom::bits::streaming::take;

use core::num::{NonZeroU64, NonZeroUsize};
use core::ops::{RangeFrom, RangeInclusive};

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
