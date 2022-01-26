//use num::{PrimInt, Unsigned};
//
//struct BitwiseArray<T: PrimInt + Unsigned, U: AsRef<T>> {

use core::borrow::Borrow;
use core::ops::{BitAnd, BitOr, BitXor};

trait FoolProofShl {
    fn fp_shl(self, shift: u32) -> Self;
    fn fp_ishl(self, shift: i32) -> Self;
}

trait FoolProofShr {
    fn fp_shr(self, shift: u32) -> Self;
    fn fp_ishr(self, shift: i32) -> Self;
}

impl FoolProofShl for u8 {
    #[inline]
    fn fp_shl(self, shift: u32) -> Self {
        self.checked_shl(shift).unwrap_or_default()
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
}

impl FoolProofShr for u8 {
    #[inline]
    fn fp_shr(self, shift: u32) -> Self {
        self.checked_shr(shift).unwrap_or_default()
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
struct BitwiseArray<U, const SHIFT_LEFT: bool> {
    data: U,
    left_margin: u32,
    right_margin: u32,
    //_marker: std::marker::PhantomData<T>,
}

impl<U, const SHIFT_LEFT: bool> BitwiseArray<U, SHIFT_LEFT> {
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

    fn reduce_len(self, len: u32) -> Self {
        if len >= self.len() {
            self
        } else if SHIFT_LEFT {
            Self {
                right_margin: self.right_margin + self.len() - len,
                ..self
            }
        } else {
            Self {
                left_margin: self.left_margin + self.len() - len,
                ..self
            }
        }
    }
}

impl<U: Borrow<u8>, const SHIFT_LEFT: bool> From<&BitwiseArray<U, SHIFT_LEFT>> for u8 {
    fn from(value: &BitwiseArray<U, SHIFT_LEFT>) -> Self {
        value.masked().fp_shr(value.right_margin)
    }
}

impl<U1: Borrow<u8>, U2: Borrow<u8>, const SHIFT_LEFT_1: bool, const SHIFT_LEFT_2: bool>
    PartialEq<BitwiseArray<U2, SHIFT_LEFT_2>> for BitwiseArray<U1, SHIFT_LEFT_1>
{
    fn eq(&self, other: &BitwiseArray<U2, SHIFT_LEFT_2>) -> bool {
        (self.len() == other.len()) & u8::from(self).eq(&u8::from(other))
    }
}

impl<U: Borrow<u8>, const SHIFT_LEFT: bool> Eq for BitwiseArray<U, SHIFT_LEFT> {}

impl<U1: Borrow<u8>, U2: Borrow<u8>, const SHIFT_LEFT_1: bool, const SHIFT_LEFT_2: bool>
    BitAnd<BitwiseArray<U2, SHIFT_LEFT_2>> for BitwiseArray<U1, SHIFT_LEFT_1>
{
    type Output = BitwiseArray<u8, true>;

    fn bitand(self, other: BitwiseArray<U2, SHIFT_LEFT_2>) -> Self::Output {
        let _self = self.reduce_len(other.len());
        let other = other.reduce_len(_self.len());

        Self::Output::new(
            _self.masked()
                & other
                    .masked()
                    .fp_ishl((other.left_margin as i32) - (_self.left_margin as i32)),
            _self.left_margin,
            _self.right_margin,
        )
    }
}

impl<U1: Borrow<u8>, U2: Borrow<u8>, const SHIFT_LEFT_1: bool, const SHIFT_LEFT_2: bool>
    BitOr<BitwiseArray<U2, SHIFT_LEFT_2>> for BitwiseArray<U1, SHIFT_LEFT_1>
{
    type Output = BitwiseArray<u8, true>;

    fn bitor(self, other: BitwiseArray<U2, SHIFT_LEFT_2>) -> Self::Output {
        let _self = self.reduce_len(other.len());
        let other = other.reduce_len(_self.len());

        Self::Output::new(
            _self.masked()
                | other
                    .masked()
                    .fp_ishl((other.left_margin as i32) - (_self.left_margin as i32)),
            _self.left_margin,
            _self.right_margin,
        )
    }
}

impl<U1: Borrow<u8>, U2: Borrow<u8>, const SHIFT_LEFT_1: bool, const SHIFT_LEFT_2: bool>
    BitXor<BitwiseArray<U2, SHIFT_LEFT_2>> for BitwiseArray<U1, SHIFT_LEFT_1>
{
    type Output = BitwiseArray<u8, true>;

    fn bitxor(self, other: BitwiseArray<U2, SHIFT_LEFT_2>) -> Self::Output {
        let _self = self.reduce_len(other.len());
        let other = other.reduce_len(_self.len());

        Self::Output::new(
            _self.masked()
                ^ other
                    .masked()
                    .fp_ishl((other.left_margin as i32) - (_self.left_margin as i32)),
            _self.left_margin,
            _self.right_margin,
        )
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
    }
}
