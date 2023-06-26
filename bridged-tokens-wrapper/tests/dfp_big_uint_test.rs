#![feature(associated_type_bounds)]
use bridged_tokens_wrapper::DFPBigUint;
use multiversx_sc_scenario::DebugApi;

#[test]
fn test_biguint() {
    let _ = DebugApi::dummy();
    let raw = 123456u64;
    let dfp = DFPBigUint::<DebugApi>::from_raw(raw.into(), 6);
    let converted = dfp.clone().convert(9);
    assert!(dfp.trunc() == converted.trunc());
    assert!(converted.clone().convert(9).to_raw() == 123456000u64);
    assert!(converted.clone().convert(1).to_raw() == 1u64);
    assert!(converted.clone().convert(3).to_raw() == 123u64);
    assert!(converted.clone().convert(5).to_raw() == 12345u64);
}

#[test]
fn test_mandos_scenario_values() {
    let _ = DebugApi::dummy();
    let raw = 300000000000000u64;
    let dfp = DFPBigUint::<DebugApi>::from_raw(raw.into(), 18);
    assert!(dfp.convert(6).to_raw() == 300u64);
}
