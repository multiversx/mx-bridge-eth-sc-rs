#[test]
fn deploy_go() {
    elrond_wasm_debug::mandos_go("mandos/deploy.scen.json");
}

#[test]
fn get_latest_price_feed_go() {
    elrond_wasm_debug::mandos_go("mandos/get_latest_price_feed.scen.json");
}

#[test]
fn oracle_gwei_in_eth_and_egld_submit_go() {
    elrond_wasm_debug::mandos_go("mandos/oracle_gwei_in_eth_and_egld_submit.scen.json");
}

#[test]
fn oracle_submit_go() {
    elrond_wasm_debug::mandos_go("mandos/oracle_submit.scen.json");
}
