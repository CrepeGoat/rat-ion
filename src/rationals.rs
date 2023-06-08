use crate::bitslice::{BitDecoder, BitEncoder};
use crate::sbs_main::Coder;
use crate::utils::IncompleteInt;

use core::num::NonZeroU64;
use std::mem::size_of;

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
                if prev_items[1] != None
                    && prev_items[0] == Some(Ok(NonZeroU64::new(1).expect("known to be non-zero")))
                {
                    Some(NonZeroU64::new(range.start.get() + 1).expect("known to be non-zero"))
                } else {
                    None
                }
            }
            Err(IncompleteInt::Bounded(range, _)) => {
                // Encodings should prefer to first cover all smaller-denominator values
                //   -> each ambiguous encoding should have the smallest denominator possible
                // https://en.wikipedia.org/wiki/Continued_fraction#Best_rational_within_an_interval
                Some(NonZeroU64::new(range.start().get() + 1).expect("known to be non-zero"))
            }
        };
        prev_items[1] = prev_items[0].clone();
        prev_items[0] = Some(inc_int);
        result
    })
}

pub fn rational64_to_cf(
    mut numerator: u64,
    denominator: NonZeroU64,
) -> impl Iterator<Item = NonZeroU64> {
    let mut denominator = denominator.get();
    if numerator > denominator {
        panic!(
            "fraction must be between 0 and 1; got ({:?} / {:?})",
            numerator, denominator
        );
    }
    std::iter::from_fn(move || {
        if numerator <= 0 {
            return None;
        }
        let next_value = NonZeroU64::new(denominator / numerator).expect("checked that den > num");
        (numerator, denominator) = (denominator % numerator, numerator);
        Some(next_value)
    })
}

pub fn cf_to_rational64<I: Iterator<Item = NonZeroU64>>(iter_rev: I) -> (u64, NonZeroU64) {
    let (den, num) = iter_rev.fold((1_u64, 0_u64), |(num, den), item| {
        (item.get() * num + den, num)
    });
    (
        num,
        NonZeroU64::new(den as u64).expect("starts at 1 & increases"),
    )
}

pub fn encode_c8(numerator: u64, denominator: NonZeroU64) -> Result<u8, u8> {
    let mut bits: [u8; size_of::<u8>()] = [0];
    let mut bitstream = BitEncoder::new(&mut bits[..]);
    let mut coder = Coder::default();
    let mut is_truncated: bool = false;

    for value in rational64_to_cf(numerator, denominator) {
        if coder.write(&mut bitstream, value).is_err() {
            is_truncated = true;
            break;
        }
    }
    coder.write_inf(&mut bitstream);

    let result = u8::from_be_bytes(bits);
    if is_truncated {
        Err(result)
    } else {
        Ok(result)
    }
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
    fn test_c8_decode_encode_inverse() {
        let encodings: Vec<_> = (0_u8..=0xFF).collect();
        let decodings = encodings.iter().map(|&byte| decode_c8(&byte));
        let encodings2: Vec<_> = decodings
            .map(|(numerator, denominator)| encode_c8(numerator, denominator).unwrap_or_else(|x| x))
            .collect();

        assert_eq!(encodings, encodings2);
    }

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
