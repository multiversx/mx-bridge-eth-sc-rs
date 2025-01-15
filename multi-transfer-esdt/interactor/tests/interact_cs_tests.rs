use multiversx_sc_snippets::imports::*;
use rust_interact::{Config, ContractInteract};

// Simple deploy test that runs using the chain simulator configuration.
// In order for this test to work, make sure that the `config.toml` file contains the chain simulator config (or choose it manually)
// The chain simulator should already be installed and running before attempting to run this test.
// The chain-simulator-tests feature should be present in Cargo.toml.
// Can be run with `sc-meta test -c`.
#[tokio::test]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn deploy_test_multi_transfer_esdt_cs() {
    let mut interactor = ContractInteract::new(Config::new()).await;

    let mut interactor_cs = ContractInteract::new(Config::chain_simulator_config()).await;

    interactor
        .interactor
        .retrieve_account(&Bech32Address::from(interactor.wallet_address.clone()))
        .await;

    interactor_cs
        .interactor
        .set_state_for_saved_accounts()
        .await
        .unwrap();

    let address = interactor_cs.deploy().await;

    // let account = interactor
    //     .interactor
    //     .get_account(&interactor.wallet_address)
    //     .await;

    // let a = interactor
    //     .interactor
    //     .proxy
    //     .get_account_esdt_tokens(&interactor.wallet_address)
    //     .await;
    // println!("{:?}", account);
    // println!("{:?}", a);

    interactor_cs.my_transfer(address.clone()).await;

    interactor_cs
        .my_distribute_payments(interactor_cs.wallet_address.clone().into())
        .await;

    interactor_cs.my_transfer(address).await;
}
