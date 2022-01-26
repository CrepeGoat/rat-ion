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
            Less => self.checked_shr(shift as u32).unwrap_or_default(),
            Equal => self,
        }
    }

    #[inline]
    fn fp_ishr(self, shift: i32) -> Self {
        use core::cmp::Ordering::*;
        match shift.cmp(&0) {
            Greater => self.checked_shr(shift as u32).unwrap_or_default(),
            Less => self.checked_shl(shift as u32).unwrap_or_default(),
            Equal => self,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct BitwiseArray<U, const ANCHOR_LEFT: bool> {
    data: U,
    left_margin: u32,
    right_margin: u32,
    //_marker: std::marker::PhantomData<T>,
}

impl<U, const ANCHOR_LEFT: bool> BitwiseArray<U, ANCHOR_LEFT> {
    pub fn new(data: U, left_margin: u32, right_margin: u32) -> Self {
        assert!(left_margin + right_margin <= 8);
        Self {
            data,
            left_margin,
            right_margin,
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
        } else if ANCHOR_LEFT {
            self.right_margin += self.len() - len;
        } else {
            self.left_margin += self.len() - len;
        }
    }

    fn apply<F: FnOnce(u8, u8) -> u8, U2: Borrow<u8>, const ANCHOR_LEFT_2: bool>(
        mut self,
        mut other: BitwiseArray<U2, ANCHOR_LEFT_2>,
        func: F,
    ) -> BitwiseArray<u8, true>
    where
        U: Borrow<u8>,
    {
        self.trim(other.len());
        other.trim(self.len());

        BitwiseArray::<u8, true>::new(
            func(
                self.masked(),
                other
                    .masked()
                    .fp_ishl((other.left_margin as i32) - (self.left_margin as i32)),
            ),
            self.left_margin,
            self.right_margin,
        )
    }

    fn assign<F: FnOnce(u8, u8) -> u8, U2: Borrow<u8>, const ANCHOR_LEFT_2: bool>(
        &mut self,
        mut other: BitwiseArray<U2, ANCHOR_LEFT_2>,
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

impl<U: Borrow<u8>, const ANCHOR_LEFT: bool> From<&BitwiseArray<U, ANCHOR_LEFT>> for u8 {
    fn from(value: &BitwiseArray<U, ANCHOR_LEFT>) -> Self {
        value.masked().fp_shr(value.right_margin)
    }
}

impl<U1: Borrow<u8>, U2: Borrow<u8>, const ANCHOR_LEFT_1: bool, const ANCHOR_LEFT_2: bool>
    PartialEq<BitwiseArray<U2, ANCHOR_LEFT_2>> for BitwiseArray<U1, ANCHOR_LEFT_1>
{
    fn eq(&self, other: &BitwiseArray<U2, ANCHOR_LEFT_2>) -> bool {
        (self.len() == other.len()) & u8::from(self).eq(&u8::from(other))
    }
}

impl<U: Borrow<u8>, const ANCHOR_LEFT: bool> Eq for BitwiseArray<U, ANCHOR_LEFT> {}

impl<U1: Borrow<u8>, U2: Borrow<u8>, const ANCHOR_LEFT_1: bool, const ANCHOR_LEFT_2: bool>
    BitAnd<BitwiseArray<U2, ANCHOR_LEFT_2>> for BitwiseArray<U1, ANCHOR_LEFT_1>
{
    type Output = BitwiseArray<u8, true>;

    fn bitand(self, other: BitwiseArray<U2, ANCHOR_LEFT_2>) -> Self::Output {
        self.apply(other, BitAnd::bitand)
    }
}

impl<U1: Borrow<u8>, U2: Borrow<u8>, const ANCHOR_LEFT_1: bool, const ANCHOR_LEFT_2: bool>
    BitOr<BitwiseArray<U2, ANCHOR_LEFT_2>> for BitwiseArray<U1, ANCHOR_LEFT_1>
{
    type Output = BitwiseArray<u8, true>;

    fn bitor(self, other: BitwiseArray<U2, ANCHOR_LEFT_2>) -> Self::Output {
        self.apply(other, BitOr::bitor)
    }
}

impl<U1: Borrow<u8>, U2: Borrow<u8>, const ANCHOR_LEFT_1: bool, const ANCHOR_LEFT_2: bool>
    BitXor<BitwiseArray<U2, ANCHOR_LEFT_2>> for BitwiseArray<U1, ANCHOR_LEFT_1>
{
    type Output = BitwiseArray<u8, true>;

    fn bitxor(self, other: BitwiseArray<U2, ANCHOR_LEFT_2>) -> Self::Output {
        self.apply(other, BitXor::bitxor)
    }
}

impl<U1: BorrowMut<u8>, U2: Borrow<u8>, const ANCHOR_LEFT_1: bool, const ANCHOR_LEFT_2: bool>
    BitAndAssign<BitwiseArray<U2, ANCHOR_LEFT_2>> for BitwiseArray<U1, ANCHOR_LEFT_1>
{
    fn bitand_assign(&mut self, other: BitwiseArray<U2, ANCHOR_LEFT_2>) {
        self.assign(other, BitAnd::bitand)
    }
}

impl<U1: BorrowMut<u8>, U2: Borrow<u8>, const ANCHOR_LEFT_1: bool, const ANCHOR_LEFT_2: bool>
    BitOrAssign<BitwiseArray<U2, ANCHOR_LEFT_2>> for BitwiseArray<U1, ANCHOR_LEFT_1>
{
    fn bitor_assign(&mut self, other: BitwiseArray<U2, ANCHOR_LEFT_2>) {
        self.assign(other, BitOr::bitor)
    }
}

impl<U1: BorrowMut<u8>, U2: Borrow<u8>, const ANCHOR_LEFT_1: bool, const ANCHOR_LEFT_2: bool>
    BitXorAssign<BitwiseArray<U2, ANCHOR_LEFT_2>> for BitwiseArray<U1, ANCHOR_LEFT_1>
{
    fn bitxor_assign(&mut self, other: BitwiseArray<U2, ANCHOR_LEFT_2>) {
        self.assign(other, BitXor::bitxor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::*;

    proptest! {
        #[test]
        fn test_bitwise_masked(left_margin in 0_u32..=4, right_margin in 0_u32..=4) {
            let bits = BitwiseArray::<_, false>::new(0xFF, left_margin, right_margin);
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
            let bits1 = BitwiseArray::<_, false>::new(value << shift1, 4 - shift1, shift1);
            let bits2 = BitwiseArray::<_, true>::new(value << shift2, 4 - shift2, shift2);

            assert_eq!(bits1, bits2);
        }

        #[test]
        fn test_trim_left(value: u8, new_size in 0_u32..=8) {
            let mut bits = BitwiseArray::<_, true>::new(value, 0, 0);
            bits.trim(new_size);

            assert_eq!(u8::from(&bits), value.fp_shr(8 - new_size));
        }

        #[test]
        fn test_trim_right(value: u8, new_size in 0_u32..=8) {
            let mut bits = BitwiseArray::<_, false>::new(value, 0, 0);
            bits.trim(new_size);

            assert_eq!(u8::from(&bits), value.fp_shl(8 - new_size).fp_shr(8 - new_size));
        }
    }
}
