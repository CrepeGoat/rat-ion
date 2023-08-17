use super::{sbs1, sbs2};
use crate::bitstream::{BitDecoder, BitEncoder};
use crate::symbol_defs::IncompleteInt;

use core::num::NonZeroU64;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum SbsMarker {
    Mode1,
    Mode2,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum RhoRegion {
    Eq0,
    Leq1div3,
    Gthan1d3Lthan3d4,
    Geq3Div4,
    Eq1,
}

impl RhoRegion {
    pub fn next(self, value: NonZeroU64) -> Self {
        match (self, value.get()) {
            (Self::Eq0, 1) => Self::Eq1,
            (Self::Leq1div3, 1) => Self::Geq3Div4,
            (_, 1) => Self::Gthan1d3Lthan3d4,
            (Self::Eq1, 2) => Self::Leq1div3,
            (_, 2) => Self::Gthan1d3Lthan3d4,
            _ => Self::Leq1div3,
        }
    }

    pub fn which(self) -> SbsMarker {
        match self {
            Self::Eq0 | Self::Leq1div3 | Self::Gthan1d3Lthan3d4 => SbsMarker::Mode1,
            Self::Geq3Div4 | Self::Eq1 => SbsMarker::Mode2,
        }
    }
}

impl Default for RhoRegion {
    fn default() -> Self {
        Self::Eq0
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct Coder(RhoRegion);

impl Coder {
    pub fn write(
        &mut self,
        bitstream: &mut BitEncoder,
        value: NonZeroU64,
    ) -> Result<(), IncompleteInt<NonZeroU64>> {
        let result = match self.0.which() {
            SbsMarker::Mode1 => sbs1::encode::write(bitstream, value),
            SbsMarker::Mode2 => sbs2::encode::write(bitstream, value),
        }?;
        self.0 = self.0.next(value);

        Ok(result)
    }

    pub fn write_inf(&mut self, bitstream: &mut BitEncoder) -> IncompleteInt<NonZeroU64> {
        let result = match self.0.which() {
            SbsMarker::Mode1 => sbs1::encode::write_inf(bitstream),
            SbsMarker::Mode2 => sbs2::encode::write_inf(bitstream),
        };
        self.0 = RhoRegion::Eq0;

        result
    }

    pub fn read(
        &mut self,
        bitstream: &mut BitDecoder,
    ) -> Result<NonZeroU64, IncompleteInt<NonZeroU64>> {
        let result = match self.0.which() {
            SbsMarker::Mode1 => sbs1::decode::read(bitstream),
            SbsMarker::Mode2 => sbs2::decode::read(bitstream),
        }?;
        self.0 = self.0.next(result);

        Ok(result)
    }

    pub fn read_iter(
        mut self,
        mut bitstream: BitDecoder,
    ) -> impl Iterator<Item = Result<NonZeroU64, IncompleteInt<NonZeroU64>>> + '_ {
        let mut is_done: bool = false;
        core::iter::from_fn(move || {
            if is_done {
                None
            } else {
                let item = self.read(&mut bitstream);
                if item.is_err() {
                    is_done = true;
                }
                Some(item)
            }
        })
    }

    pub fn write_iter<I: Iterator<Item = NonZeroU64>>(
        mut self,
        mut bitstream: BitEncoder,
        iter: I,
    ) -> Result<(), ()> {
        for item in iter {
            self.write(&mut bitstream, item).map_err(|_| ())?;
        }
        self.write_inf(&mut bitstream);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitstream::{BitDecoder, BitEncoder};
    use rstest::*;

    #[rstest(parser_state, stream, expt_mode,
        case(RhoRegion::Eq0, [0b01111111], SbsMarker::Mode1),
        case(RhoRegion::Eq0, [0b10011111], SbsMarker::Mode1),
        case(RhoRegion::Eq0, [0b10101111], SbsMarker::Mode1),
        case(RhoRegion::Eq0, [0b10111111], SbsMarker::Mode1),
        case(RhoRegion::Eq0, [0b11000111], SbsMarker::Mode1),
        case(RhoRegion::Eq0, [0b11001111], SbsMarker::Mode1),
        case(RhoRegion::Eq0, [0b11010011], SbsMarker::Mode1),
        case(RhoRegion::Eq0, [0b11011111], SbsMarker::Mode1),
        case(RhoRegion::Eq0, [0b11100001], SbsMarker::Mode1),
        case(RhoRegion::Eq0, [0b11100111], SbsMarker::Mode1),
        case(RhoRegion::Eq0, [0b11101000], SbsMarker::Mode1),
        case(RhoRegion::Eq0, [0b11101111], SbsMarker::Mode1),
        case(RhoRegion::Eq0, [0b11110000], SbsMarker::Mode1),
        case(RhoRegion::Eq0, [0b11111111], SbsMarker::Mode1),

        case(RhoRegion::Eq1, [0b00111111], SbsMarker::Mode2),
        case(RhoRegion::Eq1, [0b01111111], SbsMarker::Mode2),
        case(RhoRegion::Eq1, [0b10011111], SbsMarker::Mode2),
        case(RhoRegion::Eq1, [0b10101111], SbsMarker::Mode2),
        case(RhoRegion::Eq1, [0b10111111], SbsMarker::Mode2),
        case(RhoRegion::Eq1, [0b11000111], SbsMarker::Mode2),
        case(RhoRegion::Eq1, [0b11001111], SbsMarker::Mode2),
        case(RhoRegion::Eq1, [0b11010011], SbsMarker::Mode2),
        case(RhoRegion::Eq1, [0b11011111], SbsMarker::Mode2),
        case(RhoRegion::Eq1, [0b11100001], SbsMarker::Mode2),
        case(RhoRegion::Eq1, [0b11100111], SbsMarker::Mode2),
        case(RhoRegion::Eq1, [0b11101000], SbsMarker::Mode2),
        case(RhoRegion::Eq1, [0b11101111], SbsMarker::Mode2),
        case(RhoRegion::Eq1, [0b11110000], SbsMarker::Mode2),
        case(RhoRegion::Eq1, [0b11111111], SbsMarker::Mode2),
    )]
    fn test_read(parser_state: RhoRegion, stream: [u8; 1], expt_mode: SbsMarker) {
        let mut bitstream1 = BitDecoder::new(&stream[..]);
        let mut bitstream2 = BitDecoder::new(&stream[..]);
        let mut coder = Coder(parser_state);
        let calc_result = coder.read(&mut bitstream1);
        let expt_result = match expt_mode {
            SbsMarker::Mode1 => sbs1::decode::read(&mut bitstream2),
            SbsMarker::Mode2 => sbs2::decode::read(&mut bitstream2),
        };
        assert_eq!(calc_result, expt_result);
    }

    #[rstest(parser_state, value, expt_mode,
        case(RhoRegion::Eq0, NonZeroU64::new(1).unwrap(), SbsMarker::Mode1),
        case(RhoRegion::Eq0, NonZeroU64::new(2).unwrap(), SbsMarker::Mode1),
        case(RhoRegion::Eq0, NonZeroU64::new(3).unwrap(), SbsMarker::Mode1),
        case(RhoRegion::Eq0, NonZeroU64::new(4).unwrap(), SbsMarker::Mode1),
        case(RhoRegion::Eq0, NonZeroU64::new(5).unwrap(), SbsMarker::Mode1),
        case(RhoRegion::Eq0, NonZeroU64::new(6).unwrap(), SbsMarker::Mode1),
        case(RhoRegion::Eq0, NonZeroU64::new(7).unwrap(), SbsMarker::Mode1),
        case(RhoRegion::Eq0, NonZeroU64::new(10).unwrap(), SbsMarker::Mode1),
        case(RhoRegion::Eq0, NonZeroU64::new(11).unwrap(), SbsMarker::Mode1),
        case(RhoRegion::Eq0, NonZeroU64::new(14).unwrap(), SbsMarker::Mode1),
        case(RhoRegion::Eq0, NonZeroU64::new(15).unwrap(), SbsMarker::Mode1),
        case(RhoRegion::Eq0, NonZeroU64::new(22).unwrap(), SbsMarker::Mode1),
        case(RhoRegion::Eq0, NonZeroU64::new(23).unwrap(), SbsMarker::Mode1),
        case(RhoRegion::Eq0, NonZeroU64::new(0x17F).unwrap(), SbsMarker::Mode1),

        case(RhoRegion::Eq1, NonZeroU64::new(1).unwrap(), SbsMarker::Mode2),
        case(RhoRegion::Eq1, NonZeroU64::new(2).unwrap(), SbsMarker::Mode2),
        case(RhoRegion::Eq1, NonZeroU64::new(3).unwrap(), SbsMarker::Mode2),
        case(RhoRegion::Eq1, NonZeroU64::new(4).unwrap(), SbsMarker::Mode2),
        case(RhoRegion::Eq1, NonZeroU64::new(5).unwrap(), SbsMarker::Mode2),
        case(RhoRegion::Eq1, NonZeroU64::new(6).unwrap(), SbsMarker::Mode2),
        case(RhoRegion::Eq1, NonZeroU64::new(7).unwrap(), SbsMarker::Mode2),
        case(RhoRegion::Eq1, NonZeroU64::new(8).unwrap(), SbsMarker::Mode2),
        case(RhoRegion::Eq1, NonZeroU64::new(11).unwrap(), SbsMarker::Mode2),
        case(RhoRegion::Eq1, NonZeroU64::new(12).unwrap(), SbsMarker::Mode2),
        case(RhoRegion::Eq1, NonZeroU64::new(15).unwrap(), SbsMarker::Mode2),
        case(RhoRegion::Eq1, NonZeroU64::new(16).unwrap(), SbsMarker::Mode2),
        case(RhoRegion::Eq1, NonZeroU64::new(23).unwrap(), SbsMarker::Mode2),
        case(RhoRegion::Eq1, NonZeroU64::new(24).unwrap(), SbsMarker::Mode2),
        case(RhoRegion::Eq1, NonZeroU64::new(0x180).unwrap(), SbsMarker::Mode2),
    )]
    fn test_write(parser_state: RhoRegion, value: NonZeroU64, expt_mode: SbsMarker) {
        let mut stream1 = [0xFF_u8];
        let mut stream2 = [0xFF_u8];
        let mut bitstream1 = BitEncoder::new(&mut stream1[..]);
        let mut bitstream2 = BitEncoder::new(&mut stream2[..]);

        let mut coder = Coder(parser_state);
        let calc_result = coder.write(&mut bitstream1, value);
        coder.write_inf(&mut bitstream1);

        let expt_result = match expt_mode {
            SbsMarker::Mode1 => sbs1::encode::write(&mut bitstream2, value),
            SbsMarker::Mode2 => sbs2::encode::write(&mut bitstream2, value),
        };
        match expt_mode {
            SbsMarker::Mode1 => sbs1::encode::write_inf(&mut bitstream2),
            SbsMarker::Mode2 => sbs2::encode::write_inf(&mut bitstream2),
        };

        assert_eq!(calc_result, expt_result);
        assert_eq!(stream1, stream2);
    }
}
