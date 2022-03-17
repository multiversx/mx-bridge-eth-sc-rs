#[test]
fn unwrap_usdc_go() {
    elrond_wasm_debug::mandos_go("mandos/unwrap_usdc.scen.json");
}

#[test]
fn wrap_usdc_go() {
    elrond_wasm_debug::mandos_go("mandos/wrap_usdc.scen.json");
}

#[test]
fn whitelist_usdc_go() {
    elrond_wasm_debug::mandos_go("mandos/whitelist_usdc.scen.json");
}

#[test]
fn blacklist_usdc_go() {
    elrond_wasm_debug::mandos_go("mandos/blacklist_usdc.scen.json");
}
