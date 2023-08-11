use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(
        "file:output/bridge-proxy.wasm",
        bridge_proxy::ContractBuilder,
    );
    blockchain
}

#[test]
fn bridge_proxy_rs() {
    world().run("scenarios/bridge-proxy.scen.json");
}
