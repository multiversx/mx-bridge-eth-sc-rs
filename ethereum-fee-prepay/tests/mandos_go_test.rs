#[test]
fn setup_accounts() {
    elrond_wasm_debug::mandos_go("mandos/setup_accounts.scen.json");
}

#[test]
fn user_deposit_fee() {
    elrond_wasm_debug::mandos_go("mandos/user_deposit_fee.scen.json");
}

#[test]
fn reserve_fee() {
    elrond_wasm_debug::mandos_go("mandos/reserve_fee.scen.json");
}

#[test]
fn pay_fee() {
    elrond_wasm_debug::mandos_go("mandos/pay_fee.scen.json");
}
