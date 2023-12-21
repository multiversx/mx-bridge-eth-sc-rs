use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
    todo!()
}

#[test]
fn batch_transfer_both_executed_rs() {
    world().run("scenarios/batch_transfer_both_executed.scen.json");
}

#[test]
fn batch_transfer_both_failed_rs() {
    world().run("scenarios/batch_transfer_both_failed.scen.json");
}

#[test]
fn batch_transfer_one_executed_one_failed_rs() {
    world().run("scenarios/batch_transfer_one_executed_one_failed.scen.json");
}

#[test]
fn batch_transfer_to_frozen_account_rs() {
    world().run("scenarios/batch_transfer_to_frozen_account.scen.json");
}

#[test]
fn batch_transfer_with_wrapping_rs() {
    world().run("scenarios/batch_transfer_with_wrapping.scen.json");
}

#[test]
fn setup_accounts_rs() {
    world().run("scenarios/setup_accounts.scen.json");
}

#[test]
fn transfer_ok_rs() {
    world().run("scenarios/transfer_ok.scen.json");
}

#[test]
fn two_transfers_same_token_rs() {
    world().run("scenarios/two_transfers_same_token.scen.json");
}
