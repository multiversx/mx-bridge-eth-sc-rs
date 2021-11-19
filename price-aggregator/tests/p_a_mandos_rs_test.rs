use elrond_wasm::*;
use elrond_wasm_debug::*;

fn world() -> BlockchainMock {
    let mut blockchain = BlockchainMock::new();
    blockchain.set_current_dir_from_workspace("price-aggregator");

    blockchain.register_contract(
        "file:price-aggregator.wasm",
        Box::new(|context| Box::new(price_aggregator::contract_obj(context))),
    );
    blockchain
}

#[test]
fn deploy_rs() {
    elrond_wasm_debug::mandos_rs("mandos/deploy.scen.json", world());
}

#[test]
fn get_latest_price_feed_rs() {
    elrond_wasm_debug::mandos_rs("mandos/get_latest_price_feed.scen.json", world());
}

#[test]
fn oracle_gwei_in_eth_and_egld_submit_rs() {
    elrond_wasm_debug::mandos_rs(
        "mandos/oracle_gwei_in_eth_and_egld_submit.scen.json",
        world(),
    );
}

#[test]
fn oracle_submit_rs() {
    elrond_wasm_debug::mandos_rs("mandos/oracle_submit.scen.json", world());
}
