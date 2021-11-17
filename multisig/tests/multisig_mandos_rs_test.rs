use elrond_wasm::*;
use elrond_wasm_debug::*;

fn world() -> BlockchainMock {
    let mut blockchain = BlockchainMock::new();
    blockchain.set_current_dir_from_workspace("multisig");

    blockchain.register_contract(
        "file:output/multisig.wasm",
        Box::new(|context| Box::new(multisig::contract_obj(context))),
    );
    blockchain.register_contract(
        "file:../egld-esdt-swap/output/egld-esdt-swap.wasm",
        Box::new(|context| Box::new(egld_esdt_swap::contract_obj(context))),
    );
    blockchain.register_contract(
        "file:../esdt-safe/output/esdt-safe.wasm",
        Box::new(|context| Box::new(esdt_safe::contract_obj(context))),
    );
    blockchain.register_contract(
        "file:../multi-transfer-esdt/output/multi-transfer-esdt.wasm",
        Box::new(|context| Box::new(multi_transfer_esdt::contract_obj(context))),
    );
    blockchain.register_contract(
        "file:../price-aggregator/price-aggregator.wasm",
        Box::new(|context| Box::new(price_aggregator::contract_obj(context))),
    );
    blockchain
}

#[test]
fn change_token_config_rs() {
    elrond_wasm_debug::mandos_rs("mandos/change_token_config.scen.json", world());
}

#[test]
fn create_elrond_to_ethereum_tx_batch_rs() {
    elrond_wasm_debug::mandos_rs(
        "mandos/create_elrond_to_ethereum_tx_batch.scen.json",
        world(),
    );
}

#[test]
fn ethereum_to_elrond_tx_batch_ok_rs() {
    elrond_wasm_debug::mandos_rs("mandos/ethereum_to_elrond_tx_batch_ok.scen.json", world());
}

#[test]
fn ethereum_to_elrond_tx_batch_rejected_rs() {
    elrond_wasm_debug::mandos_rs(
        "mandos/ethereum_to_elrond_tx_batch_rejected.scen.json",
        world(),
    );
}

#[test]
fn execute_elrond_to_ethereum_tx_batch_rs() {
    elrond_wasm_debug::mandos_rs(
        "mandos/execute_elrond_to_ethereum_tx_batch.scen.json",
        world(),
    );
}

#[test]
fn get_empty_batch_rs() {
    elrond_wasm_debug::mandos_rs("mandos/get_empty_batch.scen.json", world());
}

#[test]
fn reject_elrond_to_ethereum_tx_batch_rs() {
    elrond_wasm_debug::mandos_rs(
        "mandos/reject_elrond_to_ethereum_tx_batch.scen.json",
        world(),
    );
}

#[test]
fn setup_rs() {
    elrond_wasm_debug::mandos_rs("mandos/setup.scen.json", world());
}

#[test]
fn unstake_rs() {
    elrond_wasm_debug::mandos_rs("mandos/unstake.scen.json", world());
}

#[test]
fn upgrade_child_sc_rs() {
    elrond_wasm_debug::mandos_rs("mandos/upgrade_child_sc.scen.json", world());
}
