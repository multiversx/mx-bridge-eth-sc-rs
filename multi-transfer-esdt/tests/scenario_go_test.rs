use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
    ScenarioWorld::vm_go()
}

#[test]
fn basic_transfer_test_go() {
    world().run("scenarios/basic_transfer_test.scen.json");
}

#[test]
#[ignore] //Ignore for now
fn batch_transfer_both_executed_go() {
    world().run("scenarios/batch_transfer_both_executed.scen.json");
}

#[test]
#[ignore] //Ignore for now
fn batch_transfer_both_failed_go() {
    world().run("scenarios/batch_transfer_both_failed.scen.json");
}

#[test]
#[ignore] //Ignore for now
fn batch_transfer_one_executed_one_failed_go() {
    world().run("scenarios/batch_transfer_one_executed_one_failed.scen.json");
}

#[test]
#[ignore] //Ignore for now
fn batch_transfer_to_frozen_account_go() {
    world().run("scenarios/batch_transfer_to_frozen_account.scen.json");
}

#[test]
#[ignore] //Ignore for now
fn batch_transfer_with_wrapping_go() {
    world().run("scenarios/batch_transfer_with_wrapping.scen.json");
}

#[test]
fn setup_accounts_go() {
    world().run("scenarios/setup_accounts.scen.json");
}

#[test]
fn transfer_fail_mint_burn_not_allowed_go() {
    world().run("scenarios/transfer_fail_mint_burn_not_allowed.scen.json");
}

#[test]
#[ignore] //Ignore for now
fn transfer_ok_go() {
    world().run("scenarios/transfer_ok.scen.json");
}

#[test]
#[ignore] //Ignore for now
fn two_transfers_same_token_go() {
    world().run("scenarios/two_transfers_same_token.scen.json");
}
