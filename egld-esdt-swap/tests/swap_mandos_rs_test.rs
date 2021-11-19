use elrond_wasm::*;
use elrond_wasm_debug::*;

fn world() -> BlockchainMock {
    let mut blockchain = BlockchainMock::new();
    blockchain.set_current_dir_from_workspace("egld-esdt-swap");

    blockchain.register_contract(
        "file:output/egld-esdt-swap.wasm",
        Box::new(|context| Box::new(egld_esdt_swap::contract_obj(context))),
    );
    blockchain
}

#[test]
fn unwrap_egld_rs() {
    elrond_wasm_debug::mandos_rs("mandos/unwrap_egld.scen.json", world());
}

#[test]
fn wrap_egld_rs() {
    elrond_wasm_debug::mandos_rs("mandos/wrap_egld.scen.json", world());
}
