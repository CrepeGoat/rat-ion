use core::cmp::min;
use core::num::NonZeroUsize;
use core::ops::{AddAssign, RangeFrom, Shl, Shr};
use nom::{
    bits::streaming::take as take_bits, error::ParseError, Err, IResult, InputIter, InputLength,
    Needed, Slice, ToUsize,
};

pub fn take_partial<I, O, C, E: ParseError<(I, usize)>>(
    count: C,
) -> impl Fn((I, usize)) -> IResult<(I, usize), (O, Option<NonZeroUsize>), E>
where
    I: Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength + Copy,
    C: ToUsize + std::ops::Sub<usize, Output = C> + Copy,
    O: From<u8> + AddAssign + Shl<usize, Output = O> + Shr<usize, Output = O>,
{
    let take_count = take_bits(count);

    move |input: (I, usize)| match take_count(input) {
        Ok((input, result)) => Ok((input, (result, None))),
        Err(nom::Err::Incomplete(nom::Needed::Size(needed))) => {
            if let Ok((input, partial)) = take_bits::<_, O, _, E>(count - needed.get())(input) {
                Ok((input, ((partial << needed.get()), Some(needed))))
            } else {
                unreachable!();
            }
        }
        Err(e) => Err(e),
    }
}

pub(crate) fn take_align<I, E: ParseError<(I, usize)>>(
) -> impl Fn((I, usize)) -> IResult<(I, usize), (u8, usize), E>
where
    I: Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
{
    move |(input, bit_offset): (I, usize)| {
        let bitlen = (8usize - bit_offset) % 8usize;
        take_bits(bitlen)((input, bit_offset))
            .map(move |((input, bit_offset), bits)| ((input, bit_offset), (bits, bitlen)))
    }
}

pub(crate) fn take_zeros<I, C, E: ParseError<(I, usize)>>(
    max_count: C,
) -> impl Fn((I, usize)) -> IResult<(I, usize), usize, E>
where
    I: Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
    C: ToUsize,
{
    let max_count = max_count.to_usize();
    move |(mut input, bit_offset): (I, usize)| {
        if max_count == 0 {
            return Ok(((input, bit_offset), 0usize));
        }

        let mut streak_len: usize = 0;
        let mut item = input
            .iter_elements()
            .next()
            .ok_or_else(|| Err::Incomplete(Needed::new(1)))?;
        item &= 0xFF >> bit_offset; // mask out first `bit_offset` bits

        streak_len += (item.leading_zeros() as usize) - bit_offset;
        while item.leading_zeros() == 8 && streak_len <= max_count {
            input = input.slice(1..);
            if streak_len == max_count {
                break;
            };
            item = input
                .iter_elements()
                .next()
                .ok_or_else(|| Err::Incomplete(Needed::new(1)))?;
            streak_len += item.leading_zeros() as usize;
        }
        streak_len = min(streak_len, max_count);

        Ok(((input, (streak_len + bit_offset) % 8), streak_len))
    }
}

pub(crate) fn take_ones<I, C, E: ParseError<(I, usize)>>(
    max_count: C,
) -> impl Fn((I, usize)) -> IResult<(I, usize), usize, E>
where
    I: Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
    C: ToUsize,
{
    unimplemented!()
}
