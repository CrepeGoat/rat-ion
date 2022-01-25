//use num::{PrimInt, Unsigned};
//
//struct BitwiseArray<T: PrimInt + Unsigned, U: AsRef<T>> {

use core::borrow::Borrow;
use core::ops::{BitAnd, BitOr, BitXor};

trait FoolProofShl {
    fn fp_shl(self, shift: i32) -> Self;
}

trait FoolProofShr {
    fn fp_shr(self, shift: i32) -> Self;
}

impl FoolProofShl for u8 {
    #[inline]
    fn fp_shl(self, shift: i32) -> Self {
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
    fn fp_shr(self, shift: i32) -> Self {
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
    left_offset: u32,
    right_offset: u32,
    //_marker: std::marker::PhantomData<T>,
}

impl<U, const SHIFT_LEFT: bool> BitwiseArray<U, SHIFT_LEFT> {
    pub fn new(data: U, left_offset: u32, right_offset: u32) -> Self {
        assert!(left_offset + right_offset <= 8);
        Self {
            data,
            left_offset,
            right_offset,
        }
    }

    pub fn len(&self) -> u32 {
        8 - self.left_offset - self.right_offset
    }

    fn mask(&self) -> u8 {
        u8::MAX.fp_shl(self.right_offset as i32) & u8::MAX.fp_shr(self.left_offset as i32)
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
                right_offset: self.right_offset + self.len() - len,
                ..self
            }
        } else {
            Self {
                left_offset: self.left_offset + self.len() - len,
                ..self
            }
        }
    }
}

impl<U: Borrow<u8>, const SHIFT_LEFT: bool> From<BitwiseArray<U, SHIFT_LEFT>> for u8 {
    fn from(value: BitwiseArray<U, SHIFT_LEFT>) -> Self {
        value.masked().fp_shr(value.right_offset as i32)
    }
}

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
                    .fp_shl((other.left_offset as i32) - (_self.left_offset as i32)),
            _self.left_offset,
            _self.right_offset,
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
                    .fp_shl((other.left_offset as i32) - (_self.left_offset as i32)),
            _self.left_offset,
            _self.right_offset,
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
                    .fp_shl((other.left_offset as i32) - (_self.left_offset as i32)),
            _self.left_offset,
            _self.right_offset,
        )
    }
}
