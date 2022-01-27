//use num::{PrimInt, Unsigned};
//
//struct BitwiseArray<T: PrimInt + Unsigned, U: AsRef<T>> {

use core::borrow::{Borrow, BorrowMut};
use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign};

trait FoolProofShift {
    fn fp_shl(self, shift: u32) -> Self;
    fn fp_shr(self, shift: u32) -> Self;
    fn fp_ishl(self, shift: i32) -> Self;
    fn fp_ishr(self, shift: i32) -> Self;
}

impl FoolProofShift for u8 {
    #[inline]
    fn fp_shl(self, shift: u32) -> Self {
        self.checked_shl(shift).unwrap_or_default()
    }

    #[inline]
    fn fp_shr(self, shift: u32) -> Self {
        self.checked_shr(shift).unwrap_or_default()
    }

    #[inline]
    fn fp_ishl(self, shift: i32) -> Self {
        use core::cmp::Ordering::*;
        match shift.cmp(&0) {
            Greater => self.checked_shl(shift as u32).unwrap_or_default(),
            Less => self.checked_shr((-shift) as u32).unwrap_or_default(),
            Equal => self,
        }
    }

    #[inline]
    fn fp_ishr(self, shift: i32) -> Self {
        use core::cmp::Ordering::*;
        match shift.cmp(&0) {
            Greater => self.checked_shr(shift as u32).unwrap_or_default(),
            Less => self.checked_shl((-shift) as u32).unwrap_or_default(),
            Equal => self,
        }
    }
}

pub trait TrimSide {
    const ANCHOR_LEFT: bool;
}

#[derive(Debug, Copy, Clone)]
struct TrimLeft;
#[derive(Debug, Copy, Clone)]
struct TrimRight;

impl TrimSide for TrimLeft {
    const ANCHOR_LEFT: bool = false;
}

impl TrimSide for TrimRight {
    const ANCHOR_LEFT: bool = true;
}

#[derive(Debug, Clone, Copy)]
struct BitwiseArray<U, S: TrimSide> {
    data: U,
    left_margin: u32,
    right_margin: u32,
    _side_marker: core::marker::PhantomData<S>,
    //_marker: core::marker::PhantomData<T>,
}

impl<U, S: TrimSide> BitwiseArray<U, S> {
    pub fn new(data: U, left_margin: u32, right_margin: u32) -> Self {
        assert!(left_margin + right_margin <= 8);
        Self {
            data,
            left_margin,
            right_margin,
            _side_marker: core::marker::PhantomData,
        }
    }

    pub fn len(&self) -> u32 {
        8 - self.left_margin - self.right_margin
    }

    fn mask(&self) -> u8 {
        u8::MAX.fp_shl(self.right_margin) & u8::MAX.fp_shr(self.left_margin)
    }

    fn masked(&self) -> u8
    where
        U: Borrow<u8>,
    {
        self.data.borrow() & self.mask()
    }

    fn trim(&mut self, len: u32) {
        if len >= self.len() {
        } else if S::ANCHOR_LEFT {
            self.right_margin += self.len() - len;
        } else {
            self.left_margin += self.len() - len;
        }
    }

    fn apply<F: FnOnce(u8, u8) -> u8, U2: Borrow<u8>, S2: TrimSide>(
        mut self,
        mut other: BitwiseArray<U2, S2>,
        func: F,
    ) -> BitwiseArray<u8, TrimLeft>
    where
        U: Borrow<u8>,
    {
        self.trim(other.len());
        other.trim(self.len());

        BitwiseArray::<u8, TrimLeft>::new(
            func(
                *self.data.borrow(),
                other
                    .data
                    .borrow()
                    .fp_ishr((self.left_margin as i32) - (other.left_margin as i32)),
            ),
            self.left_margin,
            self.right_margin,
        )
    }

    fn assign<F: FnOnce(u8, u8) -> u8, U2: Borrow<u8>, S2: TrimSide>(
        &mut self,
        mut other: BitwiseArray<U2, S2>,
        func: F,
    ) where
        U: BorrowMut<u8>,
    {
        self.trim(other.len());
        other.trim(self.len());

        *self.data.borrow_mut() = (*self.data.borrow() & !self.mask())
            & (func(
                *self.data.borrow(),
                other
                    .data
                    .borrow()
                    .fp_ishl((other.left_margin as i32) - (self.left_margin as i32)),
            ) & self.mask())
    }
}

impl<U: Borrow<u8>, S: TrimSide> From<&BitwiseArray<U, S>> for u8 {
    fn from(value: &BitwiseArray<U, S>) -> Self {
        value.masked().fp_shr(value.right_margin)
    }
}

impl<U1: Borrow<u8>, U2: Borrow<u8>, S1: TrimSide, S2: TrimSide> PartialEq<BitwiseArray<U2, S2>>
    for BitwiseArray<U1, S1>
{
    fn eq(&self, other: &BitwiseArray<U2, S2>) -> bool {
        (self.len() == other.len()) & u8::from(self).eq(&u8::from(other))
    }
}

impl<U: Borrow<u8>, S: TrimSide> Eq for BitwiseArray<U, S> {}

impl<U1: Borrow<u8>, U2: Borrow<u8>, S1: TrimSide, S2: TrimSide> BitAnd<BitwiseArray<U2, S2>>
    for BitwiseArray<U1, S1>
{
    type Output = BitwiseArray<u8, TrimLeft>;

    fn bitand(self, other: BitwiseArray<U2, S2>) -> Self::Output {
        self.apply(other, BitAnd::bitand)
    }
}

impl<U1: Borrow<u8>, U2: Borrow<u8>, S1: TrimSide, S2: TrimSide> BitOr<BitwiseArray<U2, S2>>
    for BitwiseArray<U1, S1>
{
    type Output = BitwiseArray<u8, TrimLeft>;

    fn bitor(self, other: BitwiseArray<U2, S2>) -> Self::Output {
        self.apply(other, BitOr::bitor)
    }
}

impl<U1: Borrow<u8>, U2: Borrow<u8>, S1: TrimSide, S2: TrimSide> BitXor<BitwiseArray<U2, S2>>
    for BitwiseArray<U1, S1>
{
    type Output = BitwiseArray<u8, TrimLeft>;

    fn bitxor(self, other: BitwiseArray<U2, S2>) -> Self::Output {
        self.apply(other, BitXor::bitxor)
    }
}

impl<U1: BorrowMut<u8>, U2: Borrow<u8>, S1: TrimSide, S2: TrimSide>
    BitAndAssign<BitwiseArray<U2, S2>> for BitwiseArray<U1, S1>
{
    fn bitand_assign(&mut self, other: BitwiseArray<U2, S2>) {
        self.assign(other, BitAnd::bitand)
    }
}

impl<U1: BorrowMut<u8>, U2: Borrow<u8>, S1: TrimSide, S2: TrimSide>
    BitOrAssign<BitwiseArray<U2, S2>> for BitwiseArray<U1, S1>
{
    fn bitor_assign(&mut self, other: BitwiseArray<U2, S2>) {
        self.assign(other, BitOr::bitor)
    }
}

impl<U1: BorrowMut<u8>, U2: Borrow<u8>, S1: TrimSide, S2: TrimSide>
    BitXorAssign<BitwiseArray<U2, S2>> for BitwiseArray<U1, S1>
{
    fn bitxor_assign(&mut self, other: BitwiseArray<U2, S2>) {
        self.assign(other, BitXor::bitxor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::*;

    proptest! {
        #[test]
        fn test_fp_shl(value: u8, lshift in 0_u32..8) {
            assert_eq!(value.fp_shl(lshift), value << lshift);
        }

        #[test]
        fn test_fp_shl_8(value: u8) {
            assert_eq!(value.fp_shl(8), 0);
        }

        #[test]
        fn test_fp_shr(value: u8, rshift in 0_u32..8) {
            assert_eq!(value.fp_shr(rshift), value >> rshift);
        }

        #[test]
        fn test_fp_shr_8(value: u8) {
            assert_eq!(value.fp_shr(8), 0);
        }

        #[test]
        fn test_fp_ishl(value: u8, lshift in 0_i32..8) {
            assert_eq!(value.fp_ishl(lshift), value << (lshift as u32));
        }

        #[test]
        fn test_fp_ishl_neg(value: u8, lshift in -7_i32..=0) {
            assert_eq!(value.fp_ishl(lshift), value >> ((-lshift) as u32));
        }

        #[test]
        fn test_fp_ishl_8(value: u8) {
            assert_eq!(value.fp_ishl(8_i32), 0);
        }

        #[test]
        fn test_fp_ishl_neg8(value: u8) {
            assert_eq!(value.fp_ishl(-8_i32), 0);
        }

        #[test]
        fn test_fp_ishr(value: u8, rshift in 0_i32..8) {
            assert_eq!(value.fp_ishr(rshift), value >> (rshift as u32));
        }

        #[test]
        fn test_fp_ishr_neg(value: u8, rshift in -7_i32..=0) {
            assert_eq!(value.fp_ishr(rshift), value << ((-rshift) as u32));
        }

        #[test]
        fn test_fp_ishr_8(value: u8) {
            assert_eq!(value.fp_ishr(8_i32), 0);
        }

        #[test]
        fn test_fp_ishr_neg8(value: u8) {
            assert_eq!(value.fp_ishr(-8_i32), 0);
        }

        #[test]
        fn test_bitwise_masked(left_margin in 0_u32..=4, right_margin in 0_u32..=4) {
            let bits = BitwiseArray::<_, TrimRight>::new(0xFF, left_margin, right_margin);
            let calc_result = bits.masked();

            println!("masked bits = {:?}", calc_result);
            assert_eq!(calc_result.count_ones(), 8 - left_margin - right_margin);
            if calc_result.count_ones() != 0 {
                assert_eq!(calc_result.leading_zeros(), left_margin);
                assert_eq!(calc_result.trailing_zeros(), right_margin);
            }
        }

        #[test]
        fn test_bitwise_array_eq(value in 0x00_u8..0x10, shift1 in 0_u32..=4, shift2 in 0_u32..=4) {
            let bits1 = BitwiseArray::<_, TrimRight>::new(value << shift1, 4 - shift1, shift1);
            let bits2 = BitwiseArray::<_, TrimLeft>::new(value << shift2, 4 - shift2, shift2);

            assert_eq!(bits1, bits2);
        }

        #[test]
        fn test_trim_left(value: u8, new_size in 0_u32..=8) {
            let mut bits = BitwiseArray::<_, TrimLeft>::new(value, 0, 0);
            bits.trim(new_size);

            assert_eq!(u8::from(&bits), value.fp_shl(8 - new_size).fp_shr(8 - new_size));
        }

        #[test]
        fn test_trim_right(value: u8, new_size in 0_u32..=8) {
            let mut bits = BitwiseArray::<_, TrimRight>::new(value, 0, 0);
            bits.trim(new_size);

            assert_eq!(u8::from(&bits), value.fp_shr(8 - new_size));
        }

        #[test]
        fn test_bitand(value1 in 0_u8..0x10, lshift1 in 0_u32..=4, value2 in 0_u8..0x40, lshift2 in 0_u32..=2) {
            let value1 = value1 << lshift1;
            let value2 = value2 << lshift2;

            let bits1 = BitwiseArray::<_, TrimLeft>::new(&value1, 4 - lshift1, lshift1);
            let bits2 = BitwiseArray::<_, TrimLeft>::new(value2, 2 - lshift2, lshift2);
            assert_eq!((bits1.len(), bits2.len()), (4, 6));
            assert_eq!((u8::from(&bits1), u8::from(&bits2)), (value1 >> lshift1, value2 >> lshift2));

            let result = bits1 & bits2;
            assert_eq!(result.len(), 4);
            assert_eq!(u8::from(&result), ((value1 >> lshift1) & (value2 >> lshift2)) & (0xFF_u8 >> 4));
        }

        #[test]
        fn test_bitor(value1 in 0_u8..0x10, lshift1 in 0_u32..=4, value2 in 0_u8..0x40, lshift2 in 0_u32..=2) {
            let value1 = value1 << lshift1;
            let value2 = value2 << lshift2;

            let bits1 = BitwiseArray::<_, TrimLeft>::new(value1, 4 - lshift1, lshift1);
            let bits2 = BitwiseArray::<_, TrimRight>::new(&value2, 2 - lshift2, lshift2);
            assert_eq!((bits1.len(), bits2.len()), (4, 6));
            assert_eq!((u8::from(&bits1), u8::from(&bits2)), (value1 >> lshift1, value2 >> lshift2));

            let result = bits1 | bits2;
            assert_eq!(result.len(), 4);
            assert_eq!(u8::from(&result), ((value1 >> lshift1) | (value2 >> (lshift2 + 2))) & (0xFF_u8 >> 4));
        }

        #[test]
        fn test_bitxor(value1 in 0_u8..0x10, lshift1 in 0_u32..=4, value2 in 0_u8..0x40, lshift2 in 0_u32..=2) {
            let value1 = value1 << lshift1;
            let value2 = value2 << lshift2;

            let bits1 = BitwiseArray::<_, TrimLeft>::new(value1, 4 - lshift1, lshift1);
            let bits2 = BitwiseArray::<_, TrimRight>::new(value2, 2 - lshift2, lshift2);
            assert_eq!((bits1.len(), bits2.len()), (4, 6));
            assert_eq!((u8::from(&bits1), u8::from(&bits2)), (value1 >> lshift1, value2 >> lshift2));

            let result = bits2 ^ bits1;
            assert_eq!(result.len(), 4);
            assert_eq!(u8::from(&result), ((value1 >> lshift1) ^ (value2 >> (lshift2 + 2))) & (0xFF_u8 >> 4));
        }
    }
}
