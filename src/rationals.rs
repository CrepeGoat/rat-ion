use crate::bitslice::BitDecoder;
use crate::sbs_main::Coder;
use crate::utils::IncompleteInt;

use core::cmp::max;
use core::num::NonZeroU64;

fn iter_cf(coder: Coder, bitstream: BitDecoder) -> impl Iterator<Item = NonZeroU64> + '_ {
    coder
        .read_iter(bitstream)
        .filter_map(|inc_int| match inc_int {
            Ok(value) => Some(value),
            Err(IncompleteInt::Unbounded(_)) => None,
            Err(IncompleteInt::Bounded(range, _)) => {
                // Goal: for a given bit count `n`:
                // - each encoding should be unique
                // - the scheme should prefer to first cover all smaller-denominator values
                //   -> each ambiguous encoding should have the smallest denominator possible
                // https://en.wikipedia.org/wiki/Continued_fraction#Best_rational_within_an_interval
                NonZeroU64::new(max(range.start().get(), 2))
            }
        })
}

pub fn cf_to_rational64<I: Iterator<Item = NonZeroU64>>(iter_rev: I) -> (i64, NonZeroU64) {
    let (mut den, mut num) = iter_rev.fold((1_i64, 0_i64), |(num, den), item| {
        ((item.get() as i64) * num + den, num)
    });
    if den < 0 {
        num *= -1;
        den *= -1;
    }
    NonZeroU64::new(den as u64).map(|d| (num, d)).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitslice::{BitDecoder, BitEncoder};
    use crate::sbs_main::Coder;
    use rstest::*;

    #[test]
    fn test_cf_to_rational64_uniques() {
        let encodeds: Vec<_> = (0_u8..=0x0F)
            .map(|byte| {
                let bits = byte.to_be_bytes();
                let mut bitstream = BitDecoder::new(&bits[..]);
                for _ in 0..4 {
                    bitstream.read_bit();
                }
                let coder = Coder::default();
                cf_to_rational64(iter_cf(coder, bitstream))
            })
            .collect();
        println!("{:?}", encodeds);

        let mut cache = std::collections::HashSet::new();
        assert!(encodeds.into_iter().all(move |item| cache.insert(item)));
    }

    #[rstest(bits, bit_offset, expt_frac,
        case([0b00001111], 4, (1, NonZeroU64::new(1).unwrap())),
    )]
    fn test_cf_to_rational64(bits: [u8; 1], bit_offset: u32, expt_frac: (i64, NonZeroU64)) {
        let mut bitstream = BitDecoder::new(&bits[..]);
        for _ in 0..bit_offset {
            bitstream.read_bit();
        }
        let coder = Coder::default();
        let result = cf_to_rational64(iter_cf(coder, bitstream));

        assert_eq!(result, expt_frac);
    }
}
