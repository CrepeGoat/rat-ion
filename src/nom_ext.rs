use core::cmp::min;
use core::ops::RangeFrom;
use nom::{
    bits::streaming::take as take_bits, error::ParseError, Err, IResult, InputIter, InputLength,
    Needed, Slice, ToUsize,
};

pub(crate) fn take_align<I, E: ParseError<(I, usize)>>(
) -> impl Fn((I, usize)) -> IResult<I, (u8, usize), E>
where
    I: Slice<RangeFrom<usize>> + InputIter<Item = u8> + InputLength,
{
    move |(input, bit_offset): (I, usize)| {
        let bitlen = (8usize - bit_offset) % 8usize;
        take_bits(bitlen)((input, bit_offset))
            .map(move |((input, _bit_offset), bits)| (input, (bits, bitlen)))
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
            .ok_or(Err::Incomplete(Needed::Unknown))?;
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
                .ok_or(Err::Incomplete(Needed::Unknown))?;
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
    let max_count = max_count.to_usize();
    move |(mut input, bit_offset): (I, usize)| {
        if max_count == 0 {
            return Ok(((input, bit_offset), 0usize));
        }

        let mut streak_len: usize = 0;
        let mut item = input
            .iter_elements()
            .next()
            .ok_or(Err::Incomplete(Needed::Unknown))?;
        item |= !(0xFF >> bit_offset); // mask out first `bit_offset` bits

        streak_len += (item.leading_ones() as usize) - bit_offset;
        while item.leading_ones() == 8 && streak_len <= max_count {
            input = input.slice(1..);
            if streak_len == max_count {
                break;
            };
            item = input
                .iter_elements()
                .next()
                .ok_or(Err::Incomplete(Needed::Unknown))?;
            streak_len += item.leading_ones() as usize;
        }
        streak_len = min(streak_len, max_count);

        Ok(((input, (streak_len + bit_offset) % 8), streak_len))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_take_zeros(streak_len in 0_usize..56, bit_offset in 0_usize..8) {
            let source: u64 = !(!(u64::MAX >> streak_len) >> bit_offset);
            let source_bytes = source.to_be_bytes();
            let input = (&source_bytes[..], bit_offset);

            let (_input, calc_result) = take_zeros::<_, _, ()>(usize::MAX)(input).unwrap();
            assert_eq!(calc_result, streak_len);
        }

        #[test]
        fn test_take_ones(streak_len in 0_usize..56, bit_offset in 0_usize..8) {
            let source: u64 = !(u64::MAX >> streak_len) >> bit_offset;
            let source_bytes = source.to_be_bytes();
            let input = (&source_bytes[..], bit_offset);

            let (_input, calc_result) = take_ones::<_, _, ()>(usize::MAX)(input).unwrap();
            assert_eq!(calc_result, streak_len);
        }

        #[test]
        fn test_take_zeros_limit(streak_limit in 0_usize..56, bit_offset in 0_usize..8) {
            let source: u64 = !(u64::MAX >> bit_offset);
            let source_bytes = source.to_be_bytes();
            let input = (&source_bytes[..], bit_offset);

            let (_input, calc_result) = take_zeros::<_, _, ()>(streak_limit)(input).unwrap();
            assert_eq!(calc_result, streak_limit);
        }

        #[test]
        fn test_take_ones_limit(streak_limit in 0_usize..56, bit_offset in 0_usize..8) {
            let source: u64 = u64::MAX >> bit_offset;
            let source_bytes = source.to_be_bytes();
            let input = (&source_bytes[..], bit_offset);

            let (_input, calc_result) = take_ones::<_, _, ()>(streak_limit)(input).unwrap();
            assert_eq!(calc_result, streak_limit);
        }
    }
}
