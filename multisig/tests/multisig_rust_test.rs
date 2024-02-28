#![allow(deprecated)]

pub mod multisig_setup;
use eth_address::EthAddress;
use multisig::multisig_general::MultisigGeneralModule;
use multisig::Multisig;
use multisig_setup::*;
use multiversx_sc::types::{ManagedByteArray, ManagedVec, MultiValueEncoded};
use multiversx_sc_scenario::{
    managed_address, managed_biguint, managed_buffer, managed_token_id, rust_biguint,
};
use transaction::call_data::CallData;

#[test]
fn setup_test() {
    let _ = MultisigSetup::new(
        multiversx_price_aggregator_sc::contract_obj,
        esdt_safe::contract_obj,
        multi_transfer_esdt::contract_obj,
        multisig::contract_obj,
    );
}

#[test]
fn eth_to_mx_transfer_both_rejected_test() {
    let mut setup = MultisigSetup::new(
        multiversx_price_aggregator_sc::contract_obj,
        esdt_safe::contract_obj,
        multi_transfer_esdt::contract_obj,
        multisig::contract_obj,
    );

    let dest_sc = setup.b_mock.create_sc_account(
        &rust_biguint!(0),
        Some(&setup.owner_addr),
        esdt_safe::contract_obj,
        "other esdt safe wasm path",
    );

    setup
        .b_mock
        .execute_tx(
            &setup.relayer_1,
            &setup.multisig_wrapper,
            &rust_biguint!(0),
            |sc| {
                let mut transfers = MultiValueEncoded::new();
                let user_eth_addr = EthAddress {
                    raw_addr: ManagedByteArray::new_from_bytes(&[
                        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
                    ]),
                };
                transfers.push(
                    (
                        user_eth_addr.clone(),
                        managed_address!(dest_sc.address_ref()),
                        managed_token_id!(WEGLD_TOKEN_ID),
                        managed_biguint!(76_000_000_000),
                        1,
                        Some(CallData {
                            endpoint: managed_buffer!(b"data"),
                            gas_limit: 5_000_000,
                            args: ManagedVec::new(),
                        }),
                    )
                        .into(),
                );
                transfers.push(
                    (
                        user_eth_addr,
                        managed_address!(dest_sc.address_ref()),
                        managed_token_id!(ETH_TOKEN_ID),
                        managed_biguint!(76_000_000_000),
                        2,
                        Some(CallData {
                            endpoint: managed_buffer!(b"data"),
                            gas_limit: 5_000_000,
                            args: ManagedVec::new(),
                        }),
                    )
                        .into(),
                );

                let action_id = sc.propose_multi_transfer_esdt_batch(1, transfers);
                assert_eq!(action_id, 1);
            },
        )
        .assert_ok();

    setup
        .b_mock
        .execute_tx(
            &setup.relayer_2,
            &setup.multisig_wrapper,
            &rust_biguint!(0),
            |sc| {
                sc.sign(1);
            },
        )
        .assert_ok();

    setup
        .b_mock
        .execute_tx(
            &setup.relayer_1,
            &setup.multisig_wrapper,
            &rust_biguint!(0),
            |sc| {
                sc.perform_action_endpoint(1);
            },
        )
        .assert_ok();
}
