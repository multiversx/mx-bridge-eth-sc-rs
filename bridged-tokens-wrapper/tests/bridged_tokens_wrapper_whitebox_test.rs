#![allow(deprecated)]

use bridged_tokens_wrapper::BridgedTokensWrapper;
use eth_address::EthAddress;
use multiversx_sc_modules::pause::PauseModule;
use multiversx_sc_scenario::imports::*;

const UNIVERSAL_TOKEN_IDENTIFIER: &[u8] = b"UNIV-abc123";
const CHAIN_TOKEN_IDENTIFIER: &[u8] = b"CHAIN-xyz789";
const CHAIN_TOKEN_IDENTIFIER2: &[u8] = b"CHAIN-xyz789123";
const ETH_ADDRESS: &str = "0x2E8e0BBe20Ecd819c721D164fb91F7c33BDFC756";
const NUM_DECIMALS: u32 = 18;

const OWNER_ADDRESS_EXPR: &str = "address:owner";
const BRIDGE_TOKENS_WRAPPER_ADDRESS_EXPR: &str = "sc:bridged-tokens-wrapper";
const BRIDGE_TOKENS_WRAPPER_PATH_EXPR: &str = "mxsc:output/bridged-tokens-wrapper.mxsc.json";

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(
        BRIDGE_TOKENS_WRAPPER_PATH_EXPR,
        bridged_tokens_wrapper::ContractBuilder,
    );

    blockchain
}

#[test]
fn test_get_converted_amount_should_work() {
    let mut world = setup();

    let bridged_tokens_wrapper = WhiteboxContract::new(
        BRIDGE_TOKENS_WRAPPER_ADDRESS_EXPR,
        bridged_tokens_wrapper::contract_obj,
    );

    let number = 100u64;
    world.whitebox_query(&bridged_tokens_wrapper, |sc| {
        let result = sc.get_converted_amount(
            &managed_token_id!(UNIVERSAL_TOKEN_IDENTIFIER),
            &managed_token_id!(CHAIN_TOKEN_IDENTIFIER),
            managed_biguint!(number),
        );
        assert_eq!(result, managed_biguint!(number));
    });
}

#[test]
fn test_require_tokens_have_set_decimals_num_should_fail_case_1() {
    let mut world = setup();
    let bridged_tokens_wrapper = WhiteboxContract::new(
        BRIDGE_TOKENS_WRAPPER_ADDRESS_EXPR,
        bridged_tokens_wrapper::contract_obj,
    );

    world.whitebox_call_check(
        &bridged_tokens_wrapper,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR)
            .expect(TxExpect::user_error(
                "str:Universal token requires updating",
            )),
        |sc| {
            sc.require_tokens_have_set_decimals_num(
                &managed_token_id!(UNIVERSAL_TOKEN_IDENTIFIER),
                &managed_token_id!(CHAIN_TOKEN_IDENTIFIER),
            );
        },
        |r| r.assert_user_error("Universal token requires updating"),
    );
}

#[test]
fn test_require_tokens_have_set_decimals_num_should_fail_case_2() {
    let mut world = setup();
    let bridged_tokens_wrapper = WhiteboxContract::new(
        BRIDGE_TOKENS_WRAPPER_ADDRESS_EXPR,
        bridged_tokens_wrapper::contract_obj,
    );

    world.whitebox_call(
        &bridged_tokens_wrapper,
        ScCallStep::new().from(OWNER_ADDRESS_EXPR),
        |sc| {
            sc.token_decimals_num(&managed_token_id!(UNIVERSAL_TOKEN_IDENTIFIER))
                .set(18u32);
        },
    );

    world.whitebox_call_check(
        &bridged_tokens_wrapper,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR)
            .expect(TxExpect::user_error(
                "str:Chain-specific token requires updating",
            )),
        |sc| {
            sc.token_decimals_num(&managed_token_id!(UNIVERSAL_TOKEN_IDENTIFIER))
                .set(18u32);
            sc.require_tokens_have_set_decimals_num(
                &managed_token_id!(UNIVERSAL_TOKEN_IDENTIFIER),
                &managed_token_id!(CHAIN_TOKEN_IDENTIFIER),
            )
        },
        |r| r.assert_user_error("Chain-specific token requires updating"),
    );
}

#[test]
fn test_require_tokens_have_set_decimals_num_should_work() {
    let mut world = setup();
    let bridged_tokens_wrapper = WhiteboxContract::new(
        BRIDGE_TOKENS_WRAPPER_ADDRESS_EXPR,
        bridged_tokens_wrapper::contract_obj,
    );

    world.whitebox_call(
        &bridged_tokens_wrapper,
        ScCallStep::new().from(OWNER_ADDRESS_EXPR).no_expect(),
        |sc| {
            sc.token_decimals_num(&managed_token_id!(UNIVERSAL_TOKEN_IDENTIFIER))
                .set(18u32);
            sc.token_decimals_num(&managed_token_id!(CHAIN_TOKEN_IDENTIFIER))
                .set(18u32);
            sc.require_tokens_have_set_decimals_num(
                &managed_token_id!(UNIVERSAL_TOKEN_IDENTIFIER),
                &managed_token_id!(CHAIN_TOKEN_IDENTIFIER),
            );
        },
    );
}

#[test]
fn test_require_mint_and_burn_roles_should_fail() {
    let mut world = setup();
    let bridged_tokens_wrapper = WhiteboxContract::new(
        BRIDGE_TOKENS_WRAPPER_ADDRESS_EXPR,
        bridged_tokens_wrapper::contract_obj,
    );

    world.whitebox_call_check(
        &bridged_tokens_wrapper,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR)
            .expect(TxExpect::user_error("str:Must set local role first")),
        |sc| {
            sc.require_mint_and_burn_roles(&managed_token_id!(UNIVERSAL_TOKEN_IDENTIFIER));
        },
        |r| r.assert_user_error("Must set local role first"),
    );
}

#[test]
fn test_require_mint_and_burn_roles_should_work() {
    let mut world = setup();
    let bridged_tokens_wrapper = WhiteboxContract::new(
        BRIDGE_TOKENS_WRAPPER_ADDRESS_EXPR,
        bridged_tokens_wrapper::contract_obj,
    );

    let contract_address = AddressValue::from(BRIDGE_TOKENS_WRAPPER_ADDRESS_EXPR);

    world.set_esdt_local_roles(
        managed_address!(&contract_address.to_address()),
        UNIVERSAL_TOKEN_IDENTIFIER,
        &[EsdtLocalRole::Mint, EsdtLocalRole::Burn],
    );

    world.whitebox_call(
        &bridged_tokens_wrapper,
        ScCallStep::new().from(OWNER_ADDRESS_EXPR).no_expect(),
        |sc| {
            sc.require_mint_and_burn_roles(&managed_token_id!(UNIVERSAL_TOKEN_IDENTIFIER));
        },
    );
}

#[test]
fn test_deposit_liquidity_should_work() {
    let mut world = setup();
    let bridged_tokens_wrapper = WhiteboxContract::new(
        BRIDGE_TOKENS_WRAPPER_ADDRESS_EXPR,
        bridged_tokens_wrapper::contract_obj,
    );

    world.set_state_step(
        SetStateStep::new().put_account(
            OWNER_ADDRESS_EXPR,
            Account::new()
                .nonce(1)
                .balance(100_000_000u64)
                .esdt_balance(UNIVERSAL_TOKEN_IDENTIFIER.to_vec(), 100_000_000u64),
        ),
    );

    world.whitebox_call(
        &bridged_tokens_wrapper,
        ScCallStep::new().from(OWNER_ADDRESS_EXPR).esdt_transfer(
            UNIVERSAL_TOKEN_IDENTIFIER,
            0,
            100u32,
        ),
        |sc| {
            sc.set_paused(false);
            sc.chain_specific_to_universal_mapping(&managed_token_id!(UNIVERSAL_TOKEN_IDENTIFIER))
                .set(managed_token_id!(UNIVERSAL_TOKEN_IDENTIFIER));

            sc.deposit_liquidity();
            let result = sc
                .token_liquidity(&managed_token_id!(UNIVERSAL_TOKEN_IDENTIFIER))
                .get();
            assert_ne!(result, 0);
        },
    );
}

#[test]
fn test_add_wrapped_token_should_fail_case_1() {
    let mut world = setup();
    let bridged_tokens_wrapper = WhiteboxContract::new(
        BRIDGE_TOKENS_WRAPPER_ADDRESS_EXPR,
        bridged_tokens_wrapper::contract_obj,
    );

    world.whitebox_call_check(
        &bridged_tokens_wrapper,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR)
            .expect(TxExpect::user_error("str:Must set local role first")),
        |sc| {
            sc.add_wrapped_token(managed_token_id!(UNIVERSAL_TOKEN_IDENTIFIER), NUM_DECIMALS);
        },
        |r| r.assert_user_error("Must set local role first"),
    );
}

#[test]
fn test_add_wrapped_token_should_work() {
    let mut world = setup();
    let bridged_tokens_wrapper = WhiteboxContract::new(
        BRIDGE_TOKENS_WRAPPER_ADDRESS_EXPR,
        bridged_tokens_wrapper::contract_obj,
    );

    let contract_address = AddressValue::from(BRIDGE_TOKENS_WRAPPER_ADDRESS_EXPR);
    world.set_esdt_local_roles(
        managed_address!(&contract_address.to_address()),
        UNIVERSAL_TOKEN_IDENTIFIER,
        &[EsdtLocalRole::Mint, EsdtLocalRole::Burn],
    );

    world.whitebox_call(
        &bridged_tokens_wrapper,
        ScCallStep::new().from(OWNER_ADDRESS_EXPR),
        |sc| {
            sc.add_wrapped_token(managed_token_id!(UNIVERSAL_TOKEN_IDENTIFIER), NUM_DECIMALS);
        },
    );
}

#[test]
fn test_update_wrapped_token_should_fail_case_1() {
    let mut world = setup();
    let bridged_tokens_wrapper = WhiteboxContract::new(
        BRIDGE_TOKENS_WRAPPER_ADDRESS_EXPR,
        bridged_tokens_wrapper::contract_obj,
    );

    world.whitebox_call_check(
        &bridged_tokens_wrapper,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR)
            .expect(TxExpect::user_error(
                "str:Universal token was not added yet",
            )),
        |sc| {
            sc.update_wrapped_token(managed_token_id!(UNIVERSAL_TOKEN_IDENTIFIER), NUM_DECIMALS);
        },
        |r| r.assert_user_error("Universal token was not added yet"),
    );
}

#[test]
fn test_update_wrapped_token_shoud_work() {
    let mut world = setup();
    let bridged_tokens_wrapper = WhiteboxContract::new(
        BRIDGE_TOKENS_WRAPPER_ADDRESS_EXPR,
        bridged_tokens_wrapper::contract_obj,
    );

    let contract_address = AddressValue::from(BRIDGE_TOKENS_WRAPPER_ADDRESS_EXPR);
    world.set_esdt_local_roles(
        managed_address!(&contract_address.to_address()),
        UNIVERSAL_TOKEN_IDENTIFIER,
        &[EsdtLocalRole::Mint, EsdtLocalRole::Burn],
    );

    world.whitebox_call(
        &bridged_tokens_wrapper,
        ScCallStep::new().from(OWNER_ADDRESS_EXPR),
        |sc| {
            sc.add_wrapped_token(managed_token_id!(UNIVERSAL_TOKEN_IDENTIFIER), NUM_DECIMALS);
        },
    );

    world.whitebox_query(&bridged_tokens_wrapper, |sc| {
        let result = sc
            .universal_bridged_token_ids()
            .contains(&managed_token_id!(UNIVERSAL_TOKEN_IDENTIFIER));
        assert!(result);
    });

    world.whitebox_call(
        &bridged_tokens_wrapper,
        ScCallStep::new().from(OWNER_ADDRESS_EXPR).no_expect(),
        |sc| {
            sc.update_wrapped_token(managed_token_id!(UNIVERSAL_TOKEN_IDENTIFIER), 32);
        },
    );

    world.whitebox_query(&bridged_tokens_wrapper, |sc| {
        let result = sc
            .token_decimals_num(&managed_token_id!(UNIVERSAL_TOKEN_IDENTIFIER))
            .get();
        assert_eq!(result, 32);
    });
}

#[test]
fn test_remove_wrapped_token_should_work() {
    let mut world = setup();
    let bridged_tokens_wrapper = WhiteboxContract::new(
        BRIDGE_TOKENS_WRAPPER_ADDRESS_EXPR,
        bridged_tokens_wrapper::contract_obj,
    );

    let contract_address = AddressValue::from(BRIDGE_TOKENS_WRAPPER_ADDRESS_EXPR);
    world.set_esdt_local_roles(
        managed_address!(&contract_address.to_address()),
        UNIVERSAL_TOKEN_IDENTIFIER,
        &[EsdtLocalRole::Mint, EsdtLocalRole::Burn],
    );

    world.whitebox_call(
        &bridged_tokens_wrapper,
        ScCallStep::new().from(OWNER_ADDRESS_EXPR),
        |sc| {
            sc.add_wrapped_token(managed_token_id!(UNIVERSAL_TOKEN_IDENTIFIER), NUM_DECIMALS);
            sc.chain_specific_token_ids(&managed_token_id!(UNIVERSAL_TOKEN_IDENTIFIER))
                .insert(managed_token_id!(CHAIN_TOKEN_IDENTIFIER));
            sc.chain_specific_token_ids(&managed_token_id!(UNIVERSAL_TOKEN_IDENTIFIER))
                .insert(managed_token_id!(CHAIN_TOKEN_IDENTIFIER2));
            sc.remove_wrapped_token(managed_token_id!(UNIVERSAL_TOKEN_IDENTIFIER));
        },
    );

    world.whitebox_query(&bridged_tokens_wrapper, |sc| {
        let result = sc
            .universal_bridged_token_ids()
            .contains(&managed_token_id!(UNIVERSAL_TOKEN_IDENTIFIER));
        assert!(!result);
    });
}

#[test]
#[ignore] //Ignore for now; Cannot import esdt-safe code here
fn test_unwrap_token_create_transaction_should_fail_case_1() {
    let mut world = setup();
    let bridged_tokens_wrapper = WhiteboxContract::new(
        BRIDGE_TOKENS_WRAPPER_ADDRESS_EXPR,
        bridged_tokens_wrapper::contract_obj,
    );

    world.set_state_step(
        SetStateStep::new().put_account(
            OWNER_ADDRESS_EXPR,
            Account::new()
                .nonce(1)
                .balance(100_000_000u64)
                .esdt_balance(UNIVERSAL_TOKEN_IDENTIFIER.to_vec(), 100_000_000u64),
        ),
    );

    world.whitebox_call_check(
        &bridged_tokens_wrapper,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR)
            .esdt_transfer(UNIVERSAL_TOKEN_IDENTIFIER, 0, 0u32)
            .expect(TxExpect::user_error("str:Contract is paused")),
        |sc| {
            sc.set_paused(true);

            let address = convert_to_eth_address(ETH_ADDRESS);
            sc.unwrap_token_create_transaction(
                managed_token_id!(UNIVERSAL_TOKEN_IDENTIFIER),
                ManagedAddress::new_from_bytes(b"0102030405060708090a0b0c0d0e0f10"),
                address,
                OptionalValue::None,
            );
        },
        |r| r.assert_user_error("Contract is paused"),
    );
}

#[test]
#[ignore] //Ignore for now; Cannot import esdt-safe code here
fn test_unwrap_token_create_transaction_should_fail_case_2() {
    let mut world = setup();
    let bridged_tokens_wrapper = WhiteboxContract::new(
        BRIDGE_TOKENS_WRAPPER_ADDRESS_EXPR,
        bridged_tokens_wrapper::contract_obj,
    );

    world.set_state_step(
        SetStateStep::new().put_account(
            OWNER_ADDRESS_EXPR,
            Account::new()
                .nonce(1)
                .balance(100_000_000u64)
                .esdt_balance(UNIVERSAL_TOKEN_IDENTIFIER.to_vec(), 100_000_000u64),
        ),
    );

    world.whitebox_call_check(
        &bridged_tokens_wrapper,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR)
            .esdt_transfer(UNIVERSAL_TOKEN_IDENTIFIER, 0, 0u32)
            .expect(TxExpect::user_error("str:Must pay more than 0 tokens!")),
        |sc| {
            sc.set_paused(false);
            let address = convert_to_eth_address(ETH_ADDRESS);
            sc.unwrap_token_create_transaction(
                managed_token_id!(UNIVERSAL_TOKEN_IDENTIFIER),
                ManagedAddress::new_from_bytes(b"0102030405060708090a0b0c0d0e0f10"),
                address,
                OptionalValue::None,
            );
        },
        |r| r.assert_user_error("Must pay more than 0 tokens!"),
    );
}

#[test]
#[ignore] //Ignore for now; Cannot import esdt-safe code here
fn test_unwrap_token_create_transaction_should_fail_case_3() {
    let mut world = setup();
    let bridged_tokens_wrapper = WhiteboxContract::new(
        BRIDGE_TOKENS_WRAPPER_ADDRESS_EXPR,
        bridged_tokens_wrapper::contract_obj,
    );

    world.set_state_step(
        SetStateStep::new().put_account(
            OWNER_ADDRESS_EXPR,
            Account::new()
                .nonce(1)
                .balance(100_000_000u64)
                .esdt_balance(UNIVERSAL_TOKEN_IDENTIFIER.to_vec(), 100_000_000u64),
        ),
    );

    world.whitebox_call_check(
        &bridged_tokens_wrapper,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR)
            .esdt_transfer(UNIVERSAL_TOKEN_IDENTIFIER, 0, 100000u32)
            .expect(TxExpect::user_error("str:Esdt token unavailable")),
        |sc| {
            sc.set_paused(false);
            let address = convert_to_eth_address(ETH_ADDRESS);
            sc.unwrap_token_create_transaction(
                managed_token_id!(UNIVERSAL_TOKEN_IDENTIFIER),
                ManagedAddress::zero(),
                address,
                OptionalValue::None,
            );
        },
        |r| r.assert_user_error("Esdt token unavailable"),
    );
}

#[test]
fn test_unwrap_token_create_transaction_should_fail_case_4() {
    let mut world = setup();
    let bridged_tokens_wrapper = WhiteboxContract::new(
        BRIDGE_TOKENS_WRAPPER_ADDRESS_EXPR,
        bridged_tokens_wrapper::contract_obj,
    );
    let contract_address = AddressValue::from(BRIDGE_TOKENS_WRAPPER_ADDRESS_EXPR);

    world.set_state_step(
        SetStateStep::new().put_account(
            OWNER_ADDRESS_EXPR,
            Account::new()
                .nonce(1)
                .balance(100_000_000u64)
                .esdt_balance(UNIVERSAL_TOKEN_IDENTIFIER.to_vec(), 100_000_000u64),
        ),
    );

    world.set_esdt_local_roles(
        managed_address!(&contract_address.to_address()),
        UNIVERSAL_TOKEN_IDENTIFIER,
        &[EsdtLocalRole::Mint, EsdtLocalRole::Burn],
    );

    world.whitebox_call(
        &bridged_tokens_wrapper,
        ScCallStep::new().from(OWNER_ADDRESS_EXPR).esdt_transfer(
            UNIVERSAL_TOKEN_IDENTIFIER,
            0,
            10000u32,
        ),
        |sc| {
            sc.set_paused(false);
            sc.add_wrapped_token(managed_token_id!(UNIVERSAL_TOKEN_IDENTIFIER), NUM_DECIMALS);
            sc.chain_specific_to_universal_mapping(&managed_token_id!(UNIVERSAL_TOKEN_IDENTIFIER))
                .set(managed_token_id!(UNIVERSAL_TOKEN_IDENTIFIER));

            sc.deposit_liquidity();
            let result = sc
                .token_liquidity(&managed_token_id!(UNIVERSAL_TOKEN_IDENTIFIER))
                .get();
            assert_ne!(result, 0);
        },
    );

    world.whitebox_query(&bridged_tokens_wrapper, |sc| {
        let result = sc
            .chain_specific_to_universal_mapping(&managed_token_id!(UNIVERSAL_TOKEN_IDENTIFIER))
            .get();
        assert_eq!(result, managed_token_id!(UNIVERSAL_TOKEN_IDENTIFIER));
    });

    world.whitebox_call_check(
        &bridged_tokens_wrapper,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR)
            .esdt_transfer(UNIVERSAL_TOKEN_IDENTIFIER, 0, 100000u32)
            .expect(TxExpect::user_error(
                "str:Contract does not have enough funds",
            )),
        |sc| {
            sc.set_paused(false);
            let address = convert_to_eth_address(ETH_ADDRESS);
            sc.unwrap_token_create_transaction(
                managed_token_id!(UNIVERSAL_TOKEN_IDENTIFIER),
                ManagedAddress::zero(),
                address,
                OptionalValue::None,
            );
        },
        |r| r.assert_user_error("Contract does not have enough funds"),
    );
}

#[test]
fn test_unwrap_token_create_transaction_should_work() {
    let mut world = setup();
    let bridged_tokens_wrapper = WhiteboxContract::new(
        BRIDGE_TOKENS_WRAPPER_ADDRESS_EXPR,
        bridged_tokens_wrapper::contract_obj,
    );
    let contract_address = AddressValue::from(BRIDGE_TOKENS_WRAPPER_ADDRESS_EXPR);

    world.set_state_step(
        SetStateStep::new().put_account(
            OWNER_ADDRESS_EXPR,
            Account::new()
                .nonce(1)
                .balance(100_000_000u64)
                .esdt_balance(UNIVERSAL_TOKEN_IDENTIFIER.to_vec(), 100_000_000u64),
        ),
    );

    world.set_esdt_local_roles(
        managed_address!(&contract_address.to_address()),
        UNIVERSAL_TOKEN_IDENTIFIER,
        &[EsdtLocalRole::Mint, EsdtLocalRole::Burn],
    );

    world.whitebox_call(
        &bridged_tokens_wrapper,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR)
            .esdt_transfer(UNIVERSAL_TOKEN_IDENTIFIER, 0, 100u32)
            .no_expect(),
        |sc| {
            sc.set_paused(false);
            sc.add_wrapped_token(managed_token_id!(UNIVERSAL_TOKEN_IDENTIFIER), NUM_DECIMALS);
            sc.chain_specific_to_universal_mapping(&managed_token_id!(UNIVERSAL_TOKEN_IDENTIFIER))
                .set(managed_token_id!(UNIVERSAL_TOKEN_IDENTIFIER));

            sc.deposit_liquidity();
        },
    );
}

#[test]
fn test_whitelist_token_should_fail() {
    let mut world = setup();
    let bridged_tokens_wrapper = WhiteboxContract::new(
        BRIDGE_TOKENS_WRAPPER_ADDRESS_EXPR,
        bridged_tokens_wrapper::contract_obj,
    );
    let contract_address = AddressValue::from(BRIDGE_TOKENS_WRAPPER_ADDRESS_EXPR);

    world.set_esdt_local_roles(
        managed_address!(&contract_address.to_address()),
        UNIVERSAL_TOKEN_IDENTIFIER,
        &[EsdtLocalRole::Mint, EsdtLocalRole::Burn],
    );

    world.whitebox_call_check(
        &bridged_tokens_wrapper,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR)
            .expect(TxExpect::user_error(
                "str:Chain-specific token is already mapped to another universal token",
            )),
        |sc| {
            sc.chain_specific_to_universal_mapping(&managed_token_id!(CHAIN_TOKEN_IDENTIFIER2))
                .set(managed_token_id!(CHAIN_TOKEN_IDENTIFIER2));
            sc.whitelist_token(
                managed_token_id!(CHAIN_TOKEN_IDENTIFIER2),
                NUM_DECIMALS,
                managed_token_id!(UNIVERSAL_TOKEN_IDENTIFIER),
            );
        },
        |r| {
            r.assert_user_error("Chain-specific token is already mapped to another universal token")
        },
    );
}

#[test]
fn test_whitelist_token_should_work() {
    let mut world = setup();
    let bridged_tokens_wrapper = WhiteboxContract::new(
        BRIDGE_TOKENS_WRAPPER_ADDRESS_EXPR,
        bridged_tokens_wrapper::contract_obj,
    );
    let contract_address = AddressValue::from(BRIDGE_TOKENS_WRAPPER_ADDRESS_EXPR);

    world.set_esdt_local_roles(
        managed_address!(&contract_address.to_address()),
        UNIVERSAL_TOKEN_IDENTIFIER,
        &[EsdtLocalRole::Mint, EsdtLocalRole::Burn],
    );

    world.whitebox_call(
        &bridged_tokens_wrapper,
        ScCallStep::new().from(OWNER_ADDRESS_EXPR),
        |sc| {
            sc.whitelist_token(
                managed_token_id!(CHAIN_TOKEN_IDENTIFIER),
                NUM_DECIMALS,
                managed_token_id!(UNIVERSAL_TOKEN_IDENTIFIER),
            );
        },
    );
}

#[test]
fn test_blacklist_token_should_work() {
    let mut world = setup();
    let bridged_tokens_wrapper = WhiteboxContract::new(
        BRIDGE_TOKENS_WRAPPER_ADDRESS_EXPR,
        bridged_tokens_wrapper::contract_obj,
    );

    world.whitebox_call(
        &bridged_tokens_wrapper,
        ScCallStep::new().from(OWNER_ADDRESS_EXPR),
        |sc| {
            sc.chain_specific_to_universal_mapping(&managed_token_id!(CHAIN_TOKEN_IDENTIFIER))
                .set(managed_token_id!(CHAIN_TOKEN_IDENTIFIER));
            sc.blacklist_token(managed_token_id!(CHAIN_TOKEN_IDENTIFIER));
        },
    );

    world.whitebox_query(&bridged_tokens_wrapper, |sc| {
        let result = sc
            .chain_specific_to_universal_mapping(&managed_token_id!(CHAIN_TOKEN_IDENTIFIER))
            .is_empty();
        let token_decimals_result = sc
            .token_decimals_num(&managed_token_id!(CHAIN_TOKEN_IDENTIFIER))
            .is_empty();
        assert!(result);
        assert!(token_decimals_result);
    });
}

#[test]
fn test_wrap_tokens_should_fail_1() {
    let mut world = setup();
    let bridged_tokens_wrapper = WhiteboxContract::new(
        BRIDGE_TOKENS_WRAPPER_ADDRESS_EXPR,
        bridged_tokens_wrapper::contract_obj,
    );

    world.set_state_step(
        SetStateStep::new().put_account(
            OWNER_ADDRESS_EXPR,
            Account::new()
                .nonce(1)
                .balance(100_000_000u64)
                .esdt_balance(CHAIN_TOKEN_IDENTIFIER.to_vec(), 100_000_000u64),
        ),
    );

    world.whitebox_call_check(
        &bridged_tokens_wrapper,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR)
            .esdt_transfer(CHAIN_TOKEN_IDENTIFIER, 0, 100u32)
            .expect(TxExpect::user_error("str:Contract is paused")),
        |sc| {
            sc.wrap_tokens();
        },
        |r| r.assert_user_error("Contract is paused"),
    );
}

#[test]
fn test_wrap_tokens_should_work_case_1() {
    let mut world = setup();
    let bridged_tokens_wrapper = WhiteboxContract::new(
        BRIDGE_TOKENS_WRAPPER_ADDRESS_EXPR,
        bridged_tokens_wrapper::contract_obj,
    );

    world.set_state_step(
        SetStateStep::new().put_account(
            OWNER_ADDRESS_EXPR,
            Account::new()
                .nonce(1)
                .balance(100_000_000u64)
                .esdt_balance(CHAIN_TOKEN_IDENTIFIER.to_vec(), 100_000_000u64),
        ),
    );

    world.whitebox_call(
        &bridged_tokens_wrapper,
        ScCallStep::new().from(OWNER_ADDRESS_EXPR),
        |sc| {
            sc.set_paused(false);
            sc.wrap_tokens();
        },
    );
}

#[test]
fn test_wrap_tokens_should_work_case_2() {
    let mut world = setup();
    let bridged_tokens_wrapper = WhiteboxContract::new(
        BRIDGE_TOKENS_WRAPPER_ADDRESS_EXPR,
        bridged_tokens_wrapper::contract_obj,
    );

    world.set_state_step(
        SetStateStep::new().put_account(
            OWNER_ADDRESS_EXPR,
            Account::new()
                .nonce(1)
                .balance(100_000_000u64)
                .esdt_balance(CHAIN_TOKEN_IDENTIFIER.to_vec(), 100_000_000u64),
        ),
    );

    world.whitebox_call(
        &bridged_tokens_wrapper,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR)
            .esdt_transfer(CHAIN_TOKEN_IDENTIFIER, 0, 100u32),
        |sc| {
            sc.set_paused(false);
            sc.wrap_tokens();
        },
    );
}

#[test]
fn test_wrap_tokens_should_work_case_3() {
    let mut world = setup();
    let bridged_tokens_wrapper = WhiteboxContract::new(
        BRIDGE_TOKENS_WRAPPER_ADDRESS_EXPR,
        bridged_tokens_wrapper::contract_obj,
    );

    world.set_state_step(
        SetStateStep::new().put_account(
            OWNER_ADDRESS_EXPR,
            Account::new()
                .nonce(1)
                .balance(100_000_000u64)
                .esdt_balance(CHAIN_TOKEN_IDENTIFIER.to_vec(), 100_000_000u64),
        ),
    );

    world.set_esdt_local_roles(
        managed_address!(&AddressValue::from(BRIDGE_TOKENS_WRAPPER_ADDRESS_EXPR).to_address()),
        CHAIN_TOKEN_IDENTIFIER,
        &[EsdtLocalRole::Mint, EsdtLocalRole::Burn],
    );
    world.set_esdt_local_roles(
        managed_address!(&AddressValue::from(BRIDGE_TOKENS_WRAPPER_ADDRESS_EXPR).to_address()),
        UNIVERSAL_TOKEN_IDENTIFIER,
        &[EsdtLocalRole::Mint, EsdtLocalRole::Burn],
    );

    world.whitebox_call(
        &bridged_tokens_wrapper,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR)
            .esdt_transfer(CHAIN_TOKEN_IDENTIFIER, 0, 100u32),
        |sc| {
            sc.token_decimals_num(&managed_token_id!(CHAIN_TOKEN_IDENTIFIER))
                .set(NUM_DECIMALS);
            sc.token_decimals_num(&managed_token_id!(UNIVERSAL_TOKEN_IDENTIFIER))
                .set(NUM_DECIMALS);
            sc.chain_specific_to_universal_mapping(&managed_token_id!(CHAIN_TOKEN_IDENTIFIER))
                .set(managed_token_id!(UNIVERSAL_TOKEN_IDENTIFIER));
            sc.set_paused(false);
            sc.wrap_tokens();
        },
    );
}

#[test]
fn test_unwrap_token_should_fail_case_1() {
    let mut world = setup();
    let bridged_tokens_wrapper = WhiteboxContract::new(
        BRIDGE_TOKENS_WRAPPER_ADDRESS_EXPR,
        bridged_tokens_wrapper::contract_obj,
    );

    world.whitebox_call_check(
        &bridged_tokens_wrapper,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR)
            .expect(TxExpect::user_error("str:Contract is paused")),
        |sc| {
            sc.unwrap_token(managed_token_id!(CHAIN_TOKEN_IDENTIFIER));
        },
        |r| r.assert_user_error("Contract is paused"),
    );
}

#[test]
fn test_unwrap_token_should_fail_case_2() {
    let mut world = setup();
    let bridged_tokens_wrapper = WhiteboxContract::new(
        BRIDGE_TOKENS_WRAPPER_ADDRESS_EXPR,
        bridged_tokens_wrapper::contract_obj,
    );

    world.set_state_step(
        SetStateStep::new().put_account(
            OWNER_ADDRESS_EXPR,
            Account::new()
                .nonce(1)
                .balance(100_000_000u64)
                .esdt_balance(CHAIN_TOKEN_IDENTIFIER.to_vec(), 100_000_000u64),
        ),
    );

    world.whitebox_call_check(
        &bridged_tokens_wrapper,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR)
            .esdt_transfer(CHAIN_TOKEN_IDENTIFIER, 0, 0u32)
            .expect(TxExpect::user_error("str:Must pay more than 0 tokens!")),
        |sc| {
            sc.set_paused(false);
            sc.unwrap_token(managed_token_id!(CHAIN_TOKEN_IDENTIFIER));
        },
        |r| r.assert_user_error("Must pay more than 0 tokens!"),
    );
}

#[test]
fn test_unwrap_token_should_fail_case_3() {
    let mut world = setup();
    let bridged_tokens_wrapper = WhiteboxContract::new(
        BRIDGE_TOKENS_WRAPPER_ADDRESS_EXPR,
        bridged_tokens_wrapper::contract_obj,
    );

    world.set_state_step(
        SetStateStep::new().put_account(
            OWNER_ADDRESS_EXPR,
            Account::new()
                .nonce(1)
                .balance(100_000_000u64)
                .esdt_balance(CHAIN_TOKEN_IDENTIFIER.to_vec(), 100_000_000u64),
        ),
    );

    world.set_esdt_local_roles(
        managed_address!(&AddressValue::from(BRIDGE_TOKENS_WRAPPER_ADDRESS_EXPR).to_address()),
        CHAIN_TOKEN_IDENTIFIER,
        &[EsdtLocalRole::Mint, EsdtLocalRole::Burn],
    );
    world.set_esdt_local_roles(
        managed_address!(&AddressValue::from(BRIDGE_TOKENS_WRAPPER_ADDRESS_EXPR).to_address()),
        UNIVERSAL_TOKEN_IDENTIFIER,
        &[EsdtLocalRole::Mint, EsdtLocalRole::Burn],
    );

    world.whitebox_call_check(
        &bridged_tokens_wrapper,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR)
            .esdt_transfer(CHAIN_TOKEN_IDENTIFIER, 0, 100u32)
            .expect(TxExpect::user_error("str:Esdt token unavailable")),
        |sc| {
            sc.set_paused(false);
            sc.unwrap_token(managed_token_id!(CHAIN_TOKEN_IDENTIFIER));
        },
        |r| r.assert_user_error("Esdt token unavailable"),
    );
}

#[test]
fn test_unwrap_token_should_fail_case_4() {
    let mut world = setup();
    let bridged_tokens_wrapper = WhiteboxContract::new(
        BRIDGE_TOKENS_WRAPPER_ADDRESS_EXPR,
        bridged_tokens_wrapper::contract_obj,
    );

    world.set_state_step(
        SetStateStep::new().put_account(
            OWNER_ADDRESS_EXPR,
            Account::new()
                .nonce(1)
                .balance(100_000_000u64)
                .esdt_balance(CHAIN_TOKEN_IDENTIFIER.to_vec(), 100_000_000u64),
        ),
    );

    world.set_esdt_local_roles(
        managed_address!(&AddressValue::from(BRIDGE_TOKENS_WRAPPER_ADDRESS_EXPR).to_address()),
        CHAIN_TOKEN_IDENTIFIER,
        &[EsdtLocalRole::Mint, EsdtLocalRole::Burn],
    );
    world.set_esdt_local_roles(
        managed_address!(&AddressValue::from(BRIDGE_TOKENS_WRAPPER_ADDRESS_EXPR).to_address()),
        UNIVERSAL_TOKEN_IDENTIFIER,
        &[EsdtLocalRole::Mint, EsdtLocalRole::Burn],
    );

    world.whitebox_call(
        &bridged_tokens_wrapper,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR)
            .esdt_transfer(CHAIN_TOKEN_IDENTIFIER, 0, 10000u32)
            .no_expect(),
        |sc| {
            sc.set_paused(false);
            sc.add_wrapped_token(managed_token_id!(CHAIN_TOKEN_IDENTIFIER), NUM_DECIMALS);
            sc.chain_specific_to_universal_mapping(&managed_token_id!(CHAIN_TOKEN_IDENTIFIER))
                .set(managed_token_id!(CHAIN_TOKEN_IDENTIFIER));

            sc.deposit_liquidity();
        },
    );

    world.whitebox_call_check(
        &bridged_tokens_wrapper,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR)
            .esdt_transfer(CHAIN_TOKEN_IDENTIFIER, 0, 100000u32)
            .expect(TxExpect::user_error(
                "str:Contract does not have enough funds",
            )),
        |sc| {
            sc.unwrap_token(managed_token_id!(CHAIN_TOKEN_IDENTIFIER));
        },
        |r| r.assert_user_error("Contract does not have enough funds"),
    );
}

#[test]
fn test_unwrap_token_should_work() {
    let mut world = setup();
    let bridged_tokens_wrapper = WhiteboxContract::new(
        BRIDGE_TOKENS_WRAPPER_ADDRESS_EXPR,
        bridged_tokens_wrapper::contract_obj,
    );

    world.set_state_step(
        SetStateStep::new().put_account(
            OWNER_ADDRESS_EXPR,
            Account::new()
                .nonce(1)
                .balance(100_000_000u64)
                .esdt_balance(CHAIN_TOKEN_IDENTIFIER.to_vec(), 100_000_000u64),
        ),
    );

    world.set_esdt_local_roles(
        managed_address!(&AddressValue::from(BRIDGE_TOKENS_WRAPPER_ADDRESS_EXPR).to_address()),
        CHAIN_TOKEN_IDENTIFIER,
        &[EsdtLocalRole::Mint, EsdtLocalRole::Burn],
    );
    world.set_esdt_local_roles(
        managed_address!(&AddressValue::from(BRIDGE_TOKENS_WRAPPER_ADDRESS_EXPR).to_address()),
        UNIVERSAL_TOKEN_IDENTIFIER,
        &[EsdtLocalRole::Mint, EsdtLocalRole::Burn],
    );

    world.whitebox_call(
        &bridged_tokens_wrapper,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR)
            .esdt_transfer(CHAIN_TOKEN_IDENTIFIER, 0, 10000u32)
            .no_expect(),
        |sc| {
            sc.set_paused(false);
            sc.add_wrapped_token(managed_token_id!(CHAIN_TOKEN_IDENTIFIER), NUM_DECIMALS);
            sc.chain_specific_to_universal_mapping(&managed_token_id!(CHAIN_TOKEN_IDENTIFIER))
                .set(managed_token_id!(CHAIN_TOKEN_IDENTIFIER));

            sc.deposit_liquidity();
        },
    );

    world.whitebox_call(
        &bridged_tokens_wrapper,
        ScCallStep::new()
            .from(OWNER_ADDRESS_EXPR)
            .esdt_transfer(CHAIN_TOKEN_IDENTIFIER, 0, 100u32)
            .no_expect(),
        |sc| {
            sc.unwrap_token(managed_token_id!(CHAIN_TOKEN_IDENTIFIER));
        },
    );
}

fn setup() -> ScenarioWorld {
    let mut world = world();
    let bridged_tokens_wrapper_whitebox = WhiteboxContract::new(
        BRIDGE_TOKENS_WRAPPER_ADDRESS_EXPR,
        bridged_tokens_wrapper::contract_obj,
    );

    let bridged_tokens_wrapper_code = world.code_expression(BRIDGE_TOKENS_WRAPPER_PATH_EXPR);

    let set_state_step = SetStateStep::new()
        .put_account(OWNER_ADDRESS_EXPR, Account::new().nonce(1))
        .new_address(OWNER_ADDRESS_EXPR, 1, BRIDGE_TOKENS_WRAPPER_ADDRESS_EXPR)
        .block_timestamp(100);

    world.set_state_step(set_state_step).whitebox_deploy(
        &bridged_tokens_wrapper_whitebox,
        ScDeployStep::new()
            .from(OWNER_ADDRESS_EXPR)
            .code(bridged_tokens_wrapper_code),
        |sc| {
            sc.init();
        },
    );

    world
}

fn convert_to_eth_address(address: &str) -> EthAddress<DebugApi> {
    let address_str = address.trim_start_matches("0x");
    let mut address_bytes = [0u8; 20];

    for (i, byte) in address_bytes.iter_mut().enumerate() {
        let offset = i * 2;
        *byte = u8::from_str_radix(&address_str[offset..offset + 2], 16).expect("Parsing error");
    }

    EthAddress {
        raw_addr: ManagedByteArray::new_from_bytes(&address_bytes),
    }
}
