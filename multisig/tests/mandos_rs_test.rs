use elrond_wasm_debug::*;

fn world() -> BlockchainMock {
    let mut blockchain = BlockchainMock::new();
    blockchain.set_current_dir_from_workspace("multisig");

    blockchain.register_contract_builder(
        "file:../egld-esdt-swap/output/egld-esdt-swap.wasm",
        egld_esdt_swap::ContractBuilder,
    );
    blockchain.register_contract_builder(
        "file:../multi-transfer-esdt/output/multi-transfer-esdt.wasm",
        multi_transfer_esdt::ContractBuilder,
    );
    blockchain.register_contract_builder(
        "file:../esdt-safe/output/esdt-safe.wasm",
        esdt_safe::ContractBuilder,
    );
    blockchain.register_contract_builder("file:output/multisig.wasm", multisig::ContractBuilder);
    blockchain
}

#[test]
fn bug_hunt_rs() {
    elrond_wasm_debug::mandos_rs("mandos/bug_hunt.scen.json", world());
}
