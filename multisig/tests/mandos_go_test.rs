#[test]
fn setup_go() {
    elrond_wasm_debug::mandos_go("mandos/setup.scen.json");
}

#[test]
fn unstake_go() {
    elrond_wasm_debug::mandos_go("mandos/unstake.scen.json");
}

#[test]
fn create_elrond_to_ethereum_tx_batch_go() {
    elrond_wasm_debug::mandos_go("mandos/create_elrond_to_ethereum_tx_batch.scen.json");
}

#[test]
fn execute_elrond_to_ethereum_tx_batch_go() {
    elrond_wasm_debug::mandos_go("mandos/execute_elrond_to_ethereum_tx_batch.scen.json");
}

#[test]
fn reject_elrond_to_ethereum_tx_batch_go() {
    elrond_wasm_debug::mandos_go("mandos/reject_elrond_to_ethereum_tx_batch.scen.json");
}
