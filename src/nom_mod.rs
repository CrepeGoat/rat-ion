use core::num::NonZeroUsize;
use nom::bits::streaming::take as nom_take;

use nom::lib::std::ops::{AddAssign, Shl, Shr};

type IResult<I, O, E> = Result<(I, O), E>;

pub fn take_partial<'a, O>(
    count: usize,
) -> impl Fn((&'a [u8], usize)) -> IResult<(&'a [u8], usize), O, Option<(O, NonZeroUsize)>>
where
    O: From<u8> + AddAssign + Shl<usize, Output = O> + Shr<usize, Output = O>,
{
    let nom_take_count = nom_take::<_, _, _, ()>(count);

    move |input: (&[u8], usize)| {
        nom_take_count(input).map_err(|e| {
            if let nom::Err::Incomplete(nom::Needed::Size(needed)) = e {
                Some((
                    nom_take::<_, _, _, ()>(count - needed.get())(input)
                        .expect("unreachable")
                        .1
                        .shl(needed.get()),
                    needed,
                ))
            } else {
                unreachable!();
            }
        })
    }
}
