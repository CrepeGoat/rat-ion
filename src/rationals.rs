use crate::bitslice::BitDecoder;
use crate::sbs_main::Coder;
use crate::utils::IncompleteInt;

use core::cmp::max;
use core::num::NonZeroU64;

pub fn map_to_complete_cf<I: Iterator<Item = Result<NonZeroU64, IncompleteInt<NonZeroU64>>>>(
    iter: I,
) -> impl Iterator<Item = NonZeroU64> {
    iter.filter_map(|inc_int| match inc_int {
        Ok(value) => Some(value),
        Err(IncompleteInt::Unbounded(_)) => None,
        Err(IncompleteInt::Bounded(range, _)) => {
            // Goal: for a given bit count `n`:
            // - each encoding should be unique
            // - the scheme should prefer to first cover all smaller-denominator values
            //   -> each ambiguous encoding should have the smallest denominator possible
            // https://en.wikipedia.org/wiki/Continued_fraction#Best_rational_within_an_interval
            NonZeroU64::new(range.start().get() + 1)
        }
    })
}

pub fn cf_to_rational64<I: Iterator<Item = NonZeroU64>>(iter_rev: I) -> (u64, NonZeroU64) {
    let (den, num) = iter_rev.fold((1_u64, 0_u64), |(num, den), item| {
        (item.get() * num + den, num)
    });
    NonZeroU64::new(den as u64).map(|d| (num, d)).unwrap()
}

pub fn decode_c8(bits: &u8) -> (u64, NonZeroU64) {
    let bits = bits.to_be_bytes();
    let bitstream = BitDecoder::new(&bits[..]);
    let coder = Coder::default();
    cf_to_rational64(
        map_to_complete_cf(coder.read_iter(bitstream))
            .collect::<Vec<_>>()
            .into_iter()
            .rev(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    // use rstest::*;

    #[test]
    fn test_decode_c8_uniqueness() {
        let encodings = 0_u8..=0xFF;
        let endecodings: Vec<_> = encodings.map(|byte| (byte, decode_c8(&byte))).collect();
        // println!("{:?}", endecodings);

        let mut duplicates = std::collections::HashMap::new();
        for (byte, decoding) in endecodings {
            duplicates
                .entry(decoding)
                .or_insert(Vec::default())
                .push(byte);
        }
        duplicates.retain(|_decoding, encodings| encodings.len() > 1);
        println!("{:?}", duplicates.len());
        println!("{:X?}", duplicates);
        assert!(duplicates.len() == 0);
    }

    // #[rstest(seq1, seq2,
    //     case(vec![], vec![]),
    // )]
    // fn test_map_to_complete_cf_uniqueness(
    //     seq1: Vec<Result<NonZeroU64, IncompleteInt<NonZeroU64>>>,
    //     seq2: Vec<Result<NonZeroU64, IncompleteInt<NonZeroU64>>>,
    // ) {
    //     let result1 = cf_to_rational64(map_to_complete_cf(seq1.into_iter().rev()));
    //     let result2 = cf_to_rational64(map_to_complete_cf(seq2.into_iter().rev()));
    //     assert_ne!(result1, result2);
    // }
}
