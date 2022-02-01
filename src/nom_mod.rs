use core::num::NonZeroUsize;
use nom::bits::streaming::take as nom_take;

use nom::lib::std::ops::{AddAssign, Shl, Shr};

use crate::utils::OutputStream;

type IResult<I, O, E> = Result<(I, O), E>;

pub fn take_partial<'a, O>(
    count: usize,
) -> impl Fn((&'a [u8], usize)) -> IResult<(&'a [u8], usize), O, (O, NonZeroUsize)>
where
    O: From<u8> + AddAssign + Shl<usize, Output = O> + Shr<usize, Output = O>,
{
    let nom_take_count = nom_take::<_, _, _, ()>(count);

    move |input: (&[u8], usize)| {
        nom_take_count(input).map_err(|e| {
            if let nom::Err::Incomplete(nom::Needed::Size(needed)) = e {
                (
                    nom_take::<_, O, _, ()>(count - needed.get())(input)
                        .expect("unreachable")
                        .1,
                    needed,
                )
            } else {
                unreachable!();
            }
        })
    }
}

pub fn give8(
    (output, bit_offset): OutputStream,
    (source, length): (u8, usize),
) -> nom::IResult<OutputStream, (u8, usize), ()> {
    assert!(bit_offset < 8);
    assert!(length <= 8);

    if length == 0 {
        return Ok(((output, bit_offset), (0, length)));
    }
    if output.is_empty() {
        return Err(nom::Err::Incomplete(nom::Needed::Size(
            NonZeroUsize::new(length).unwrap(),
        )));
    }

    let output_rem = 8 - bit_offset;
    let len_write = core::cmp::min(length, output_rem);
    let source_rem = length - len_write;
    let source_write = source >> source_rem;
    let output_mask =
        !0xFF_u8.checked_shl(len_write as u32).unwrap_or_default() << (output_rem - len_write);

    output[0] ^= (output[0] & output_mask) ^ (source_write << (output_rem - len_write));

    Ok((
        (
            &mut output[(bit_offset + len_write) / 8..],
            (bit_offset + len_write) % 8,
        ),
        (
            source
                & 0xFF_u8
                    .checked_shr(8 - (source_rem as u32))
                    .unwrap_or_default(),
            source_rem,
        ),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_give8_1(bit_offset in 0_usize..8, source: u8) {
            let mut buffer = [0_u8; 1];
            let (_, calc_result) = give8((&mut buffer[..], bit_offset), (source, 8)).unwrap();

            assert_eq!(calc_result, (source & !(0xFF_u8 << bit_offset), bit_offset));
            assert_eq!(buffer, [source >> bit_offset]);
        }

        #[test]
        fn test_give8_2(source: u8, bitlen in 0_usize..8) {
            let mut buffer = [0_u8; 1];
            let (_, calc_result) = give8((&mut buffer[..], 0), (source, bitlen)).unwrap();
            assert_eq!(buffer, [source.checked_shl(8 - bitlen as u32).unwrap_or_default()]);
            assert_eq!(calc_result, (0, 0));
        }
    }
}
