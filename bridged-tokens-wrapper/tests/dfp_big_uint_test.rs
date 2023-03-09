#![feature(associated_type_bounds)]
use bridged_tokens_wrapper::DFPBigUint;
use elrond_wasm::types::BigUint;
use elrond_wasm_debug::DebugApi;

#[test]
fn test_biguint() {
    let _ = DebugApi::dummy();
    let raw = BigUint::<DebugApi>::from(123456u64);
    let dfp = DFPBigUint::from_raw(raw.clone(), 6);
    let converted = dfp.convert(9);
    assert!(dfp.trunc() == converted.trunc());
    assert!(converted.convert(9).to_raw() == BigUint::<DebugApi>::from(123456000u64));
    assert!(converted.convert(1).to_raw() == BigUint::<DebugApi>::from(1u64));
    assert!(converted.convert(3).to_raw() == BigUint::<DebugApi>::from(123u64));
    assert!(converted.convert(5).to_raw() == BigUint::<DebugApi>::from(12345u64));
}
