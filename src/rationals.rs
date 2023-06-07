use crate::bitslice::BitDecoder;
use crate::sbs_main::Coder;
use crate::utils::IncompleteInt;

use core::num::NonZeroU64;

pub fn map_to_complete_cf<I: Iterator<Item = Result<NonZeroU64, IncompleteInt<NonZeroU64>>>>(
    iter: I,
) -> impl Iterator<Item = NonZeroU64> {
    let mut prev_items: [Option<I::Item>; 2] = [None, None];
    iter.filter_map(move |inc_int| {
        let result = match &inc_int {
            &Ok(value) => Some(value),
            Err(IncompleteInt::Unbounded(range)) => {
                // Each encoding should be unique
                //   -> change [..., n, 1, inf] to resolve differently from [..., n+1, inf]
                if prev_items[1] != None && prev_items[0] == Some(Ok(NonZeroU64::new(1).unwrap())) {
                    Some(NonZeroU64::new(range.start.get() + 1).unwrap())
                } else {
                    None
                }
            }
            Err(IncompleteInt::Bounded(range, _)) => {
                // Encodings should prefer to first cover all smaller-denominator values
                //   -> each ambiguous encoding should have the smallest denominator possible
                // https://en.wikipedia.org/wiki/Continued_fraction#Best_rational_within_an_interval
                Some(NonZeroU64::new(range.start().get() + 1).unwrap())
            }
        };
        prev_items[1] = prev_items[0].clone();
        prev_items[0] = Some(inc_int);
        result
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
    use std::collections::HashSet;

    use super::*;
    // use rstest::*;

    #[test]
    fn test_decode_c8_uniqueness() {
        let encodings = 0_u8..=0xFF;
        let mut decodings = encodings.map(|byte| decode_c8(&byte));

        let mut duplicates = std::collections::HashSet::new();
        assert!(decodings.all(|decoding| duplicates.insert(decoding)));
    }

    #[test]
    fn test_decode_c8_denominator_11_completeness() {
        const LARGEST_COVERED_DENOMINATOR: u64 = 11;
        let encodings = 0_u8..=0xFF;
        let decodings: HashSet<_> = encodings.map(|byte| decode_c8(&byte)).collect();

        for denom in 1..=LARGEST_COVERED_DENOMINATOR {
            for numer in 1..denom {
                if gcd(numer, denom) != 1 {
                    continue;
                }
                assert!(decodings.contains(&(numer, NonZeroU64::new(denom).unwrap())));
            }
            println!("denominator {:?} covered!", denom);
        }
    }

    #[test]
    fn test_gcd() {
        assert_eq!(gcd(5, 0), 5); // this behavior is used above -> do NOT remove!

        assert_eq!(gcd(5, 2), 1);
        assert_eq!(gcd(2, 5), 1);

        assert_eq!(gcd(128, 36), 4);
    }

    fn gcd(mut a: u64, mut b: u64) -> u64 {
        if b < a {
            std::mem::swap(&mut a, &mut b);
        }
        while a > 0 {
            b %= a;
            std::mem::swap(&mut a, &mut b);
        }
        b
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
