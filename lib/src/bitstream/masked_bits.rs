//use num::{PrimInt, Unsigned};
//
//struct MaskedBits<T: PrimInt + Unsigned, U: AsRef<T>> {

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
            Greater => self.fp_shl(shift as u32),
            Less => self.fp_shr((-shift) as u32),
            Equal => self,
        }
    }

    #[inline]
    fn fp_ishr(self, shift: i32) -> Self {
        use core::cmp::Ordering::*;
        match shift.cmp(&0) {
            Greater => self.fp_shr(shift as u32),
            Less => self.fp_shl((-shift) as u32),
            Equal => self,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MaskedBits<U: Borrow<u8>> {
    data: U,
    left_margin: u32,
    right_margin: u32,
    //_marker: core::marker::PhantomData<T>,
}

impl<U: Borrow<u8>> MaskedBits<U> {
    pub fn new(data: U, left_margin: u32, right_margin: u32) -> Self {
        assert!(left_margin + right_margin <= 8);
        Self {
            data,
            left_margin,
            right_margin,
        }
    }

    pub fn new_leading(data: U, len: u32) -> Self {
        Self::new(data, 0, 8 - len)
    }

    pub fn new_trailing(data: U, len: u32) -> Self {
        Self::new(data, 8 - len, 0)
    }

    pub fn len(&self) -> u32 {
        8 - self.left_margin - self.right_margin
    }

    #[inline]
    fn mask_left(&self) -> u8 {
        u8::MAX.fp_shr(self.left_margin)
    }

    #[inline]
    fn mask_right(&self) -> u8 {
        u8::MAX.fp_shl(self.right_margin)
    }

    #[inline]
    fn mask(&self) -> u8 {
        self.mask_left() & self.mask_right()
    }

    fn masked(&self) -> u8
    where
        U: Borrow<u8>,
    {
        self.data.borrow() & self.mask()
    }

    pub fn trim_leading_to(mut self, len: u32) -> Self {
        if len >= self.len() {
        } else {
            self.left_margin += self.len() - len;
        }

        self
    }

    pub fn trim_trailing_to(mut self, len: u32) -> Self {
        if len >= self.len() {
        } else {
            self.right_margin += self.len() - len;
        }

        self
    }

    pub fn split_leading_at(&self, len: u32) -> (Self, Self)
    where
        U: Copy,
    {
        assert!(len <= self.len());

        (
            self.trim_trailing_to(len),
            self.trim_leading_to(self.len() - len),
        )
    }

    pub fn split_trailing_at(&self, len: u32) -> (Self, Self)
    where
        U: Copy,
    {
        assert!(len <= self.len());

        (
            self.trim_trailing_to(self.len() - len),
            self.trim_leading_to(len),
        )
    }

    pub unsafe fn split_leading_at_mut(&mut self, _len: u32) -> (Self, Self)
    where
        U: BorrowMut<u8>,
    {
        todo!()
    }

    pub unsafe fn split_trailing_at_mut(&mut self, _len: u32) -> (Self, Self)
    where
        U: BorrowMut<u8>,
    {
        todo!()
    }

    fn apply<F: FnOnce(u8, u8) -> u8, U2: Borrow<u8>>(
        self,
        other: MaskedBits<U2>,
        func: F,
    ) -> MaskedBits<u8> {
        assert_eq!(self.len(), other.len());

        MaskedBits::new(
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

    pub fn assign<U2: Borrow<u8>>(&mut self, other: MaskedBits<U2>)
    where
        U: BorrowMut<u8>,
    {
        assert_eq!(self.len(), other.len());

        *self.data.borrow_mut() = (*self.data.borrow() & !self.mask())
            | (other
                .data
                .borrow()
                .fp_ishr((self.left_margin as i32) - (other.left_margin as i32))
                & self.mask())
    }

    fn apply_assign<F: FnOnce(u8, u8) -> u8, U2: Borrow<u8>>(
        &mut self,
        other: MaskedBits<U2>,
        func: F,
    ) where
        U: BorrowMut<u8>,
    {
        assert_eq!(self.len(), other.len());

        *self.data.borrow_mut() = (*self.data.borrow() & !self.mask())
            | (func(
                *self.data.borrow(),
                other
                    .data
                    .borrow()
                    .fp_ishr((self.left_margin as i32) - (other.left_margin as i32)),
            ) & self.mask())
    }

    pub fn leading_zeros(&self) -> u32 {
        ((self.data.borrow() & self.mask_left()) | !self.mask_right()).leading_zeros()
            - self.left_margin
    }

    pub fn leading_ones(&self) -> u32 {
        ((self.data.borrow() | !self.mask_left()) & self.mask_right()).leading_ones()
            - self.left_margin
    }

    pub fn trailing_zeros(&self) -> u32 {
        ((self.data.borrow() & self.mask_right()) | !self.mask_left()).trailing_zeros()
            - self.right_margin
    }

    pub fn trailing_ones(&self) -> u32 {
        ((self.data.borrow() | !self.mask_right()) & self.mask_left()).trailing_ones()
            - self.right_margin
    }
}

impl<U: Borrow<u8>> From<&MaskedBits<U>> for u8 {
    fn from(value: &MaskedBits<U>) -> Self {
        value.masked().fp_shr(value.right_margin)
    }
}

impl<U1: Borrow<u8>, U2: Borrow<u8>> PartialEq<MaskedBits<U2>> for MaskedBits<U1> {
    fn eq(&self, other: &MaskedBits<U2>) -> bool {
        (self.len() == other.len()) & u8::from(self).eq(&u8::from(other))
    }
}

impl<U: Borrow<u8>> Eq for MaskedBits<U> {}

impl<U1: Borrow<u8>, U2: Borrow<u8>> BitAnd<MaskedBits<U2>> for MaskedBits<U1> {
    type Output = MaskedBits<u8>;

    fn bitand(self, other: MaskedBits<U2>) -> Self::Output {
        self.apply(other, BitAnd::bitand)
    }
}

impl<U1: Borrow<u8>, U2: Borrow<u8>> BitOr<MaskedBits<U2>> for MaskedBits<U1> {
    type Output = MaskedBits<u8>;

    fn bitor(self, other: MaskedBits<U2>) -> Self::Output {
        self.apply(other, BitOr::bitor)
    }
}

impl<U1: Borrow<u8>, U2: Borrow<u8>> BitXor<MaskedBits<U2>> for MaskedBits<U1> {
    type Output = MaskedBits<u8>;

    fn bitxor(self, other: MaskedBits<U2>) -> Self::Output {
        self.apply(other, BitXor::bitxor)
    }
}

impl<U1: BorrowMut<u8>, U2: Borrow<u8>> BitAndAssign<MaskedBits<U2>> for MaskedBits<U1> {
    fn bitand_assign(&mut self, other: MaskedBits<U2>) {
        self.apply_assign(other, BitAnd::bitand)
    }
}

impl<U1: BorrowMut<u8>, U2: Borrow<u8>> BitOrAssign<MaskedBits<U2>> for MaskedBits<U1> {
    fn bitor_assign(&mut self, other: MaskedBits<U2>) {
        self.apply_assign(other, BitOr::bitor)
    }
}

impl<U1: BorrowMut<u8>, U2: Borrow<u8>> BitXorAssign<MaskedBits<U2>> for MaskedBits<U1> {
    fn bitxor_assign(&mut self, other: MaskedBits<U2>) {
        self.apply_assign(other, BitXor::bitxor)
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
            let bits = MaskedBits::new(0xFF, left_margin, right_margin);
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
            let bits1 = MaskedBits::new(value << shift1, 4 - shift1, shift1);
            let bits2 = MaskedBits::new(value << shift2, 4 - shift2, shift2);

            assert_eq!(bits1, bits2);
        }

        #[test]
        fn test_trim_leading_to(value: u8, new_size in 0_u32..=8) {
            let mut bits = MaskedBits::new(value, 0, 0);
            bits = bits.trim_leading_to(new_size);

            assert_eq!(u8::from(&bits), value.fp_shl(8 - new_size).fp_shr(8 - new_size));
        }

        #[test]
        fn test_trim_trailing_to(value: u8, new_size in 0_u32..=8) {
            let mut bits = MaskedBits::new(value, 0, 0);
            bits = bits.trim_trailing_to(new_size);

            assert_eq!(u8::from(&bits), value.fp_shr(8 - new_size));
        }

        #[test]
        fn test_leading_zeros(value in 0_u8..0x10, shift_left in 0_u32..=4) {
            let bits = MaskedBits::new(value << shift_left, 4 - shift_left, shift_left);

            assert_eq!(bits.leading_zeros(), value.leading_zeros() - 4);
        }

        #[test]
        fn test_leading_ones(value in 0_u8..0x10, shift_left in 0_u32..=4) {
            let bits = MaskedBits::new(value << shift_left, 4 - shift_left, shift_left);

            assert_eq!(bits.leading_ones(), (value | (0xFF << 4)).leading_ones() - 4);
        }

        #[test]
        fn test_trailing_zeros(value in 0_u8..0x10, shift_left in 0_u32..=4) {
            let bits = MaskedBits::new(value << shift_left, 4 - shift_left, shift_left);

            assert_eq!(bits.trailing_zeros(), (value | (0xFF << 4)).trailing_zeros());
        }

        #[test]
        fn test_trailing_ones(value in 0_u8..0x10, shift_left in 0_u32..=4) {
            let bits = MaskedBits::new(value << shift_left, 4 - shift_left, shift_left);

            assert_eq!(bits.trailing_ones(), value.trailing_ones());
        }

        #[test]
        fn test_bitand(value1 in 0_u8..0x10, lshift1 in 0_u32..=4, value2 in 0_u8..0x10, lshift2 in 0_u32..=4) {
            let value1 = value1 << lshift1;
            let value2 = value2 << lshift2;

            let bits1 = MaskedBits::new(&value1, 4 - lshift1, lshift1);
            let bits2 = MaskedBits::new(value2, 4 - lshift2, lshift2);
            assert_eq!((bits1.len(), bits2.len()), (4, 4));
            assert_eq!((u8::from(&bits1), u8::from(&bits2)), (value1 >> lshift1, value2 >> lshift2));

            let result = bits1 & bits2;
            assert_eq!(result.len(), 4);
            assert_eq!(u8::from(&result), ((value1 >> lshift1) & (value2 >> lshift2)) & (0xFF_u8 >> 4));
        }

        #[test]
        fn test_bitor(value1 in 0_u8..0x10, lshift1 in 0_u32..=4, value2 in 0_u8..0x10, lshift2 in 0_u32..=4) {
            let value1 = value1 << lshift1;
            let value2 = value2 << lshift2;

            let bits1 = MaskedBits::new(value1, 4 - lshift1, lshift1);
            let bits2 = MaskedBits::new(&value2, 4 - lshift2, lshift2);
            assert_eq!((bits1.len(), bits2.len()), (4, 4));
            assert_eq!((u8::from(&bits1), u8::from(&bits2)), (value1 >> lshift1, value2 >> lshift2));

            let result = bits1 | bits2;
            assert_eq!(result.len(), 4);
            assert_eq!(u8::from(&result), ((value1 >> lshift1) | (value2 >> lshift2)) & (0xFF_u8 >> 4));
        }

        #[test]
        fn test_bitxor(value1 in 0_u8..0x10, lshift1 in 0_u32..=4, value2 in 0_u8..0x10, lshift2 in 0_u32..=4) {
            let value1 = value1 << lshift1;
            let value2 = value2 << lshift2;

            let bits1 = MaskedBits::new(value1, 4 - lshift1, lshift1);
            let bits2 = MaskedBits::new(value2, 4 - lshift2, lshift2);
            assert_eq!((bits1.len(), bits2.len()), (4, 4));
            assert_eq!((u8::from(&bits1), u8::from(&bits2)), (value1 >> lshift1, value2 >> lshift2));

            let result = bits2 ^ bits1;
            assert_eq!(result.len(), 4);
            assert_eq!(u8::from(&result), ((value1 >> lshift1) ^ (value2 >> lshift2)) & (0xFF_u8 >> 4));
        }

        #[test]
        fn test_assign1(value1 in 0_u8..0x10, lshift1 in 0_u32..=4, value2 in 0_u8..0x10, lshift2 in 0_u32..=4) {
            let value1 = value1 << lshift1;
            let value2 = value2 << lshift2;
            let mut result = value1;

            let mut bits_result = MaskedBits::new(&mut result, 4 - lshift1, lshift1);
            let bits2 = MaskedBits::new(value2, 4 - lshift2, lshift2);

            bits_result.assign(bits2);

            assert_eq!(bits_result.len(), 4);
            let mask = (0xFF_u8 << lshift1) & (0xFF_u8 >> (4 - lshift1));
            assert_eq!(result & !mask, (!mask) & value1);
            assert_eq!(result & mask, mask & (value2.fp_ishl((lshift1 as i32) - (lshift2 as i32))));
        }

        #[test]
        fn test_assign2(value1 in 0_u8..0x10, lshift1 in 0_u32..=4, value2 in 0_u8..0x10, lshift2 in 0_u32..=4) {
            let value1 = value1 << lshift1;
            let value2 = value2 << lshift2;
            let mut result = value2;

            let mut bits_result = MaskedBits::new(&mut result, 4 - lshift2, lshift2);
            let bits1 = MaskedBits::new(value1, 4 - lshift1, lshift1);

            bits_result.assign(bits1);

            assert_eq!(bits_result.len(), 4);
            let mask = (0xFF_u8 << lshift2) & (0xFF_u8 >> (4 - lshift2));
            assert_eq!(result & !mask, (!mask) & value2);
            assert_eq!(result & mask, mask & (value1.fp_ishl((lshift2 as i32) - (lshift1 as i32))));
        }

        #[test]
        fn test_assign3(value1 in 0_u8..0x10, lshift1 in 0_u32..=4, value2 in 0_u8..0x10, lshift2 in 0_u32..=4) {
            let value1 = value1 << lshift1;
            let value2 = value2 << lshift2;
            let mut result = value2;

            let mut bits_result = MaskedBits::new(&mut result, 4 - lshift2, lshift2);
            let bits1 = MaskedBits::new(value1, 4 - lshift1, lshift1);

            bits_result.assign(bits1);

            assert_eq!(bits_result.len(), 4);
            let mask = (0xFF_u8 << lshift2) & (0xFF_u8 >> (4 - lshift2));
            assert_eq!(result & !mask, (!mask) & value2);
            assert_eq!(result & mask, mask & (value1.fp_ishl((lshift2 as i32) - (lshift1 as i32))));
        }

        #[test]
        fn test_bitand_assign(value1 in 0_u8..0x10, lshift1 in 0_u32..=4, value2 in 0_u8..0x10, lshift2 in 0_u32..=4) {
            let value1 = value1 << lshift1;
            let value2 = value2 << lshift2;
            let mut result = value1;

            let mut bits_result = MaskedBits::new(&mut result, 4 - lshift1, lshift1);
            let bits2 = MaskedBits::new(value2, 4 - lshift2, lshift2);

            bits_result &= bits2;

            assert_eq!(bits_result.len(), 4);
            let mask = (0xFF_u8 << lshift1) & (0xFF_u8 >> (4 - lshift1));
            assert_eq!(result & !mask, (!mask) & value1);
            assert_eq!(result & mask, mask & value1 & (value2.fp_ishl((lshift1 as i32) - (lshift2 as i32))));
        }

        #[test]
        fn test_bitor_assign(value1 in 0_u8..0x10, lshift1 in 0_u32..=4, value2 in 0_u8..0x10, lshift2 in 0_u32..=4) {
            let value1 = value1 << lshift1;
            let value2 = value2 << lshift2;
            let mut result = value2;

            let mut bits_result = MaskedBits::new(&mut result, 4 - lshift2, lshift2);
            let bits1 = MaskedBits::new(value1, 4 - lshift1, lshift1);

            bits_result |= bits1;

            assert_eq!(bits_result.len(), 4);
            let mask = (0xFF_u8 << lshift2) & (0xFF_u8 >> (4 - lshift2));
            assert_eq!(result & !mask, (!mask) & value2);
            assert_eq!(result & mask, mask & value2 | (value1.fp_ishl((lshift2 as i32) - (lshift1 as i32))));
        }

        #[test]
        fn test_bitxor_assign(value1 in 0_u8..0x10, lshift1 in 0_u32..=4, value2 in 0_u8..0x10, lshift2 in 0_u32..=4) {
            let value1 = value1 << lshift1;
            let value2 = value2 << lshift2;
            let mut result = value2;

            let mut bits_result = MaskedBits::new(&mut result, 4 - lshift2, lshift2);
            let bits1 = MaskedBits::new(value1, 4 - lshift1, lshift1);

            bits_result ^= bits1;

            assert_eq!(bits_result.len(), 4);
            let mask = (0xFF_u8 << lshift2) & (0xFF_u8 >> (4 - lshift2));
            assert_eq!(result & !mask, (!mask) & value2);
            assert_eq!(result & mask, mask & value2 ^ (value1.fp_ishl((lshift2 as i32) - (lshift1 as i32))));
        }
    }
}
