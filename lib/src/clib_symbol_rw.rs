use crate::symbol_rw;
use std::slice::from_raw_parts;
use std::slice::from_raw_parts_mut;

#[no_mangle]
pub unsafe extern "C" fn encode_stream(
    numerator: u64,
    denominator: u64,
    raw_bits: *mut u8,
    len: usize,
) -> bool {
    let bits = from_raw_parts_mut(raw_bits, len);
    symbol_rw::encode_stream(
        numerator,
        core::num::NonZeroU64::new(denominator).unwrap(),
        bits,
    )
}

#[no_mangle]
pub unsafe extern "C" fn decode_stream(
    numerator: *mut u64,
    denominator: *mut u64,
    raw_bits: *const u8,
    len: usize,
) {
    let bits = unsafe { from_raw_parts(raw_bits, len) };
    let (num, denom) = symbol_rw::decode_stream(bits);
    *numerator = num;
    *denominator = denom.get();
}

#[no_mangle]
pub unsafe extern "C" fn encode_c8(result: *mut u8, numerator: u64, denominator: u64) -> bool {
    let (value, status) =
        match symbol_rw::encode_c8(numerator, core::num::NonZeroU64::new(denominator).unwrap()) {
            Ok(val) => (val, true),
            Err(val) => (val, false),
        };
    *result = value;
    status
}

#[no_mangle]
pub unsafe extern "C" fn decode_c8(numerator: *mut u64, denominator: *mut u64, bits: u8) {
    let (num, denom) = symbol_rw::decode_c8(bits);
    *numerator = num;
    *denominator = denom.get();
}

#[no_mangle]
pub unsafe extern "C" fn encode_c16(result: *mut u16, numerator: u64, denominator: u64) -> bool {
    let (value, status) =
        match symbol_rw::encode_c16(numerator, core::num::NonZeroU64::new(denominator).unwrap()) {
            Ok(val) => (val, true),
            Err(val) => (val, false),
        };
    *result = value;
    status
}

#[no_mangle]
pub unsafe extern "C" fn decode_c16(numerator: *mut u64, denominator: *mut u64, bits: u16) {
    let (num, denom) = symbol_rw::decode_c16(bits);
    *numerator = num;
    *denominator = denom.get();
}

#[no_mangle]
pub unsafe extern "C" fn encode_c32(result: *mut u32, numerator: u64, denominator: u64) -> bool {
    let (value, status) =
        match symbol_rw::encode_c32(numerator, core::num::NonZeroU64::new(denominator).unwrap()) {
            Ok(val) => (val, true),
            Err(val) => (val, false),
        };
    *result = value;
    status
}

#[no_mangle]
pub unsafe extern "C" fn decode_c32(numerator: *mut u64, denominator: *mut u64, bits: u32) {
    let (num, denom) = symbol_rw::decode_c32(bits);
    *numerator = num;
    *denominator = denom.get();
}

#[no_mangle]
pub unsafe extern "C" fn encode_c64(result: *mut u64, numerator: u64, denominator: u64) -> bool {
    let (value, status) =
        match symbol_rw::encode_c64(numerator, core::num::NonZeroU64::new(denominator).unwrap()) {
            Ok(val) => (val, true),
            Err(val) => (val, false),
        };
    *result = value;
    status
}

#[no_mangle]
pub unsafe extern "C" fn decode_c64(numerator: *mut u64, denominator: *mut u64, bits: u64) {
    let (num, denom) = symbol_rw::decode_c64(bits);
    *numerator = num;
    *denominator = denom.get();
}
