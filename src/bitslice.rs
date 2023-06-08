use std::num::NonZeroUsize;

use crate::masked_bits::MaskedBits;

#[inline(always)]
const fn masked_bit(bits: u8, index: u32) -> bool {
    (bits & (1 << index)) != 0
}

pub struct BitDecoder<'a> {
    bits: &'a [u8],
    bit_offset: u32,
}

impl<'a> BitDecoder<'a> {
    pub fn new(bits: &'a [u8]) -> Self {
        Self {
            bits,
            bit_offset: 0,
        }
    }

    pub fn bits_left(&self) -> usize {
        8 * self.bits.len() - (self.bit_offset as usize)
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.bits_left() == 0
    }

    #[inline]
    fn validate_len(&self, count: usize) -> Result<(), NonZeroUsize> {
        if self.bits_left() < count {
            Err(NonZeroUsize::new(count - self.bits_left()).expect("known to be non-zero"))
        } else {
            Ok(())
        }
    }

    // fn bitarray(&mut self) -> Result<MaskedBits<&u8>, usize> {
    //     Ok(MaskedBits::new(
    //         self.bits.first().ok_or(0_usize)?,
    //         self.bit_offset,
    //         0,
    //     ))
    // }

    pub fn skip_bits(&mut self, count: usize) -> Result<(), NonZeroUsize> {
        self.validate_len(count)?;
        self.bits = &self.bits[((count + self.bit_offset as usize) / 8)..];
        self.bit_offset = ((count + self.bit_offset as usize) % 8) as u32;
        Ok(())
    }

    pub fn read_bit(&mut self) -> Result<bool, NonZeroUsize> {
        self.validate_len(1)?;
        let result = masked_bit(self.bits[0], 7 - self.bit_offset);
        self.skip_bits(1)
            .expect("length already guaranteed to be at least 1");
        Ok(result)
    }
}

pub struct BitEncoder<'a> {
    bits: &'a mut [u8],
    bit_offset: u32,
}

impl<'a> BitEncoder<'a> {
    pub fn new(bits: &'a mut [u8]) -> Self {
        Self {
            bits,
            bit_offset: 0,
        }
    }

    pub fn bits_left(&self) -> usize {
        8 * self.bits.len() - (self.bit_offset as usize)
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.bits_left() == 0
    }

    #[inline]
    fn validate_len(&self, count: usize) -> Result<(), NonZeroUsize> {
        if self.bits_left() < count {
            Err(NonZeroUsize::new(count - self.bits_left()).expect("known to be non-zero"))
        } else {
            Ok(())
        }
    }

    fn bitarray(&mut self) -> Option<MaskedBits<&mut u8>> {
        Some(MaskedBits::new(self.bits.first_mut()?, self.bit_offset, 0))
    }

    #[inline]
    fn advance(&mut self, count: usize) -> Result<(), NonZeroUsize> {
        self.validate_len(count)?;
        self.bits = &mut core::mem::replace(&mut self.bits, &mut [][..]) // <- avoids multiple &mut's at once
            [((count + self.bit_offset as usize) / 8)..];
        self.bit_offset = ((count + self.bit_offset as usize) % 8) as u32;
        Ok(())
    }

    pub fn write_bit(&mut self, bit: bool) -> Result<(), usize> {
        self.bitarray()
            .ok_or(NonZeroUsize::new(1).expect("known to be non-zero"))?
            .trim_trailing_to(1)
            .assign(MaskedBits::new(if bit { 0xFF } else { 0x00 }, 7, 0));
        self.advance(1)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::*;

    proptest! {
        #[test]
        fn read_bit_write_bit_eq(bits: [u8; 2]) {
            let mut new_bits = [0_u8; 2];
            let mut bit_buffer = vec![];

            // Read bits
            let mut reader = BitDecoder::new(&bits);
            while let Ok(bit) = reader.read_bit() {
                bit_buffer.push(bit);
            }

            // Write bits
            let mut writer = BitEncoder::new(&mut new_bits);
            for bit in bit_buffer.into_iter() {
                writer.write_bit(bit).expect("can write the same number of bits read");
            }

            assert_eq!(new_bits, bits);
        }
    }
}
