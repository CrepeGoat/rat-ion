use crate::nom_ext::*;
use nom::{bits::streaming::take, IResult, Needed};

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

    pub(crate) fn read(stream: (&[u8], usize)) -> IResult<(&[u8], usize), u64, (u64, Needed)> {
        let (stream, first_bit) = take(1_usize)(stream).map_err(unimplemented!())?;
        if first_bit == 0 {
            take(1).map(|x| x + 1)(stream).map_err(unimplemented!())
        } else {
            decode::read(stream)
        }
    }
}
