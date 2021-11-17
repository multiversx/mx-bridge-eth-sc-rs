use elrond_wasm::*;
use elrond_wasm_debug::*;

fn world() -> BlockchainMock {
    let mut blockchain = BlockchainMock::new();
    blockchain.set_current_dir_from_workspace("esdt-safe");

    blockchain.register_contract(
        "file:output/esdt-safe.wasm",
        Box::new(|context| Box::new(esdt_safe::contract_obj(context))),
    );
    blockchain.register_contract(
        "file:../price-aggregator/price-aggregator.wasm",
        Box::new(|context| Box::new(price_aggregator::contract_obj(context))),
    );
    blockchain
}

#[test]
fn create_another_tx_ok_rs() {
    elrond_wasm_debug::mandos_rs("mandos/create_another_tx_ok.scen.json", world());
}

#[test]
fn create_another_tx_too_late_for_batch_rs() {
    elrond_wasm_debug::mandos_rs(
        "mandos/create_another_tx_too_late_for_batch.scen.json",
        world(),
    );
}

#[test]
fn create_transaction_ok_rs() {
    elrond_wasm_debug::mandos_rs("mandos/create_transaction_ok.scen.json", world());
}

#[test]
fn distribute_fees_rs() {
    elrond_wasm_debug::mandos_rs("mandos/distribute_fees.scen.json", world());
}

#[test]
fn execute_batch_both_rejected_rs() {
    elrond_wasm_debug::mandos_rs("mandos/execute_batch_both_rejected.scen.json", world());
}

#[test]
fn execute_batch_both_success_rs() {
    elrond_wasm_debug::mandos_rs("mandos/execute_batch_both_success.scen.json", world());
}

#[test]
fn execute_batch_one_success_one_rejected_rs() {
    elrond_wasm_debug::mandos_rs(
        "mandos/execute_batch_one_success_one_rejected.scen.json",
        world(),
    );
}

#[test]
fn execute_transaction_rejected_rs() {
    elrond_wasm_debug::mandos_rs("mandos/execute_transaction_rejected.scen.json", world());
}

#[test]
fn execute_transaction_success_rs() {
    elrond_wasm_debug::mandos_rs("mandos/execute_transaction_success.scen.json", world());
}

#[test]
fn get_next_pending_tx_rs() {
    elrond_wasm_debug::mandos_rs("mandos/get_next_pending_tx.scen.json", world());
}

#[test]
fn get_next_tx_batch_rs() {
    elrond_wasm_debug::mandos_rs("mandos/get_next_tx_batch.scen.json", world());
}

#[test]
fn get_next_tx_batch_too_early_rs() {
    elrond_wasm_debug::mandos_rs("mandos/get_next_tx_batch_too_early.scen.json", world());
}

#[test]
fn setup_accounts_rs() {
    elrond_wasm_debug::mandos_rs("mandos/setup_accounts.scen.json", world());
}

#[test]
fn zero_fees_rs() {
    elrond_wasm_debug::mandos_rs("mandos/zero_fees.scen.json", world());
}
