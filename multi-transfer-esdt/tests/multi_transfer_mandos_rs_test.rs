use elrond_wasm::*;
use elrond_wasm_debug::*;

fn world() -> BlockchainMock {
    let mut blockchain = BlockchainMock::new();
    blockchain.set_current_dir_from_workspace("multi-transfer-esdt");

    blockchain.register_contract(
        "file:output/multi-transfer-esdt.wasm",
        Box::new(|context| Box::new(multi_transfer_esdt::contract_obj(context))),
    );    
    blockchain.register_contract(
        "file:../price-aggregator/price-aggregator.wasm",
        Box::new(|context| Box::new(price_aggregator::contract_obj(context))),
    );
    blockchain
}

#[test]
fn batch_transfer_both_executed_rs() {
    elrond_wasm_debug::mandos_rs("mandos/batch_transfer_both_executed.scen.json", world());
}

#[test]
fn batch_transfer_both_failed_rs() {
    elrond_wasm_debug::mandos_rs("mandos/batch_transfer_both_failed.scen.json", world());
}

#[test]
fn batch_transfer_one_executed_one_failed_rs() {
    elrond_wasm_debug::mandos_rs(
        "mandos/batch_transfer_one_executed_one_failed.scen.json",
        world(),
    );
}

#[test]
fn distribute_fees_rs() {
    elrond_wasm_debug::mandos_rs("mandos/distribute_fees.scen.json", world());
}

#[test]
fn setup_accounts_rs() {
    elrond_wasm_debug::mandos_rs("mandos/setup_accounts.scen.json", world());
}

#[test]
fn transfer_ok_rs() {
    elrond_wasm_debug::mandos_rs("mandos/transfer_ok.scen.json", world());
}

#[test]
fn two_transfers_same_token_rs() {
    elrond_wasm_debug::mandos_rs("mandos/two_transfers_same_token.scen.json", world());
}
