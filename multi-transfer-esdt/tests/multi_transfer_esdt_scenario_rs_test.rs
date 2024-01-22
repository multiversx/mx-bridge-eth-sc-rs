use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();
    blockchain.set_current_dir_from_workspace("multi-transfer-esdt/");

    blockchain.register_contract("file:output/multi-transfer-esdt.wasm", multi_transfer_esdt::ContractBuilder);
    blockchain.register_contract("file:../esdt-safe/output/esdt-safe.wasm", esdt_safe::ContractBuilder);
    blockchain.register_contract("file:../bridge-proxy/output/bridge-proxy.wasm", bridge_proxy::ContractBuilder);
    blockchain.register_contract("file:../bridged-tokens-wrapper/output/bridged-tokens-wrapper.wasm", bridged_tokens_wrapper::ContractBuilder);

    blockchain
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
fn transfer_fail_mint_burn_not_allowed_rs() {
    world().run("scenarios/transfer_fail_mint_burn_not_allowed.scen.json");
}

#[test]
fn transfer_ok_rs() {
    world().run("scenarios/transfer_ok.scen.json");
}

#[test]
fn two_transfers_same_token_rs() {
    world().run("scenarios/two_transfers_same_token.scen.json");
}
