#[test]
fn setup_go() {
    elrond_wasm_debug::mandos_go("mandos/setup.scen.json");
}

#[test]
fn unstake_go() {
    elrond_wasm_debug::mandos_go("mandos/unstake.scen.json");
}
