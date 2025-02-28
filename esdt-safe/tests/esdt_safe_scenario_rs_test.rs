use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();
    blockchain.set_current_dir_from_workspace("esdt-safe");

    blockchain.register_contract("file:output/esdt-safe.mxsc.json", esdt_safe::ContractBuilder);
    blockchain.register_contract(
        "file:../price-aggregator/multiversx-price-aggregator-sc.mxsc.json",
        multiversx_price_aggregator_sc::ContractBuilder,
    );
    blockchain.register_contract(
        "file:../common/mock-contracts/mock-multi-transfer-esdt/output/mock-multi-transfer-esdt.mxsc.json",
        mock_multi_transfer_esdt::ContractBuilder,
    );
    blockchain.register_contract(
        "file:../common/mock-contracts/mock-multisig/output/mock-multisig.mxsc.json",
        mock_multisig::ContractBuilder,
    );

    blockchain
}

#[test]
fn add_refund_batch_rs() {
    world().run("scenarios/add_refund_batch.scen.json");
}

#[test]
fn create_another_tx_ok_rs() {
    world().run("scenarios/create_another_tx_ok.scen.json");
}

#[test]
fn create_another_tx_too_late_for_batch_rs() {
    world().run("scenarios/create_another_tx_too_late_for_batch.scen.json");
}

#[test]
fn create_transaction_ok_rs() {
    world().run("scenarios/create_transaction_ok.scen.json");
}

#[test]
fn create_transaction_over_max_amount_rs() {
    world().run("scenarios/create_transaction_over_max_amount.scen.json");
}

#[test]
fn distribute_fees_rs() {
    world().run("scenarios/distribute_fees.scen.json");
}

#[test]
fn execute_batch_both_rejected_rs() {
    world().run("scenarios/execute_batch_both_rejected.scen.json");
}

#[test]
fn execute_batch_both_success_rs() {
    world().run("scenarios/execute_batch_both_success.scen.json");
}

#[test]
fn execute_batch_one_success_one_rejected_rs() {
    world().run("scenarios/execute_batch_one_success_one_rejected.scen.json");
}

#[test]
fn execute_transaction_rejected_rs() {
    world().run("scenarios/execute_transaction_rejected.scen.json");
}

#[test]
fn execute_transaction_success_rs() {
    world().run("scenarios/execute_transaction_success.scen.json");
}

#[test]
fn get_next_pending_tx_rs() {
    world().run("scenarios/get_next_pending_tx.scen.json");
}

#[test]
fn get_next_tx_batch_rs() {
    world().run("scenarios/get_next_tx_batch.scen.json");
}

#[test]
fn get_next_tx_batch_too_early_rs() {
    world().run("scenarios/get_next_tx_batch_too_early.scen.json");
}

#[test]
fn setup_accounts_rs() {
    world().run("scenarios/setup_accounts.scen.json");
}

#[test]
fn zero_fees_rs() {
    world().run("scenarios/zero_fees.scen.json");
}


#[test]
fn upgrade_test() {
    world().run("scenarios/upgrade_test_generated.scen.json");
}

#[test]
fn deploy_aggregator() {
    world().run("../price-aggregator/scenarios/deploy.scen.json");
}