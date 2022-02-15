use core::num::NonZeroU32;
use core::num::NonZeroU64;

fn cf_to_rational32<I: Iterator<Item = NonZeroU32>>(iter_rev: I) -> Option<(i32, NonZeroU32)> {
    let (mut num, mut den) = iter_rev.fold((1_i32, 0_i32), |(num, den), item| {
        ((item.get() as i32) * num + den, num)
    });
    if den < 0 {
        num *= -1;
        den *= -1;
    }
    NonZeroU32::new(den as u32).map(|d| (num, d))
}

fn cf_to_rational64<I: Iterator<Item = NonZeroU64>>(iter_rev: I) -> (i64, NonZeroU64) {
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
    fn test_cf_to_rational64() {
        let encodeds: Vec<_> = (0_u8..=0x0F)
            .map(|byte| {
                let bits = byte.to_be_bytes();
                let mut bitstream = BitDecoder::new(&bits[..]);
                for _ in 0..4 {
                    bitstream.read_bit();
                }
                let coder = Coder::default();
                cf_to_rational64(coder.read_iter(bitstream))
            })
            .collect();
        println!("{:?}", encodeds);

        let mut cache = std::collections::HashSet::new();
        assert!(encodeds.into_iter().all(move |item| cache.insert(item)));
    }
}
