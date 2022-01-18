use core::cmp::min;
use core::ops::RangeFrom;
use nom::{
    bits::streaming::take as take_bits, error::ParseError, Err, IResult, InputIter, InputLength,
    Needed, Slice, ToUsize,
};

fn take_rem<I, E: ParseError<(I, usize)>>(
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

fn take_zeros<I, C, E: ParseError<(I, usize)>>(
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
