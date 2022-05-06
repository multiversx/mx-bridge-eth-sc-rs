#[test]
fn batch_transfer_both_executed_go() {
    elrond_wasm_debug::mandos_go("mandos/batch_transfer_both_executed.scen.json");
}

#[test]
fn batch_transfer_both_failed_go() {
    elrond_wasm_debug::mandos_go("mandos/batch_transfer_both_failed.scen.json");
}

#[test]
fn batch_transfer_one_executed_one_failed_go() {
    elrond_wasm_debug::mandos_go("mandos/batch_transfer_one_executed_one_failed.scen.json");
}

#[test]
fn batch_transfer_to_frozen_account_go() {
    elrond_wasm_debug::mandos_go("mandos/batch_transfer_to_frozen_account.scen.json");
}

#[test]
fn setup_accounts_go() {
    elrond_wasm_debug::mandos_go("mandos/setup_accounts.scen.json");
}

#[test]
fn transfer_ok_go() {
    elrond_wasm_debug::mandos_go("mandos/transfer_ok.scen.json");
}

#[test]
fn two_transfers_same_token_go() {
    elrond_wasm_debug::mandos_go("mandos/two_transfers_same_token.scen.json");
}
