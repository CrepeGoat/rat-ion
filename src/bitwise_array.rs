//use num::{PrimInt, Unsigned};
//
//struct BitwiseArray<T: PrimInt + Unsigned, U: AsRef<T>> {

use core::borrow::Borrow;
use core::ops::{BitAnd, BitOr, BitXor};

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
        u8::MAX.checked_shl(self.right_offset).unwrap_or_default()
            & u8::MAX.checked_shr(self.left_offset).unwrap_or_default()
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
        value
            .masked()
            .checked_shr(value.right_offset)
            .unwrap_or_default()
    }
}

impl<U1: Borrow<u8>, U2: Borrow<u8>, const SHIFT_LEFT_1: bool, const SHIFT_LEFT_2: bool>
    BitAnd<BitwiseArray<U2, SHIFT_LEFT_2>> for BitwiseArray<U1, SHIFT_LEFT_1>
{
    type Output = BitwiseArray<u8, true>;

    fn bitand(self, other: BitwiseArray<U2, SHIFT_LEFT_2>) -> Self::Output {
        let _self = self.reduce_len(other.len());
        let other = other.reduce_len(_self.len());

        if _self.left_offset < other.left_offset {
            Self::Output::new(
                _self.masked()
                    & other
                        .masked()
                        .checked_shl(other.left_offset - _self.left_offset)
                        .unwrap_or_default(),
                _self.left_offset,
                _self.left_offset,
            )
        } else {
            Self::Output {
                data: _self.masked()
                    & other
                        .masked()
                        .checked_shr(other.right_offset - _self.right_offset)
                        .unwrap_or_default(),
                left_offset: _self.left_offset,
                right_offset: _self.left_offset,
            }
        }
    }
}

impl<U1: Borrow<u8>, U2: Borrow<u8>, const SHIFT_LEFT_1: bool, const SHIFT_LEFT_2: bool>
    BitOr<BitwiseArray<U2, SHIFT_LEFT_2>> for BitwiseArray<U1, SHIFT_LEFT_1>
{
    type Output = BitwiseArray<u8, true>;

    fn bitor(self, other: BitwiseArray<U2, SHIFT_LEFT_2>) -> Self::Output {
        let _self = self.reduce_len(other.len());
        let other = other.reduce_len(_self.len());

        if _self.left_offset < other.left_offset {
            Self::Output {
                data: _self.masked()
                    | other
                        .masked()
                        .checked_shl(other.left_offset - _self.left_offset)
                        .unwrap_or_default(),
                left_offset: _self.left_offset,
                right_offset: _self.left_offset,
            }
        } else {
            Self::Output {
                data: _self.masked()
                    | other
                        .masked()
                        .checked_shr(other.right_offset - _self.right_offset)
                        .unwrap_or_default(),
                left_offset: _self.left_offset,
                right_offset: _self.left_offset,
            }
        }
    }
}

impl<U1: Borrow<u8>, U2: Borrow<u8>, const SHIFT_LEFT_1: bool, const SHIFT_LEFT_2: bool>
    BitXor<BitwiseArray<U2, SHIFT_LEFT_2>> for BitwiseArray<U1, SHIFT_LEFT_1>
{
    type Output = BitwiseArray<u8, true>;

    fn bitxor(self, other: BitwiseArray<U2, SHIFT_LEFT_2>) -> Self::Output {
        let _self = self.reduce_len(other.len());
        let other = other.reduce_len(_self.len());

        if _self.left_offset < other.left_offset {
            Self::Output {
                data: _self.masked()
                    ^ other
                        .masked()
                        .checked_shl(other.left_offset - _self.left_offset)
                        .unwrap_or_default(),
                left_offset: _self.left_offset,
                right_offset: _self.left_offset,
            }
        } else {
            Self::Output {
                data: _self.masked()
                    ^ other
                        .masked()
                        .checked_shr(other.right_offset - _self.right_offset)
                        .unwrap_or_default(),
                left_offset: _self.left_offset,
                right_offset: _self.left_offset,
            }
        }
    }
}
