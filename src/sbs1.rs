use crate::nom_ext::*;
use nom::{
    bits::streaming::take as take_bits, bytes::streaming::take as take_bytes, error::ParseError,
    Err, IResult, InputIter, InputLength, Needed, Slice, ToUsize,
};

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

mod decode {
    use super::*;

    fn skip() -> bool {
        unimplemented!()
    }

    fn peek() -> u64 {
        unimplemented!()
    }
    fn read((stream, offset): (&[u8], usize)) -> IResult<(&[u8], usize), u64, (u64, usize)> {
        unimplemented!()
    }
}
