use elrond_wasm_debug::*;

fn world() -> BlockchainMock {
    let mut blockchain = BlockchainMock::new();
    blockchain.set_current_dir_from_workspace("bridged-tokens-wrapper");

    blockchain.register_contract_builder(
        "file:output/bridged-tokens-wrapper.wasm".into(),
        bridged_tokens_wrapper::ContractBuilder,
    );
    blockchain
}
#[test]
fn unwrap_token_rs() {
    elrond_wasm_debug::mandos_rs("mandos/unwrap_token.scen.json", world());
}

#[test]
fn wrap_token_rs() {
    elrond_wasm_debug::mandos_rs("mandos/wrap_token.scen.json", world());
}

#[test]
fn whitelist_token_rs() {
    elrond_wasm_debug::mandos_rs("mandos/whitelist_token.scen.json", world());
}

#[test]
fn blacklist_token_rs() {
    elrond_wasm_debug::mandos_rs("mandos/blacklist_token.scen.json", world());
}

#[test]
fn add_wrapped_token_rs() {
    elrond_wasm_debug::mandos_rs("mandos/add_wrapped_token.scen.json", world());
}

#[test]
fn remove_wrapped_token_rs() {
    elrond_wasm_debug::mandos_rs("mandos/remove_wrapped_token.scen.json", world());
}
