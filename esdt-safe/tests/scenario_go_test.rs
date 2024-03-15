use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
    ScenarioWorld::vm_go()
}

#[test]
fn add_refund_batch_go() {
    world().run("scenarios/add_refund_batch.scen.json");
}

#[test]
fn create_another_tx_ok_go() {
    world().run("scenarios/create_another_tx_ok.scen.json");
}

#[test]
fn create_another_tx_too_late_for_batch_go() {
    world().run("scenarios/create_another_tx_too_late_for_batch.scen.json");
}

#[test]
fn create_transaction_ok_go() {
    world().run("scenarios/create_transaction_ok.scen.json");
}

#[test]
fn create_transaction_over_max_amount_go() {
    world().run("scenarios/create_transaction_over_max_amount.scen.json");
}

#[test]
fn distribute_fees_go() {
    world().run("scenarios/distribute_fees.scen.json");
}

#[test]
fn execute_batch_both_rejected_go() {
    world().run("scenarios/execute_batch_both_rejected.scen.json");
}

#[test]
fn execute_batch_both_success_go() {
    world().run("scenarios/execute_batch_both_success.scen.json");
}

#[test]
fn execute_batch_one_success_one_rejected_go() {
    world().run("scenarios/execute_batch_one_success_one_rejected.scen.json");
}

#[test]
fn execute_transaction_rejected_go() {
    world().run("scenarios/execute_transaction_rejected.scen.json");
}

#[test]
fn execute_transaction_success_go() {
    world().run("scenarios/execute_transaction_success.scen.json");
}

#[test]
fn get_next_pending_tx_go() {
    world().run("scenarios/get_next_pending_tx.scen.json");
}

#[test]
fn get_next_tx_batch_go() {
    world().run("scenarios/get_next_tx_batch.scen.json");
}

#[test]
fn get_next_tx_batch_too_early_go() {
    world().run("scenarios/get_next_tx_batch_too_early.scen.json");
}

#[test]
fn setup_accounts_go() {
    world().run("scenarios/setup_accounts.scen.json");
}

#[test]
fn zero_fees_go() {
    world().run("scenarios/zero_fees.scen.json");
}
