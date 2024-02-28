use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
    ScenarioWorld::vm_go()
}
#[test]
fn change_token_config_go() {
    world().run("scenarios/change_token_config.scen.json");
}

#[test]
fn create_multiversx_to_ethereum_tx_batch_go() {
    world().run("scenarios/create_multiversx_to_ethereum_tx_batch.scen.json");
}

#[test]
fn ethereum_to_multiversx_tx_batch_ok_go() {
    world().run("scenarios/ethereum_to_multiversx_tx_batch_ok.scen.json");
}

#[test]
fn ethereum_to_multiversx_tx_batch_rejected_go() {
    world().run("scenarios/ethereum_to_multiversx_tx_batch_rejected.scen.json");
}

#[test]
fn execute_multiversx_to_ethereum_tx_batch_go() {
    world().run("scenarios/execute_multiversx_to_ethereum_tx_batch.scen.json");
}

#[test]
fn get_empty_batch_go() {
    world().run("scenarios/get_empty_batch.scen.json");
}

#[test]
fn reject_multiversx_to_ethereum_tx_batch_go() {
    world().run("scenarios/reject_multiversx_to_ethereum_tx_batch.scen.json");
}

#[test]
fn setup_go() {
    world().run("scenarios/setup.scen.json");
}

#[test]
fn unstake_go() {
    world().run("scenarios/unstake.scen.json");
}
