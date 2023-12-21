use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
    todo!()
}

#[test]
fn change_token_config_rs() {
    world().run("scenarios/change_token_config.scen.json");
}

#[test]
fn create_elrond_to_ethereum_tx_batch_rs() {
    world().run("scenarios/create_elrond_to_ethereum_tx_batch.scen.json");
}

#[test]
fn ethereum_to_elrond_tx_batch_ok_rs() {
    world().run("scenarios/ethereum_to_elrond_tx_batch_ok.scen.json");
}

#[test]
fn ethereum_to_elrond_tx_batch_rejected_rs() {
    world().run("scenarios/ethereum_to_elrond_tx_batch_rejected.scen.json");
}

#[test]
fn execute_elrond_to_ethereum_tx_batch_rs() {
    world().run("scenarios/execute_elrond_to_ethereum_tx_batch.scen.json");
}

#[test]
fn get_empty_batch_rs() {
    world().run("scenarios/get_empty_batch.scen.json");
}

#[test]
fn reject_elrond_to_ethereum_tx_batch_rs() {
    world().run("scenarios/reject_elrond_to_ethereum_tx_batch.scen.json");
}

#[test]
fn setup_rs() {
    world().run("scenarios/setup.scen.json");
}

#[test]
fn unstake_rs() {
    world().run("scenarios/unstake.scen.json");
}
