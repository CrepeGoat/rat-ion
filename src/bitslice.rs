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
    fn validate_len(&self, count: usize) -> Result<(), usize> {
        if self.bits_left() < count {
            Err(count - self.bits_left())
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

    pub fn skip_bits(&mut self, count: usize) -> Result<(), usize> {
        self.validate_len(count)?;
        self.bits = &self.bits[((count + self.bit_offset as usize) / 8)..];
        self.bit_offset = ((count + self.bit_offset as usize) % 8) as u32;
        Ok(())
    }

    pub fn read_bit(&mut self) -> Result<bool, usize> {
        self.validate_len(1)?;
        let result = masked_bit(self.bits[0], 7 - self.bit_offset);
        self.skip_bits(1).unwrap();
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
    fn validate_len(&self, count: usize) -> Result<(), usize> {
        if self.bits_left() < count {
            Err(self.bits_left())
        } else {
            Ok(())
        }
    }

    fn bitarray(&mut self) -> Result<MaskedBits<&mut u8>, usize> {
        Ok(MaskedBits::new(
            self.bits.first_mut().ok_or(0_usize)?,
            self.bit_offset,
            0,
        ))
    }

    #[inline]
    fn advance(&mut self, count: usize) -> Result<(), usize> {
        self.validate_len(count)?;
        self.bits = &mut core::mem::replace(&mut self.bits, &mut [][..]) // <- avoids multiple &mut's at once
            [((count + self.bit_offset as usize) / 8)..];
        self.bit_offset = ((count + self.bit_offset as usize) % 8) as u32;
        Ok(())
    }

    pub fn write_bit(&mut self, bit: bool) -> Result<(), usize> {
        self.bitarray()?.trim_trailing_to(1).assign(MaskedBits::new(
            if bit { 0xFF } else { 0x00 },
            7,
            0,
        ));
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
                writer.write_bit(bit).unwrap();
            }

            assert_eq!(new_bits, bits);
        }
    }
}
