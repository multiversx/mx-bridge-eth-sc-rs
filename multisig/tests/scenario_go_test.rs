#[test]
fn create_elrond_to_ethereum_tx_batch_go() {
    multiversx_sc_scenario::run_go("mandos/create_elrond_to_ethereum_tx_batch.scen.json");
}

#[test]
fn ethereum_to_elrond_tx_batch_ok_go() {
    multiversx_sc_scenario::run_go("mandos/ethereum_to_elrond_tx_batch_ok.scen.json");
}

#[test]
fn ethereum_to_elrond_tx_batch_rejected_go() {
    multiversx_sc_scenario::run_go("mandos/ethereum_to_elrond_tx_batch_rejected.scen.json");
}

#[test]
fn execute_elrond_to_ethereum_tx_batch_go() {
    multiversx_sc_scenario::run_go("mandos/execute_elrond_to_ethereum_tx_batch.scen.json");
}

#[test]
fn get_empty_batch_go() {
    multiversx_sc_scenario::run_go("mandos/get_empty_batch.scen.json");
}

#[test]
fn reject_elrond_to_ethereum_tx_batch_go() {
    multiversx_sc_scenario::run_go("mandos/reject_elrond_to_ethereum_tx_batch.scen.json");
}

#[test]
fn setup_go() {
    multiversx_sc_scenario::run_go("mandos/setup.scen.json");
}

#[test]
fn unstake_go() {
    multiversx_sc_scenario::run_go("mandos/unstake.scen.json");
}

/*
#[test]
fn upgrade_child_sc_go() {
    multiversx_sc_scenario::run_go("mandos/upgrade_child_sc.scen.json");
}
*/
