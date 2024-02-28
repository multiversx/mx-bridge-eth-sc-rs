#![allow(deprecated)]

pub mod multisig_setup;
use multisig_setup::*;

#[test]
fn setup_test() {
    let _ = MultisigSetup::new(
        multiversx_price_aggregator_sc::contract_obj,
        esdt_safe::contract_obj,
        multi_transfer_esdt::contract_obj,
        multisig::contract_obj,
    );
}
