use eth_address::EthAddress;
use fee_estimator_module::FeeEstimatorModule;
use max_bridged_amount_module::MaxBridgedAmountModule;
use multiversx_sc_modules::pause::PauseModule;
use multiversx_sc_scenario::imports::*;
use esdt_safe::EsdtSafe;
use token_module::TokenModule;
use transaction::Transaction;
use transaction::transaction_status::TransactionStatus;
use tx_batch_module::TxBatchModule;
use multi_transfer_esdt::MultiTransferEsdt;

const OWNER_ADDRESS_EXPR: &str = "address:owner";
const SIGNER_0_ADDRESS_EXPR: &str = "address:signer0";
const ETH_ADDRESS: &str = "0x0x2E110BBe2eEcd819c721D1a4fb91F3c33BDF0798";
const ESTD_SAFE_ADDRESS_EXPR: &str = "sc:esdt-safe";
const MULTI_TRANSFER_ADDRESS_EXPR: &str = "sc:multi-transfer";
const ESTD_SAFE_PATH_EXPR: &str = "mxsc:output/esdt-safe.mxsc.json";
const MULTI_TRANSFER_PATH_EXPR: &str = "mxsc:output/multi-transfer-esdt.mxsc.json";
const TOKEN_ID: &[u8] = b"TOKEN-abc123";
const TOKEN_ID_2: &[u8] = b"TOKEN-xyz789";


fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(
        ESTD_SAFE_PATH_EXPR,
        multiversx_price_aggregator_sc::ContractBuilder,
    );

    blockchain
}


#[test]
fn test_create_transaction_should_fail_case_1() {
    let mut world = setup();

    let esdt_safe_whitebox = WhiteboxContract::new(
        ESTD_SAFE_ADDRESS_EXPR,
        esdt_safe::contract_obj,
    );

    world.whitebox_call(
        &esdt_safe_whitebox,
        ScCallStep::new().from(OWNER_ADDRESS_EXPR),
        |sc| {
            sc.paused_status().set(true);
        }
    );

    const ESDT_TRANSFER_AMOUNT: u32 = 100u32;

    world.whitebox_call_check(
        &esdt_safe_whitebox,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR)
            .esdt_transfer(TOKEN_ID, 0, ESDT_TRANSFER_AMOUNT)
            .expect(TxExpect::user_error("str:Cannot create transaction while paused")),
        |sc| {
            sc.create_transaction(convert_to_eth_address(ETH_ADDRESS))
        },
        |r| r.assert_user_error("Cannot create transaction while paused"),
    );
}

#[test]
fn test_create_transaction_should_fail_case_2() {
    let mut world = setup();

    let esdt_safe_whitebox = WhiteboxContract::new(
        ESTD_SAFE_ADDRESS_EXPR,
        esdt_safe::contract_obj,
    );

    world.whitebox_call(
        &esdt_safe_whitebox,
        ScCallStep::new().from(OWNER_ADDRESS_EXPR),
        |sc| {
            sc.paused_status().set(false);
        }
    );

    const ESDT_TRANSFER_AMOUNT: u32 = 100u32;

    world.whitebox_call_check(
        &esdt_safe_whitebox,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR)
            .esdt_transfer(TOKEN_ID, 0, ESDT_TRANSFER_AMOUNT)
            .expect(TxExpect::user_error("str:Token not in whitelist")),
        |sc| {
            sc.create_transaction(convert_to_eth_address(ETH_ADDRESS))
        },
        |r| r.assert_user_error("Token not in whitelist"),
    );
}

//TODO: Add tests with fee-estimator contract integrated


#[test]
fn test_create_transaction_should_work_case_1() {
    let mut world = setup();

    let esdt_safe_whitebox = WhiteboxContract::new(
        ESTD_SAFE_ADDRESS_EXPR,
        esdt_safe::contract_obj,
    );

    const MAX_AMOUNT: u32 = 100_000_000u32;

    world.whitebox_call(
        &esdt_safe_whitebox,
        ScCallStep::new().from(OWNER_ADDRESS_EXPR),
        |sc| {
            sc.paused_status().set(false);
            sc.token_whitelist().insert(TokenIdentifier::from(TOKEN_ID));
            sc.max_bridged_amount(&TokenIdentifier::from(TOKEN_ID)).set(BigUint::from(MAX_AMOUNT));
            sc.mint_burn_token(&TokenIdentifier::from(TOKEN_ID)).set(true);
            sc.native_token(&TokenIdentifier::from(TOKEN_ID)).set(true);
        }
    );

    world.set_esdt_local_roles(
        managed_address!(&AddressValue::from(ESTD_SAFE_ADDRESS_EXPR).to_address()),
        TOKEN_ID,
        &[EsdtLocalRole::Mint, EsdtLocalRole::Burn]
    );

    const ESDT_TRANSFER_AMOUNT: u32 = 100u32;

    world.whitebox_call(
        &esdt_safe_whitebox,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR)
            .esdt_transfer(TOKEN_ID, 0, ESDT_TRANSFER_AMOUNT),
        |sc| {
            let burned_amount_before = sc.burn_balances(&TokenIdentifier::from(TOKEN_ID)).get();

            sc.create_transaction(convert_to_eth_address(ETH_ADDRESS));

            let burned_amount_after = sc.burn_balances(&TokenIdentifier::from(TOKEN_ID)).get();

            assert_eq!(burned_amount_after, burned_amount_before + ESDT_TRANSFER_AMOUNT);
        },
    );
}

#[test]
fn test_create_transaction_should_work_case_2() {
    let mut world = setup();

    let esdt_safe_whitebox = WhiteboxContract::new(
        ESTD_SAFE_ADDRESS_EXPR,
        esdt_safe::contract_obj,
    );

    const MAX_AMOUNT: u32 = 100_000_000u32;

    world.whitebox_call(
        &esdt_safe_whitebox,
        ScCallStep::new().from(OWNER_ADDRESS_EXPR),
        |sc| {
            sc.paused_status().set(false);
            sc.token_whitelist().insert(TokenIdentifier::from(TOKEN_ID));
            sc.max_bridged_amount(&TokenIdentifier::from(TOKEN_ID)).set(BigUint::from(MAX_AMOUNT));
            sc.mint_burn_token(&TokenIdentifier::from(TOKEN_ID)).set(true);
            sc.native_token(&TokenIdentifier::from(TOKEN_ID)).set(false);
        }
    );

    const MINTED_AMOUNT: u32 = 201u32;
    const BURNED_AMOUNT: u32 = 100u32;

    world.whitebox_call(
        &esdt_safe_whitebox,
        ScCallStep::new().from(OWNER_ADDRESS_EXPR),
        |sc| {
            sc.mint_balances(&TokenIdentifier::from(TOKEN_ID)).set(BigUint::from(MINTED_AMOUNT));
            sc.burn_balances(&TokenIdentifier::from(TOKEN_ID)).set(BigUint::from(BURNED_AMOUNT));
        }
    );


    world.set_esdt_local_roles(
        managed_address!(&AddressValue::from(ESTD_SAFE_ADDRESS_EXPR).to_address()),
        TOKEN_ID,
        &[EsdtLocalRole::Mint, EsdtLocalRole::Burn]
    );

    const ESDT_TRANSFER_AMOUNT: u32 = 100u32;

    world.whitebox_call(
        &esdt_safe_whitebox,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR)
            .esdt_transfer(TOKEN_ID, 0, ESDT_TRANSFER_AMOUNT),
        |sc| {
            let burned_amount_before = sc.burn_balances(&TokenIdentifier::from(TOKEN_ID)).get();

            sc.create_transaction(convert_to_eth_address(ETH_ADDRESS));

            let burned_amount_after = sc.burn_balances(&TokenIdentifier::from(TOKEN_ID)).get();

            assert_eq!(burned_amount_after, burned_amount_before + ESDT_TRANSFER_AMOUNT);
        },
    );
}

#[test]
fn test_create_transaction_should_work_case_3() {
    let mut world = setup();

    let esdt_safe_whitebox = WhiteboxContract::new(
        ESTD_SAFE_ADDRESS_EXPR,
        esdt_safe::contract_obj,
    );

    const MAX_AMOUNT: u32 = 100_000_000u32;

    world.whitebox_call(
        &esdt_safe_whitebox,
        ScCallStep::new().from(OWNER_ADDRESS_EXPR),
        |sc| {
            sc.paused_status().set(false);
            sc.token_whitelist().insert(TokenIdentifier::from(TOKEN_ID));
            sc.max_bridged_amount(&TokenIdentifier::from(TOKEN_ID)).set(BigUint::from(MAX_AMOUNT));
            sc.mint_burn_token(&TokenIdentifier::from(TOKEN_ID)).set(false);
            sc.native_token(&TokenIdentifier::from(TOKEN_ID)).set(true);
        }
    );

    world.set_esdt_local_roles(
        managed_address!(&AddressValue::from(ESTD_SAFE_ADDRESS_EXPR).to_address()),
        TOKEN_ID,
        &[EsdtLocalRole::Mint, EsdtLocalRole::Burn]
    );

    const ESDT_TRANSFER_AMOUNT: u32 = 100u32;

    world.whitebox_call(
        &esdt_safe_whitebox,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR)
            .esdt_transfer(TOKEN_ID, 0, ESDT_TRANSFER_AMOUNT),
        |sc| {
            let total_balances_before = sc.total_balances(&TokenIdentifier::from(TOKEN_ID)).get();

            sc.create_transaction(convert_to_eth_address(ETH_ADDRESS));

            let total_balances_after = sc.total_balances(&TokenIdentifier::from(TOKEN_ID)).get();

            assert_eq!(total_balances_after, total_balances_before + ESDT_TRANSFER_AMOUNT);
        },
    );
}


#[test]
fn test_claim_refund_should_fail_case_1() {
    let mut world = setup();

    let esdt_safe_whitebox = WhiteboxContract::new(
        ESTD_SAFE_ADDRESS_EXPR,
        esdt_safe::contract_obj,
    );

    world.whitebox_call_check(
        &esdt_safe_whitebox,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR)
            .expect(TxExpect::user_error("str:Nothing to refund")),
        |sc| {
            sc.claim_refund(TokenIdentifier::from(TOKEN_ID));
        },
        |r| r.assert_user_error("Nothing to refund"),
    );
}

#[test]
fn test_claim_refund_should_work() {
    let mut world = setup();

    let esdt_safe_whitebox = WhiteboxContract::new(
        ESTD_SAFE_ADDRESS_EXPR,
        esdt_safe::contract_obj,
    );

    const REFUND_AMOUNT: u32 = 100u32;
    const TOTAL_REFUND_AMOUNT: u32 = 150u32;
    const TOTAL_BALANCE_MAPPER: u32 = 300u32;

    world.whitebox_call(
        &esdt_safe_whitebox,
        ScCallStep::new().from(OWNER_ADDRESS_EXPR),
        |sc| {
            sc.refund_amount(
                &managed_address!(&AddressValue::from(OWNER_ADDRESS_EXPR).to_address()),
                &TokenIdentifier::from(TOKEN_ID)
            ).set(BigUint::from(REFUND_AMOUNT));

            sc.total_refund_amount(&TokenIdentifier::from(TOKEN_ID)).set(BigUint::from(TOTAL_REFUND_AMOUNT));

            sc.total_balances(&TokenIdentifier::from(TOKEN_ID)).set(BigUint::from(TOTAL_BALANCE_MAPPER));
        }
    );

    world.whitebox_call(
        &esdt_safe_whitebox,
        ScCallStep::new().from(OWNER_ADDRESS_EXPR),
        |sc| {
            let esdt_token_payment = sc.claim_refund(TokenIdentifier::from(TOKEN_ID));
            let (token_id, nonce, amount) = esdt_token_payment.into_tuple();
            assert_eq!(token_id, TokenIdentifier::from(TOKEN_ID));
            assert_eq!(nonce, 0u64);
            assert_eq!(amount, REFUND_AMOUNT);
        }
    );
}


#[test]
fn test_init_supply_should_fail_case_1() {
    let mut world = setup();

    let esdt_safe_whitebox = WhiteboxContract::new(
        ESTD_SAFE_ADDRESS_EXPR,
        esdt_safe::contract_obj,
    );

    const SEND_AMOUNT: u32 = 10_000u32;
    const INIT_AMOUNT: u32 = SEND_AMOUNT;

    world.whitebox_call_check(
        &esdt_safe_whitebox,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR)
            .esdt_transfer(TOKEN_ID_2, 0, SEND_AMOUNT)
            .expect(TxExpect::user_error("str:Invalid token ID")),
        |sc| {
            sc.init_supply(TokenIdentifier::from(TOKEN_ID), BigUint::from(INIT_AMOUNT));
        },
        |r| r.assert_user_error("Invalid token ID"),
    );
}

#[test]
fn test_init_supply_should_fail_case_2() {
    let mut world = setup();

    let esdt_safe_whitebox = WhiteboxContract::new(
        ESTD_SAFE_ADDRESS_EXPR,
        esdt_safe::contract_obj,
    );

    const SEND_AMOUNT: u32 = 10_000u32;
    const INIT_AMOUNT: u32 = 100_000u32;

    world.whitebox_call_check(
        &esdt_safe_whitebox,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR)
            .esdt_transfer(TOKEN_ID, 0, SEND_AMOUNT)
            .expect(TxExpect::user_error("str:Invalid amount")),
        |sc| {
            sc.init_supply(TokenIdentifier::from(TOKEN_ID), BigUint::from(INIT_AMOUNT));
        },
        |r| r.assert_user_error("Invalid amount"),
    );
}

#[test]
fn test_init_supply_should_fail_case_3() {
    let mut world = setup();

    let esdt_safe_whitebox = WhiteboxContract::new(
        ESTD_SAFE_ADDRESS_EXPR,
        esdt_safe::contract_obj,
    );

    const SEND_AMOUNT: u32 = 10_000u32;
    const INIT_AMOUNT: u32 = SEND_AMOUNT;

    world.whitebox_call_check(
        &esdt_safe_whitebox,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR)
            .esdt_transfer(TOKEN_ID, 0, SEND_AMOUNT)
            .expect(TxExpect::user_error("str:Token not in whitelist")),
        |sc| {
            sc.init_supply(TokenIdentifier::from(TOKEN_ID), BigUint::from(INIT_AMOUNT));
        },
        |r| r.assert_user_error("Token not in whitelist"),
    );
}

#[test]
fn test_init_supply_should_fail_case_4() {
    let mut world = setup();

    let esdt_safe_whitebox = WhiteboxContract::new(
        ESTD_SAFE_ADDRESS_EXPR,
        esdt_safe::contract_obj,
    );

    world.whitebox_call(
        &esdt_safe_whitebox,
        ScCallStep::new().from(OWNER_ADDRESS_EXPR),
        |sc| {
            sc.token_whitelist().insert(TokenIdentifier::from(TOKEN_ID));
            sc.mint_burn_token(&TokenIdentifier::from(TOKEN_ID)).set(true);
            sc.native_token(&TokenIdentifier::from(TOKEN_ID)).set(false);
        }
    );

    const SEND_AMOUNT: u32 = 10_000u32;
    const INIT_AMOUNT: u32 = SEND_AMOUNT;

    world.whitebox_call_check(
        &esdt_safe_whitebox,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR)
            .esdt_transfer(TOKEN_ID, 0, SEND_AMOUNT)
            .expect(TxExpect::user_error("str:Cannot init for non native tokens")),
        |sc| {
            sc.init_supply(TokenIdentifier::from(TOKEN_ID), BigUint::from(INIT_AMOUNT));
        },
        |r| r.assert_user_error("Cannot init for non native tokens"),
    );
}

#[test]
fn test_init_supply_should_fail_case_5() {
    let mut world = setup();

    let esdt_safe_whitebox = WhiteboxContract::new(
        ESTD_SAFE_ADDRESS_EXPR,
        esdt_safe::contract_obj,
    );

    world.whitebox_call(
        &esdt_safe_whitebox,
        ScCallStep::new().from(OWNER_ADDRESS_EXPR),
        |sc| {
            sc.token_whitelist().insert(TokenIdentifier::from(TOKEN_ID));
            sc.mint_burn_token(&TokenIdentifier::from(TOKEN_ID)).set(true);
            sc.native_token(&TokenIdentifier::from(TOKEN_ID)).set(true);
        }
    );

    const SEND_AMOUNT: u32 = 10_000u32;
    const INIT_AMOUNT: u32 = SEND_AMOUNT;

    world.whitebox_call_check(
        &esdt_safe_whitebox,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR)
            .esdt_transfer(TOKEN_ID, 0, SEND_AMOUNT)
            .expect(TxExpect::user_error("str:Cannot do the burn action!")),
        |sc| {
            sc.init_supply(TokenIdentifier::from(TOKEN_ID), BigUint::from(INIT_AMOUNT));
        },
        |r| r.assert_user_error("Cannot do the burn action!"),
    );
}

#[test]
fn test_init_supply_should_work_case_1() {
    let mut world = setup();

    let esdt_safe_whitebox = WhiteboxContract::new(
        ESTD_SAFE_ADDRESS_EXPR,
        esdt_safe::contract_obj,
    );

    world.whitebox_call(
        &esdt_safe_whitebox,
        ScCallStep::new().from(OWNER_ADDRESS_EXPR),
        |sc| {
            sc.token_whitelist().insert(TokenIdentifier::from(TOKEN_ID));
            sc.mint_burn_token(&TokenIdentifier::from(TOKEN_ID)).set(false);
        }
    );

    const SEND_AMOUNT: u32 = 10_000u32;
    const INIT_AMOUNT: u32 = SEND_AMOUNT;

    world.whitebox_call(
        &esdt_safe_whitebox,
        ScCallStep::new()
        .from(OWNER_ADDRESS_EXPR)
        .esdt_transfer(TOKEN_ID, 0, SEND_AMOUNT),
        |sc| {
            let total_balances_before = sc.total_balances(&TokenIdentifier::from(TOKEN_ID)).get();

            sc.init_supply(TokenIdentifier::from(TOKEN_ID), BigUint::from(INIT_AMOUNT));

            let total_balances_after = sc.total_balances(&TokenIdentifier::from(TOKEN_ID)).get();

            assert_eq!(total_balances_after, total_balances_before + SEND_AMOUNT);
        }
    );
}

#[test]
fn test_init_supply_should_work_case_2() {
    let mut world = setup();

    let esdt_safe_whitebox = WhiteboxContract::new(
        ESTD_SAFE_ADDRESS_EXPR,
        esdt_safe::contract_obj,
    );

    world.whitebox_call(
        &esdt_safe_whitebox,
        ScCallStep::new().from(OWNER_ADDRESS_EXPR),
        |sc| {
            sc.token_whitelist().insert(TokenIdentifier::from(TOKEN_ID));
            sc.mint_burn_token(&TokenIdentifier::from(TOKEN_ID)).set(true);
            sc.native_token(&TokenIdentifier::from(TOKEN_ID)).set(true);
        }
    );

    const SEND_AMOUNT: u32 = 10_000u32;
    const INIT_AMOUNT: u32 = SEND_AMOUNT;

    world.set_esdt_local_roles(
        managed_address!(&AddressValue::from(ESTD_SAFE_ADDRESS_EXPR).to_address()),
        TOKEN_ID,
        &[EsdtLocalRole::Mint, EsdtLocalRole::Burn]
    );

    world.whitebox_call(
        &esdt_safe_whitebox,
        ScCallStep::new()
        .from(OWNER_ADDRESS_EXPR)
        .esdt_transfer(TOKEN_ID, 0, SEND_AMOUNT),
        |sc| {
            let burn_balance_before = sc.burn_balances(&TokenIdentifier::from(TOKEN_ID)).get();

            sc.init_supply(TokenIdentifier::from(TOKEN_ID), BigUint::from(INIT_AMOUNT));

            let burn_balance_after = sc.burn_balances(&TokenIdentifier::from(TOKEN_ID)).get();

            assert_eq!(burn_balance_after, burn_balance_before + SEND_AMOUNT);
        }
    );
}


#[test]
fn test_set_transaction_batch_status_should_fail_case_1() {
    let mut world = setup();

    let esdt_safe_whitebox = WhiteboxContract::new(
        ESTD_SAFE_ADDRESS_EXPR,
        esdt_safe::contract_obj,
    );

    let batch_id = 2u64;
    let array_of_statuses = vec![TransactionStatus::Executed];

    world.whitebox_call_check(
        &esdt_safe_whitebox,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR)
            .expect(TxExpect::user_error("str:Batches must be processed in order")),
        |sc| {

            let managed_vec_of_statuses = ManagedVec::from(array_of_statuses);
            sc.set_transaction_batch_status(batch_id, MultiValueEncoded::from(managed_vec_of_statuses));
        },
        |r| r.assert_user_error("Batches must be processed in order"),
    );
}

#[test]
fn test_set_transaction_batch_status_should_fail_case_2() {
    let mut world = setup();

    let esdt_safe_whitebox = WhiteboxContract::new(
        ESTD_SAFE_ADDRESS_EXPR,
        esdt_safe::contract_obj,
    );

    world.whitebox_call(
        &esdt_safe_whitebox,
        ScCallStep::new()
        .from(OWNER_ADDRESS_EXPR),
        |sc| {
            let txn_vec = vec![
                Transaction {
                    block_nonce: 1u64,
                    nonce: 1u64,
                    from: managed_buffer!(AddressValue::from(OWNER_ADDRESS_EXPR).to_address().as_bytes()),
                    to: managed_buffer!(AddressValue::from(SIGNER_0_ADDRESS_EXPR).to_address().as_bytes()),
                    token_identifier: TokenIdentifier::from(TOKEN_ID),
                    amount: BigUint::from(100u32),
                    is_refund_tx: false,
                },
                Transaction {
                    block_nonce: 1u64,
                    nonce: 2u64,
                    from: managed_buffer!(AddressValue::from(SIGNER_0_ADDRESS_EXPR).to_address().as_bytes()),
                    to: managed_buffer!(AddressValue::from(OWNER_ADDRESS_EXPR).to_address().as_bytes()),
                    token_identifier: TokenIdentifier::from(TOKEN_ID_2),
                    amount: BigUint::from(20u32),
                    is_refund_tx: false,
                }
            ];
            let txn_managed_vec = ManagedVec::from(txn_vec);
            sc.add_multiple_tx_to_batch(&txn_managed_vec);
        }
    );


    let batch_id = 1u64;
    let array_of_statuses = vec![TransactionStatus::Executed, TransactionStatus::Rejected, TransactionStatus::Executed];

    world.whitebox_call_check(
        &esdt_safe_whitebox,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR)
            .expect(TxExpect::user_error("str:Invalid number of statuses provided")),
        |sc| {

            let managed_vec_of_statuses = ManagedVec::from(array_of_statuses);
            sc.set_transaction_batch_status(batch_id, MultiValueEncoded::from(managed_vec_of_statuses));
        },
        |r| r.assert_user_error("Invalid number of statuses provided"),
    );
}

#[test]
fn test_set_transaction_batch_status_should_fail_case_3() {
    let mut world = setup();

    let esdt_safe_whitebox = WhiteboxContract::new(
        ESTD_SAFE_ADDRESS_EXPR,
        esdt_safe::contract_obj,
    );

    world.whitebox_call(
        &esdt_safe_whitebox,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR),
        |sc| {
            let txn_vec = vec![
                Transaction {
                    block_nonce: 1u64,
                    nonce: 1u64,
                    from: managed_buffer!(AddressValue::from(OWNER_ADDRESS_EXPR).to_address().as_bytes()),
                    to: managed_buffer!(AddressValue::from(SIGNER_0_ADDRESS_EXPR).to_address().as_bytes()),
                    token_identifier: TokenIdentifier::from(TOKEN_ID),
                    amount: BigUint::from(100u32),
                    is_refund_tx: false,
                },
            ];
            let txn_managed_vec = ManagedVec::from(txn_vec);
            sc.add_multiple_tx_to_batch(&txn_managed_vec);
        }
    );


    let batch_id = 1u64;
    let array_of_statuses = vec![TransactionStatus::Pending];

    world.whitebox_call_check(
        &esdt_safe_whitebox,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR)
            .expect(TxExpect::user_error("str:Transaction status may only be set to Executed or Rejected")),
        |sc| {

            let managed_vec_of_statuses = ManagedVec::from(array_of_statuses);
            sc.set_transaction_batch_status(batch_id, MultiValueEncoded::from(managed_vec_of_statuses));
        },
        |r| r.assert_user_error("Transaction status may only be set to Executed or Rejected"),
    );
}

#[test]
fn test_set_transaction_batch_status_should_work() {
    let mut world = setup();

    let esdt_safe_whitebox = WhiteboxContract::new(
        ESTD_SAFE_ADDRESS_EXPR,
        esdt_safe::contract_obj,
    );

    world.whitebox_call(
        &esdt_safe_whitebox,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR),
        |sc| {
            let txn_vec = vec![
                Transaction {
                    block_nonce: 1u64,
                    nonce: 1u64,
                    from: managed_buffer!(AddressValue::from(OWNER_ADDRESS_EXPR).to_address().as_bytes()),
                    to: managed_buffer!(AddressValue::from(SIGNER_0_ADDRESS_EXPR).to_address().as_bytes()),
                    token_identifier: TokenIdentifier::from(TOKEN_ID),
                    amount: BigUint::from(100u32),
                    is_refund_tx: false,
                },
                Transaction {
                    block_nonce: 1u64,
                    nonce: 2u64,
                    from: managed_buffer!(AddressValue::from(SIGNER_0_ADDRESS_EXPR).to_address().as_bytes()),
                    to: managed_buffer!(AddressValue::from(OWNER_ADDRESS_EXPR).to_address().as_bytes()),
                    token_identifier: TokenIdentifier::from(TOKEN_ID_2),
                    amount: BigUint::from(200u32),
                    is_refund_tx: false,
                }
            ];
            let txn_managed_vec = ManagedVec::from(txn_vec);
            sc.add_multiple_tx_to_batch(&txn_managed_vec);
        }
    );


    let batch_id = 1u64;
    let array_of_statuses = vec![TransactionStatus::Executed, TransactionStatus::Rejected];

    world.whitebox_call(
        &esdt_safe_whitebox,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR),
        |sc| {
            let managed_vec_of_statuses = ManagedVec::from(array_of_statuses);
            sc.set_transaction_batch_status(batch_id, MultiValueEncoded::from(managed_vec_of_statuses));

            let refound_amount = sc.refund_amount(
                &managed_address!(&AddressValue::from(SIGNER_0_ADDRESS_EXPR).to_address()),
                &TokenIdentifier::from(TOKEN_ID_2)
            ).get();

            let total_refund_amount = sc.total_refund_amount(&TokenIdentifier::from(TOKEN_ID_2)).get();

            assert_eq!(total_refund_amount, BigUint::from(200u32));
            assert_eq!(refound_amount, BigUint::from(200u32));

            assert_eq!(sc.first_batch_id().get(), batch_id + 1);
        },
    );
}


#[test]
fn test_add_refund_batch_should_fail_case_1() {
    let mut world = setup();

    let esdt_safe_whitebox = WhiteboxContract::new(
        ESTD_SAFE_ADDRESS_EXPR,
        esdt_safe::contract_obj,
    );

    world.whitebox_call_check(
        &esdt_safe_whitebox,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR)
            .expect(TxExpect::user_error("str:Invalid caller")),
        |sc| {
            sc.add_refund_batch(ManagedVec::new());
        },
        |r| r.assert_user_error("Invalid caller"),
    );
}

#[test]
fn test_add_refund_batch_should_fail_case_2() {
    let mut world = setup();

    let esdt_safe_whitebox = WhiteboxContract::new(
        ESTD_SAFE_ADDRESS_EXPR,
        esdt_safe::contract_obj,
    );

    world.whitebox_call_check(
        &esdt_safe_whitebox,
        ScCallStep::new()
            .from(MULTI_TRANSFER_ADDRESS_EXPR)
            .expect(TxExpect::user_error("str:Cannot refund with no payments")),
        |sc| {
            sc.multi_transfer_contract_address().set(managed_address!(&AddressValue::from(MULTI_TRANSFER_ADDRESS_EXPR).to_address()));
            sc.add_refund_batch(ManagedVec::new());
        },
        |r| r.assert_user_error("Cannot refund with no payments"),
    );
}

#[test]
fn test_add_refund_batch_should_fail_case_3() {
    let mut world = setup();

    let esdt_safe_whitebox = WhiteboxContract::new(
        ESTD_SAFE_ADDRESS_EXPR,
        esdt_safe::contract_obj,
    );

    world.whitebox_call_check(
        &esdt_safe_whitebox,
        ScCallStep::new()
            .from(MULTI_TRANSFER_ADDRESS_EXPR)
            .esdt_transfer(TOKEN_ID_2, 0, 100u32)
            .esdt_transfer(TOKEN_ID_2, 0, 20u32)
            .expect(TxExpect::user_error("str:Token identifiers do not match")),
        |sc| {
            sc.multi_transfer_contract_address().set(managed_address!(&AddressValue::from(MULTI_TRANSFER_ADDRESS_EXPR).to_address()));

            let txn_vec = vec![
                Transaction {
                    block_nonce: 1u64,
                    nonce: 1u64,
                    from: managed_buffer!(AddressValue::from(OWNER_ADDRESS_EXPR).to_address().as_bytes()),
                    to: managed_buffer!(AddressValue::from(SIGNER_0_ADDRESS_EXPR).to_address().as_bytes()),
                    token_identifier: TokenIdentifier::from(TOKEN_ID),
                    amount: BigUint::from(100u32),
                    is_refund_tx: false,
                },
                Transaction {
                    block_nonce: 1u64,
                    nonce: 2u64,
                    from: managed_buffer!(AddressValue::from(SIGNER_0_ADDRESS_EXPR).to_address().as_bytes()),
                    to: managed_buffer!(AddressValue::from(OWNER_ADDRESS_EXPR).to_address().as_bytes()),
                    token_identifier: TokenIdentifier::from(TOKEN_ID_2),
                    amount: BigUint::from(20u32),
                    is_refund_tx: false,
                }
            ];
            let txn_managed_vec = ManagedVec::from(txn_vec);

            sc.add_refund_batch(txn_managed_vec);
        },
        |r| r.assert_user_error("Token identifiers do not match"),
    );
}

#[test]
fn test_add_refund_batch_should_fail_case_4() {
    let mut world = setup();

    let esdt_safe_whitebox = WhiteboxContract::new(
        ESTD_SAFE_ADDRESS_EXPR,
        esdt_safe::contract_obj,
    );

    world.whitebox_call_check(
        &esdt_safe_whitebox,
        ScCallStep::new()
            .from(MULTI_TRANSFER_ADDRESS_EXPR)
            .esdt_transfer(TOKEN_ID, 0, 90u32)
            .esdt_transfer(TOKEN_ID_2, 0, 30u32)
            .expect(TxExpect::user_error("str:Amounts do not match")),
        |sc| {
            sc.multi_transfer_contract_address().set(managed_address!(&AddressValue::from(MULTI_TRANSFER_ADDRESS_EXPR).to_address()));

            let txn_vec = vec![
                Transaction {
                    block_nonce: 1u64,
                    nonce: 1u64,
                    from: managed_buffer!(AddressValue::from(OWNER_ADDRESS_EXPR).to_address().as_bytes()),
                    to: managed_buffer!(AddressValue::from(SIGNER_0_ADDRESS_EXPR).to_address().as_bytes()),
                    token_identifier: TokenIdentifier::from(TOKEN_ID),
                    amount: BigUint::from(100u32),
                    is_refund_tx: false,
                },
                Transaction {
                    block_nonce: 1u64,
                    nonce: 2u64,
                    from: managed_buffer!(AddressValue::from(SIGNER_0_ADDRESS_EXPR).to_address().as_bytes()),
                    to: managed_buffer!(AddressValue::from(OWNER_ADDRESS_EXPR).to_address().as_bytes()),
                    token_identifier: TokenIdentifier::from(TOKEN_ID_2),
                    amount: BigUint::from(20u32),
                    is_refund_tx: false,
                }
            ];
            let txn_managed_vec = ManagedVec::from(txn_vec);

            sc.add_refund_batch(txn_managed_vec);
        },
        |r| r.assert_user_error("Amounts do not match"),
    );
}

#[test]
fn test_add_refund_batch_should_work() {
    let mut world = setup();

    let esdt_safe_whitebox = WhiteboxContract::new(
        ESTD_SAFE_ADDRESS_EXPR,
        esdt_safe::contract_obj,
    );

    world.whitebox_call(
        &esdt_safe_whitebox,
        ScCallStep::new()
            .from(MULTI_TRANSFER_ADDRESS_EXPR)
            .esdt_transfer(TOKEN_ID, 0, 100u32)
            .esdt_transfer(TOKEN_ID_2, 0, 20u32),
        |sc| {
            sc.multi_transfer_contract_address().set(managed_address!(&AddressValue::from(MULTI_TRANSFER_ADDRESS_EXPR).to_address()));

            let txn_vec = vec![
                Transaction {
                    block_nonce: 1u64,
                    nonce: 1u64,
                    from: managed_buffer!(AddressValue::from(OWNER_ADDRESS_EXPR).to_address().as_bytes()),
                    to: managed_buffer!(AddressValue::from(SIGNER_0_ADDRESS_EXPR).to_address().as_bytes()),
                    token_identifier: TokenIdentifier::from(TOKEN_ID),
                    amount: BigUint::from(100u32),
                    is_refund_tx: false,
                },
                Transaction {
                    block_nonce: 1u64,
                    nonce: 2u64,
                    from: managed_buffer!(AddressValue::from(SIGNER_0_ADDRESS_EXPR).to_address().as_bytes()),
                    to: managed_buffer!(AddressValue::from(OWNER_ADDRESS_EXPR).to_address().as_bytes()),
                    token_identifier: TokenIdentifier::from(TOKEN_ID_2),
                    amount: BigUint::from(20u32),
                    is_refund_tx: false,
                }
            ];
            let txn_managed_vec = ManagedVec::from(txn_vec);
            sc.add_refund_batch(txn_managed_vec);
        },
    );
}


#[test]
fn test_compute_total_amounts_from_index() {
    let mut world = setup();

    let esdt_safe_whitebox = WhiteboxContract::new(
        ESTD_SAFE_ADDRESS_EXPR,
        esdt_safe::contract_obj,
    );

    world.whitebox_call(
        &esdt_safe_whitebox,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR),
        |sc| {
            let txn1 = Transaction {
                block_nonce: 1u64,
                nonce: 1u64,
                from: managed_buffer!(AddressValue::from(OWNER_ADDRESS_EXPR).to_address().as_bytes()),
                to: managed_buffer!(AddressValue::from(SIGNER_0_ADDRESS_EXPR).to_address().as_bytes()),
                token_identifier: TokenIdentifier::from(TOKEN_ID),
                amount: BigUint::from(100u32),
                is_refund_tx: false,
            };

            let txn2 = Transaction {
                block_nonce: 1u64,
                nonce: 2u64,
                from: managed_buffer!(AddressValue::from(SIGNER_0_ADDRESS_EXPR).to_address().as_bytes()),
                to: managed_buffer!(AddressValue::from(OWNER_ADDRESS_EXPR).to_address().as_bytes()),
                token_identifier: TokenIdentifier::from(TOKEN_ID_2),
                amount: BigUint::from(200u32),
                is_refund_tx: false,
            };

            let txn3 = Transaction {
                block_nonce: 1u64,
                nonce: 2u64,
                from: managed_buffer!(AddressValue::from(SIGNER_0_ADDRESS_EXPR).to_address().as_bytes()),
                to: managed_buffer!(AddressValue::from(OWNER_ADDRESS_EXPR).to_address().as_bytes()),
                token_identifier: TokenIdentifier::from(TOKEN_ID),
                amount: BigUint::from(400u32),
                is_refund_tx: false,
            };

            sc.add_to_batch(txn1);
            sc.add_to_batch(txn2);

            sc.create_new_batch(txn3);

            let total_amounts = sc.compute_total_amounts_from_index(1u64, 3u64);
            assert_eq!(total_amounts.get(0), EsdtTokenPayment::from((TokenIdentifier::from(TOKEN_ID), 0u64, BigUint::from(500u32))));
            assert_eq!(total_amounts.get(1), EsdtTokenPayment::from((TokenIdentifier::from(TOKEN_ID_2), 0u64, BigUint::from(200u32))));
        }
    );
}


#[test]
fn test_get_total_refund_amounts_for_address() {
    let mut world = setup();

    let esdt_safe_whitebox = WhiteboxContract::new(
        ESTD_SAFE_ADDRESS_EXPR,
        esdt_safe::contract_obj,
    );

    let owner_address = AddressValue::from(OWNER_ADDRESS_EXPR);
    let signer_0_address = AddressValue::from(SIGNER_0_ADDRESS_EXPR);

    // 1st step: Prepare the refund amounts for the owner and signer_0
    world.whitebox_call(
        &esdt_safe_whitebox,
        ScCallStep::new().from(OWNER_ADDRESS_EXPR),
        |sc| {
            sc.token_whitelist().insert(TokenIdentifier::from(TOKEN_ID));
            sc.token_whitelist().insert(TokenIdentifier::from(TOKEN_ID_2));

            sc.refund_amount(
                &managed_address!(&owner_address.to_address()),
                &TokenIdentifier::from(TOKEN_ID)
            ).set(BigUint::from(100u32));

            sc.refund_amount(
                &managed_address!(&signer_0_address.to_address()),
                &TokenIdentifier::from(TOKEN_ID_2)
            ).set(BigUint::from(200u32));
        }
    );

    // 2nd step: Query the total refund amounts for the owner and signer_0
    world.whitebox_query(&esdt_safe_whitebox, |sc| {
        let total_refund_amounts = sc.get_refund_amounts(managed_address!(&owner_address.to_address()));

        let expected = MultiValue2::from((TokenIdentifier::from(TOKEN_ID), BigUint::from(100u32)));
        let result_vec: Vec<_> = total_refund_amounts.into_iter().collect();

        assert!(result_vec.contains(&expected), "Expected refund amount for TOKEN_ID not found.");
    });

    world.whitebox_query(&esdt_safe_whitebox, |sc| {
        let total_refund_amounts = sc.get_refund_amounts(managed_address!(&signer_0_address.to_address()));

        let expected = MultiValue2::from((TokenIdentifier::from(TOKEN_ID_2), BigUint::from(200u32)));
        let result_vec: Vec<_> = total_refund_amounts.into_iter().collect();

        assert!(result_vec.contains(&expected), "Expected refund amount for TOKEN_ID_MINT_BURN not found.");
    });
}


#[test]
fn test_get_total_refund_amounts() {
    let mut world = setup();

    let esdt_safe_whitebox = WhiteboxContract::new(
        ESTD_SAFE_ADDRESS_EXPR,
        esdt_safe::contract_obj,
    );

    world.whitebox_call(
        &esdt_safe_whitebox,
        ScCallStep::new().from(OWNER_ADDRESS_EXPR),
        |sc| {
            sc.token_whitelist().insert(TokenIdentifier::from(TOKEN_ID));
            sc.total_refund_amount(&TokenIdentifier::from(TOKEN_ID)).set(BigUint::from(100u32));
        }
    );

    world.whitebox_call(
        &esdt_safe_whitebox,
        ScCallStep::new().from(SIGNER_0_ADDRESS_EXPR),
        |sc| {
            sc.token_whitelist().insert(TokenIdentifier::from(TOKEN_ID_2));
            sc.total_refund_amount(&TokenIdentifier::from(TOKEN_ID_2)).set(BigUint::from(200u32));
        }
    );

    // Query the total refund amounts
    world.whitebox_query(&esdt_safe_whitebox, |sc| {
        let total_refund_amounts = sc.getTotalRefundAmounts();

        // Expected tuples
        let expected_1 = MultiValue2::from((TokenIdentifier::from(TOKEN_ID), BigUint::from(100u32)));
        let expected_2 = MultiValue2::from((TokenIdentifier::from(TOKEN_ID_2), BigUint::from(200u32)));

        // Convert to vector for easy comparison
        let result_vec: Vec<_> = total_refund_amounts.into_iter().collect();

        // Check that both expected tuples are in the result set
        assert!(result_vec.contains(&expected_1), "Expected refund amount for TOKEN_ID not found.");
        assert!(result_vec.contains(&expected_2), "Expected refund amount for TOKEN_ID_MINT_BURN not found.");
        assert_eq!(result_vec.len(), 2);
    });
}


#[test]
fn test_rebalance_for_refund_case_1() {
    let mut world = setup();

    let esdt_safe_whitebox = WhiteboxContract::new(
        ESTD_SAFE_ADDRESS_EXPR,
        esdt_safe::contract_obj,
    );

    // Scenario 1: mintBurnToken = false
    world.whitebox_call(
        &esdt_safe_whitebox,
        ScCallStep::new().from(OWNER_ADDRESS_EXPR),
        |sc| {
            sc.mint_burn_token(&TokenIdentifier::from(TOKEN_ID)).set(false);
            sc.total_balances(&TokenIdentifier::from(TOKEN_ID)).set(BigUint::from(100u32));
        },
    );

    let refund_amount = 10u64;
    world.whitebox_call(
        &esdt_safe_whitebox,
        ScCallStep::new().from(OWNER_ADDRESS_EXPR),
        |sc| {
            let balance_before = sc.total_balances(&managed_token_id!(TOKEN_ID)).get().to_u64().unwrap_or(0);

            sc.rebalance_for_refund(&TokenIdentifier::from(TOKEN_ID), &BigUint::from(refund_amount));

            let balance_after = sc.total_balances(&TokenIdentifier::from(TOKEN_ID)).get().to_u64().unwrap_or(0);

            assert_eq!(balance_after, balance_before - refund_amount);
        },
    );
}

#[test]
fn test_rebalance_for_refund_case_2() {
    let mut world = setup();

    let esdt_safe_whitebox = WhiteboxContract::new(
        ESTD_SAFE_ADDRESS_EXPR,
        esdt_safe::contract_obj,
    );

    // Scenario 2: mintBurnToken = true
    world.whitebox_call(
        &esdt_safe_whitebox,
        ScCallStep::new().from(OWNER_ADDRESS_EXPR),
        |sc| {
            sc.mint_burn_token(&TokenIdentifier::from(TOKEN_ID_2)).set(true);
            assert_eq!(sc.mint_burn_token(&TokenIdentifier::from(TOKEN_ID_2)).get(), true);
        },
    );

    world.set_esdt_local_roles(
        managed_address!(&AddressValue::from(ESTD_SAFE_ADDRESS_EXPR).to_address()),
        TOKEN_ID_2,
        &[EsdtLocalRole::Mint, EsdtLocalRole::Burn]
    );

    let refund_amount = 10u64;
    world.whitebox_call(
        &esdt_safe_whitebox,
        ScCallStep::new().from(OWNER_ADDRESS_EXPR),
        |sc| {
            let balance_before = sc.mint_balances(&TokenIdentifier::from(TOKEN_ID_2)).get().to_u64().unwrap_or(0);

            sc.rebalance_for_refund(&TokenIdentifier::from(TOKEN_ID_2), &BigUint::from(refund_amount));

            let balance_after = sc.mint_balances(&TokenIdentifier::from(TOKEN_ID_2)).get().to_u64().unwrap_or(0);

            assert_eq!(balance_after, balance_before + refund_amount);
        },
    );
}


#[test]
fn test_mark_refund() {
    let mut world = setup();

    let esdt_safe_whitebox = WhiteboxContract::new(
        ESTD_SAFE_ADDRESS_EXPR,
        esdt_safe::contract_obj,
    );

    world.whitebox_call(
        &esdt_safe_whitebox,
        ScCallStep::new().from(OWNER_ADDRESS_EXPR),
        |sc| {
            sc.mark_refund(
                &managed_address!(&AddressValue::from(SIGNER_0_ADDRESS_EXPR).to_address()),
                &TokenIdentifier::from(TOKEN_ID),
                &BigUint::from(10u32)
            );

            let refund_amount = sc.refund_amount(
                &managed_address!(&AddressValue::from(SIGNER_0_ADDRESS_EXPR).to_address()),
                &managed_token_id!(TOKEN_ID)
            ).get();
            assert_eq!(refund_amount, 10u64);

            let total_refund_amount = sc.total_refund_amount(&managed_token_id!(TOKEN_ID)).get();
            assert_eq!(total_refund_amount, 10u64);
        }

    );
}


fn setup() -> ScenarioWorld {
    let mut world = world();

    let esdt_safe_whitebox = WhiteboxContract::new(
        ESTD_SAFE_ADDRESS_EXPR,
        esdt_safe::contract_obj,
    );
    let estd_safe_code = world.code_expression(ESTD_SAFE_PATH_EXPR);

    let multi_transfer_whitebox = WhiteboxContract::new(
        MULTI_TRANSFER_ADDRESS_EXPR,
        multi_transfer_esdt::contract_obj,
    );
    let multi_transfer_code = world.code_expression(MULTI_TRANSFER_PATH_EXPR);

    let set_state_step = SetStateStep::new()
        .put_account(
            OWNER_ADDRESS_EXPR,
            Account::new().nonce(1)
                .balance(100_000_000u64)
                .esdt_balance(TOKEN_ID.to_vec(), 100_000_000u64)
                .esdt_balance(TOKEN_ID_2.to_vec(), 100_000_000u64)
        )
        .put_account(
            SIGNER_0_ADDRESS_EXPR,
            Account::new().nonce(1)
                .balance(100_000_000u64)
                .esdt_balance(TOKEN_ID.to_vec(), 100_000_000u64)
                .esdt_balance(TOKEN_ID_2.to_vec(), 100_000_000u64)
        )
        .new_address(OWNER_ADDRESS_EXPR, 1, ESTD_SAFE_ADDRESS_EXPR)
        .new_address(OWNER_ADDRESS_EXPR, 2, MULTI_TRANSFER_ADDRESS_EXPR)
        .block_timestamp(100);

    let fee_estimator_expr = format!("sc:fee_estimator");
    let multi_transfer_expr = format!("sc:multi_transfer");

    world.set_state_step(set_state_step)
        .whitebox_deploy(
            &esdt_safe_whitebox,
            ScDeployStep::new()
                .from(OWNER_ADDRESS_EXPR)
                .code(estd_safe_code),
            |sc| {
                sc.init(
                    managed_address!(&AddressValue::from(fee_estimator_expr.as_str()).to_address()),
                    managed_address!(&AddressValue::from(multi_transfer_expr.as_str()).to_address()),
                    BigUint::from(1000u32),
                );
            },
        )
        .whitebox_deploy(
            &multi_transfer_whitebox,
            ScDeployStep::new()
                .from(OWNER_ADDRESS_EXPR)
                .code(multi_transfer_code),
            |sc|
                    sc.init(),
            
        );

    world.set_esdt_balance(esdt_safe_whitebox.address_expr.value.clone(), TOKEN_ID, 100_000_000u64);
    world.set_esdt_balance(esdt_safe_whitebox.address_expr.value.clone(), TOKEN_ID_2, 100_000_000u64);
    world.set_esdt_balance(multi_transfer_whitebox.address_expr.value.clone(), TOKEN_ID, 100_000_000u64);
    world.set_esdt_balance(multi_transfer_whitebox.address_expr.value.clone(), TOKEN_ID_2, 100_000_000u64);

    world
}


fn convert_to_eth_address(address: &str) -> EthAddress<DebugApi> {
    let address_str = address.trim_start_matches("0x"); // Remove the "0x" prefix if it's present

    // Convert the hexadecimal string to a byte array
    let mut address_bytes = [0u8; 20]; // Initialize a 20-byte array
    for (i, byte) in address_bytes.iter_mut().enumerate() {
        let offset = i * 2;
        *byte = u8::from_str_radix(&address_str[offset..offset + 2], 16).expect("Parsing error");
    }

    EthAddress{raw_addr: ManagedByteArray::new_from_bytes(&address_bytes)}
}

#[test]
fn test_all() {
    test_create_transaction_should_fail_case_1();
    test_create_transaction_should_fail_case_2();
    // test_create_transaction_should_fail_case_3();
    // test_create_transaction_should_fail_case_4();
    // test_create_transaction_should_fail_case_5();
    // test_create_transaction_should_fail_case_6();
    // test_create_transaction_should_work_case_1();
    // test_create_transaction_should_work_case_2();
    // test_create_transaction_should_work_case_3();

    test_claim_refund_should_fail_case_1();
    test_claim_refund_should_work();

    test_init_supply_should_fail_case_1();
    test_init_supply_should_fail_case_2();
    test_init_supply_should_fail_case_3();
    test_init_supply_should_fail_case_4();
    test_init_supply_should_fail_case_5();
    test_init_supply_should_work_case_1();
    test_init_supply_should_work_case_2();

    test_set_transaction_batch_status_should_fail_case_1();
    test_set_transaction_batch_status_should_fail_case_2();
    test_set_transaction_batch_status_should_fail_case_3();
    test_set_transaction_batch_status_should_work();

    test_add_refund_batch_should_fail_case_1();
    test_add_refund_batch_should_fail_case_2();
    test_add_refund_batch_should_fail_case_3();
    test_add_refund_batch_should_fail_case_4();
    // test_add_refund_batch_should_work();

    test_compute_total_amounts_from_index();

    test_get_total_refund_amounts_for_address();

    test_get_total_refund_amounts();

    test_rebalance_for_refund_case_1();
    test_rebalance_for_refund_case_2();

    test_mark_refund();
}