use crate::sbs_utils;
use crate::utils::IncompleteInt;

use core::num::{NonZeroU64, NonZeroUsize};

pub(crate) mod encode {
    use super::*;
    use crate::bitslice::BitEncoder;

    pub(crate) fn write(
        bitstream: &mut BitEncoder,
        value: NonZeroU64,
    ) -> Result<(), IncompleteInt<NonZeroU64>> {
        bitstream.write_bit(value.get() > 2).map_err(|_| {
            IncompleteInt::new_unbounded(NonZeroU64::new(1).expect("known to be non-zero"))
        })?;
        if value.get() <= 2 {
            bitstream.write_bit(value.get() > 1).map_err(|needed| {
                IncompleteInt::new_bounded(
                    (
                        NonZeroU64::new(1).expect("known to be non-zero"),
                        NonZeroU64::new(2).expect("known to be non-zero"),
                    ),
                    NonZeroUsize::new(needed).unwrap(),
                )
            })
        } else {
            sbs_utils::encode::write(bitstream, value)
        }
    }

    pub use crate::sbs_utils::encode::write_inf;
}

pub(crate) mod decode {

    use super::*;
    use crate::bitslice::BitDecoder;

    pub(crate) fn read(
        bitstream: &mut BitDecoder,
    ) -> Result<NonZeroU64, IncompleteInt<NonZeroU64>> {
        let first_bit = bitstream.read_bit().map_err(|_| {
            IncompleteInt::new_unbounded(NonZeroU64::new(1).expect("known to be non-zero"))
        })?;
        if first_bit {
            sbs_utils::decode::read(bitstream)
        } else {
            let second_bit = bitstream.read_bit().map_err(|needed| {
                IncompleteInt::new_bounded(
                    (
                        NonZeroU64::new(1).expect("known to be non-zero"),
                        NonZeroU64::new(2).expect("known to be non-zero"),
                    ),
                    needed,
                )
            })?;
            Ok(NonZeroU64::new(second_bit as u64 + 1).expect("known to be non-zero"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitslice::{BitDecoder, BitEncoder};
    use rstest::*;

    #[rstest(stream, expt_result,
        case([0b00111111], Ok(NonZeroU64::new(1).unwrap())),
        case([0b01111111], Ok(NonZeroU64::new(2).unwrap())),
        case([0b10011111], Ok(NonZeroU64::new(3).unwrap())),
        case([0b10101111], Ok(NonZeroU64::new(4).unwrap())),
        case([0b10111111], Ok(NonZeroU64::new(5).unwrap())),
        case([0b11000111], Ok(NonZeroU64::new(6).unwrap())),
        case([0b11001111], Ok(NonZeroU64::new(7).unwrap())),
        case([0b11010011], Ok(NonZeroU64::new(8).unwrap())),
        case([0b11011111], Ok(NonZeroU64::new(11).unwrap())),
        case([0b11100001], Ok(NonZeroU64::new(12).unwrap())),
        case([0b11100111], Ok(NonZeroU64::new(15).unwrap())),
        case([0b11101000], Ok(NonZeroU64::new(16).unwrap())),
        case([0b11101111], Ok(NonZeroU64::new(23).unwrap())),
        case([0b11110000], Err(
            IncompleteInt::new_bounded(
                (NonZeroU64::new(24).unwrap(), NonZeroU64::new(25).unwrap()),
                NonZeroUsize::new(1).unwrap(),
            ),
        )),
        case([0b11111111], Err(
            IncompleteInt::new_unbounded(NonZeroU64::new(0x180).unwrap()),
        )),
    )]
    fn test_read(stream: [u8; 1], expt_result: Result<NonZeroU64, IncompleteInt<NonZeroU64>>) {
        let mut bitstream = BitDecoder::new(&stream[..]);
        let calc_result = decode::read(&mut bitstream);
        assert_eq!(calc_result, expt_result);
    }

    #[rstest(value, expt_stream, expt_result,
        case(NonZeroU64::new(1).unwrap(), [0b00111111], Ok(())),
        case(NonZeroU64::new(2).unwrap(), [0b01111111], Ok(())),
        case(NonZeroU64::new(3).unwrap(), [0b10011111], Ok(())),
        case(NonZeroU64::new(4).unwrap(), [0b10101111], Ok(())),
        case(NonZeroU64::new(5).unwrap(), [0b10111111], Ok(())),
        case(NonZeroU64::new(6).unwrap(), [0b11000111], Ok(())),
        case(NonZeroU64::new(7).unwrap(), [0b11001111], Ok(())),
        case(NonZeroU64::new(8).unwrap(), [0b11010011], Ok(())),
        case(NonZeroU64::new(11).unwrap(), [0b11011111], Ok(())),
        case(NonZeroU64::new(12).unwrap(), [0b11100001], Ok(())),
        case(NonZeroU64::new(15).unwrap(), [0b11100111], Ok(())),
        case(NonZeroU64::new(16).unwrap(), [0b11101000], Ok(())),
        case(NonZeroU64::new(23).unwrap(), [0b11101111], Ok(())),
        case(NonZeroU64::new(24).unwrap(), [0b11110000], Err(
            IncompleteInt::new_bounded(
                (NonZeroU64::new(24).unwrap(), NonZeroU64::new(25).unwrap()),
                NonZeroUsize::new(1).unwrap(),
            ),
        )),
        case(NonZeroU64::new(0x180).unwrap(), [0b11111111], Err(
            IncompleteInt::new_unbounded(NonZeroU64::new(0x180).unwrap()),
        )),
    )]
    fn test_write(
        value: NonZeroU64,
        expt_stream: [u8; 1],
        expt_result: Result<(), IncompleteInt<NonZeroU64>>, // <- need to use `InputStream` for immutabile static references
    ) {
        let mut stream = [0xFF_u8];
        let mut bitstream = BitEncoder::new(&mut stream[..]);
        let calc_result = encode::write(&mut bitstream, value);
        encode::write_inf(&mut bitstream);
        assert_eq!(calc_result, expt_result);
        assert_eq!(stream, expt_stream);
    }
}
