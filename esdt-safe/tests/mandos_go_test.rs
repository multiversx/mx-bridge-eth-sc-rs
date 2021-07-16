#[test]
fn claim_fees_go() {
    elrond_wasm_debug::mandos_go("mandos/distribute_fees.scen.json");
}

#[test]
fn create_another_tx_ok_go() {
    elrond_wasm_debug::mandos_go("mandos/create_another_tx_ok.scen.json");
}

#[test]
fn create_another_tx_too_late_for_batch_go() {
    elrond_wasm_debug::mandos_go("mandos/create_another_tx_too_late_for_batch.scen.json");
}

#[test]
fn create_transaction_ok_go() {
    elrond_wasm_debug::mandos_go("mandos/create_transaction_ok.scen.json");
}

#[test]
fn execute_batch_both_rejected_go() {
    elrond_wasm_debug::mandos_go("mandos/execute_batch_both_rejected.scen.json");
}

#[test]
fn execute_batch_both_success_go() {
    elrond_wasm_debug::mandos_go("mandos/execute_batch_both_success.scen.json");
}

#[test]
fn execute_batch_one_success_one_rejected_go() {
    elrond_wasm_debug::mandos_go("mandos/execute_batch_one_success_one_rejected.scen.json");
}

#[test]
fn execute_transaction_rejected_go() {
    elrond_wasm_debug::mandos_go("mandos/execute_transaction_rejected.scen.json");
}

#[test]
fn execute_transaction_success_go() {
    elrond_wasm_debug::mandos_go("mandos/execute_transaction_success.scen.json");
}

#[test]
fn get_next_pending_tx_go() {
    elrond_wasm_debug::mandos_go("mandos/get_next_pending_tx.scen.json");
}

#[test]
fn get_next_tx_batch_go() {
    elrond_wasm_debug::mandos_go("mandos/get_next_tx_batch.scen.json");
}

#[test]
fn get_next_tx_batch_too_early_go() {
    elrond_wasm_debug::mandos_go("mandos/get_next_tx_batch_too_early.scen.json");
}

#[test]
fn setup_accounts_go() {
    elrond_wasm_debug::mandos_go("mandos/setup_accounts.scen.json");
}

#[test]
fn zero_fees_go() {
    elrond_wasm_debug::mandos_go("mandos/zero_fees.scen.json");
}
