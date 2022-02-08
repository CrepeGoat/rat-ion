use crate::bitwise_array::{BitwiseArray, TrimLeft, TrimRight};

#[inline(always)]
const fn masked_suffix(bits: u64, len: usize) -> u64 {
    if len >= 8 * core::mem::size_of::<u64>() {
        bits
    } else {
        bits & !(u64::MAX << len)
    }
}

#[inline(always)]
const fn masked_bit(bits: u8, index: u32) -> bool {
    (bits & (1 << index)) != 0
}

pub(crate) struct BitDecoder<'a> {
    bits: &'a [u8],
    bit_offset: u32,
}

impl<'a> BitDecoder<'a> {
    pub(crate) fn new(bits: &'a [u8]) -> Self {
        Self {
            bits,
            bit_offset: 0,
        }
    }

    pub(crate) fn bits_left(&self) -> usize {
        8 * self.bits.len() - (self.bit_offset as usize)
    }

    #[inline]
    fn validate_len(&self, count: usize) -> Result<(), usize> {
        if self.bits_left() < count {
            Err(self.bits_left())
        } else {
            Ok(())
        }
    }

    fn bitarray(&mut self) -> Result<BitwiseArray<&u8, TrimRight>, usize> {
        self.validate_len(1)?;
        Ok(BitwiseArray::new(&self.bits[0], self.bit_offset, 0))
    }

    pub(crate) fn skip_bits(&mut self, count: usize) -> Result<(), usize> {
        self.validate_len(count)?;
        self.bits = &self.bits[((count + self.bit_offset as usize) / 8)..];
        self.bit_offset = ((count + self.bit_offset as usize) % 8) as u32;
        Ok(())
    }

    pub(crate) fn read_bit(&mut self) -> Result<bool, usize> {
        self.validate_len(1)?;
        let result = masked_bit(self.bits[0], self.bit_offset);
        self.skip_bits(1).unwrap();
        Ok(result)
    }
}

pub(crate) struct BitEncoder<'a> {
    bits: &'a mut [u8],
    bit_offset: u32,
}

impl<'a> BitEncoder<'a> {
    pub(crate) fn new(bits: &'a mut [u8]) -> Self {
        Self {
            bits,
            bit_offset: 0,
        }
    }

    pub(crate) fn bits_left(&self) -> usize {
        8 * self.bits.len() - (self.bit_offset as usize)
    }

    #[inline]
    fn validate_len(&self, count: usize) -> Result<(), usize> {
        if self.bits_left() < count {
            Err(self.bits_left())
        } else {
            Ok(())
        }
    }

    fn bitarray(&mut self) -> Result<BitwiseArray<&mut u8, TrimRight>, usize> {
        self.validate_len(1)?;
        Ok(BitwiseArray::new(&mut self.bits[0], self.bit_offset, 0))
    }

    fn advance(&'a mut self, count: usize) -> Result<(), usize> {
        self.validate_len(count)?;
        self.bits = &mut self.bits[((count + self.bit_offset as usize) / 8)..];
        self.bit_offset = ((count + self.bit_offset as usize) % 8) as u32;
        Ok(())
    }

    pub(crate) fn write_bit(&'a mut self, bit: bool) -> Result<(), usize> {
        let mut bitarray = self.bitarray()?;
        bitarray.assign(BitwiseArray::<_, TrimRight>::new(
            if bit { 0xFF } else { 0x00 },
            7,
            0,
        ));
        self.bits = &mut self.bits[((self.bit_offset + 1) / 8) as usize..];
        self.bit_offset = (self.bit_offset + 1) % 8;

        Ok(())
    }
}
