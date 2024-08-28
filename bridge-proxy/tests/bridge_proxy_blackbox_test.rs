#![allow(unused)]

use std::collections::LinkedList;
use std::ops::Add;

use bridge_proxy::{bridge_proxy_contract_proxy, config::ProxyTrait as _};
use bridge_proxy::{bridged_tokens_wrapper_proxy, ProxyTrait};

use crowdfunding_esdt::crowdfunding_esdt_proxy;
use multiversx_sc::codec::NestedEncode;
use multiversx_sc::contract_base::ManagedSerializer;
use multiversx_sc::sc_print;
use multiversx_sc::types::{
    EgldOrEsdtTokenIdentifier, EsdtTokenPayment, ManagedOption, ReturnsNewAddress, TestAddress,
    TestSCAddress, TestTokenIdentifier,
};
use multiversx_sc::{
    api::{HandleConstraints, ManagedTypeApi},
    codec::{
        multi_types::{MultiValueVec, OptionalValue},
        TopEncodeMultiOutput,
    },
    storage::mappers::SingleValue,
    types::{
        Address, BigUint, CodeMetadata, ManagedAddress, ManagedArgBuffer, ManagedBuffer,
        ManagedByteArray, ManagedVec, TokenIdentifier,
    },
};
use multiversx_sc_scenario::imports::MxscPath;
use multiversx_sc_scenario::{
    api::StaticApi,
    rust_biguint,
    scenario_format::interpret_trait::{InterpretableFrom, InterpreterContext},
    scenario_model::*,
    ContractInfo, ScenarioWorld,
};
use multiversx_sc_scenario::{ExpectValue, ScenarioTxRun};

use eth_address::*;
use transaction::{CallData, EthTransaction};

const BRIDGE_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("BRIDGE-123456");
const WBRIDGE_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("WBRIDGE-123456");

const GAS_LIMIT: u64 = 10_000_000;
const CF_DEADLINE: u64 = 7 * 24 * 60 * 60; // 1 week in seconds

const OWNER_ADDRESS: TestAddress = TestAddress::new("owner");
const BRIDGE_PROXY_ADDRESS: TestSCAddress = TestSCAddress::new("bridge-proxy");
const CROWDFUNDING_ADDRESS: TestSCAddress = TestSCAddress::new("crowfunding");
const MULTI_TRANSFER_ADDRESS: TestSCAddress = TestSCAddress::new("multi-transfer");
const ESDT_SAFE_ADDRESS: TestSCAddress = TestSCAddress::new("esdt-safe");
const BRIDGED_TOKENS_WRAPPER_ADDRESS: TestSCAddress = TestSCAddress::new("bridged-tokens-wrapper");

const BRIDGE_PROXY_PATH_EXPR: MxscPath = MxscPath::new("output/bridge-proxy.mxsc.json");
const CROWDFUNDING_PATH_EXPR: MxscPath =
    MxscPath::new("tests/test-contract/crowdfunding-esdt.mxsc.json");
const MULTI_TRANSFER_PATH_EXPR: &str =
    "mxsc:../multi-transfer-esdt/output/multi-transfer-esdt.mxsc.json";
const ESDT_SAFE_PATH_EXPR: &str = "mxsc:../esdt-safe/output/esdt-safe.mxsc.json";
const BRIDGED_TOKENS_WRAPPER_CODE_PATH_EXPR: MxscPath =
    MxscPath::new("../bridged-tokens-wrapper/output/bridged-tokens-wrapper.mxsc.json");

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(BRIDGE_PROXY_PATH_EXPR, bridge_proxy::ContractBuilder);
    blockchain.register_contract(CROWDFUNDING_PATH_EXPR, crowdfunding_esdt::ContractBuilder);
    blockchain.register_contract(
        BRIDGED_TOKENS_WRAPPER_CODE_PATH_EXPR,
        bridged_tokens_wrapper::ContractBuilder,
    );
    blockchain.register_contract(ESDT_SAFE_PATH_EXPR, esdt_safe::ContractBuilder);

    blockchain
}

type BridgeProxyContract = ContractInfo<bridge_proxy::Proxy<StaticApi>>;
type CrowdfundingContract = ContractInfo<crowdfunding_esdt::Proxy<StaticApi>>;
type BridgedTokensWrapperContract = ContractInfo<bridged_tokens_wrapper::Proxy<StaticApi>>;

struct BridgeProxyTestState {
    world: ScenarioWorld,
}

impl BridgeProxyTestState {
    fn new() -> Self {
        let mut world = world();
        let multi_transfer_code = world.code_expression(MULTI_TRANSFER_PATH_EXPR);
        let esdt_safe_code = world.code_expression(ESDT_SAFE_PATH_EXPR);

        world
            .account(OWNER_ADDRESS)
            .nonce(1)
            .esdt_balance(TokenIdentifier::from(BRIDGE_TOKEN_ID), 10_000u64)
            .account(MULTI_TRANSFER_ADDRESS)
            .esdt_balance(TokenIdentifier::from(WBRIDGE_TOKEN_ID), 10_000u64)
            .esdt_balance(TokenIdentifier::from(BRIDGE_TOKEN_ID), 10_000u64)
            .code(multi_transfer_code)
            .account(ESDT_SAFE_ADDRESS)
            .code(esdt_safe_code);

        let roles = vec![
            "ESDTRoleLocalMint".to_string(),
            "ESDTRoleLocalBurn".to_string(),
        ];
        world
            .account(BRIDGED_TOKENS_WRAPPER_ADDRESS)
            .esdt_roles(WBRIDGE_TOKEN_ID, roles.clone())
            .esdt_roles(BRIDGE_TOKEN_ID, roles)
            .esdt_balance(TokenIdentifier::from(WBRIDGE_TOKEN_ID), 10_000u64)
            .esdt_balance(TokenIdentifier::from(BRIDGE_TOKEN_ID), 10_000u64)
            .code(BRIDGED_TOKENS_WRAPPER_CODE_PATH_EXPR)
            .owner(OWNER_ADDRESS);

        Self { world }
    }

    fn bridge_proxy_deploy(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
            .init(OptionalValue::Some(MULTI_TRANSFER_ADDRESS))
            .code(BRIDGE_PROXY_PATH_EXPR)
            .new_address(BRIDGE_PROXY_ADDRESS)
            .run();

        self
    }

    fn bridged_tokens_wrapper_deploy(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
            .init()
            .code(BRIDGED_TOKENS_WRAPPER_CODE_PATH_EXPR)
            .new_address(BRIDGED_TOKENS_WRAPPER_ADDRESS)
            .run();

        self
    }

    fn deploy_crowdfunding(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(crowdfunding_esdt_proxy::CrowdfundingProxy)
            .init(
                2_000u32,
                CF_DEADLINE,
                EgldOrEsdtTokenIdentifier::esdt(BRIDGE_TOKEN_ID),
            )
            .code(CROWDFUNDING_PATH_EXPR)
            .new_address(CROWDFUNDING_ADDRESS)
            .run();
        self
    }

    fn config_bridge(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(BRIDGE_PROXY_ADDRESS)
            .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
            .unpause_endpoint()
            .run();

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(BRIDGED_TOKENS_WRAPPER_ADDRESS)
            .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
            .unpause_endpoint()
            .run();

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(BRIDGE_PROXY_ADDRESS)
            .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
            .set_bridged_tokens_wrapper(OptionalValue::Some(BRIDGED_TOKENS_WRAPPER_ADDRESS))
            .run();

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(BRIDGED_TOKENS_WRAPPER_ADDRESS)
            .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
            .whitelist_token(BRIDGE_TOKEN_ID, 18u32, WBRIDGE_TOKEN_ID)
            .run();

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(BRIDGED_TOKENS_WRAPPER_ADDRESS)
            .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
            .add_wrapped_token(WBRIDGE_TOKEN_ID, 18u32)
            .run();

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(BRIDGED_TOKENS_WRAPPER_ADDRESS)
            .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
            .deposit_liquidity()
            .single_esdt(
                &TokenIdentifier::from(BRIDGE_TOKEN_ID),
                0u64,
                &BigUint::from(5_000u64),
            )
            .run();

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(BRIDGED_TOKENS_WRAPPER_ADDRESS)
            .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
            .set_esdt_safe_contract_address(OptionalValue::Some(ESDT_SAFE_ADDRESS))
            .run();
        self
    }
}

#[test]
fn deploy_test() {
    let mut test = BridgeProxyTestState::new();

    test.bridge_proxy_deploy();
    test.deploy_crowdfunding();
    test.config_bridge();
}

#[test]
fn bridge_proxy_execute_crowdfunding_test() {
    let mut test = BridgeProxyTestState::new();

    test.world.start_trace();

    test.bridge_proxy_deploy();
    test.deploy_crowdfunding();
    test.config_bridge();

    let mut args = ManagedVec::new();

    let call_data: CallData<StaticApi> = CallData {
        endpoint: ManagedBuffer::from("fund"),
        gas_limit: GAS_LIMIT,
        args: ManagedOption::some(args),
    };

    let call_data: ManagedBuffer<StaticApi> =
        ManagedSerializer::new().top_encode_to_managed_buffer(&call_data);

    let eth_tx = EthTransaction {
        from: EthAddress {
            raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
        },
        to: ManagedAddress::from(CROWDFUNDING_ADDRESS.eval_to_array()),
        token_id: BRIDGE_TOKEN_ID.into(),
        amount: BigUint::from(500u64),
        tx_nonce: 1u64,
        call_data: ManagedOption::some(call_data),
    };

    test.world
        .tx()
        .from(MULTI_TRANSFER_ADDRESS)
        .to(BRIDGE_PROXY_ADDRESS)
        .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
        .deposit(&eth_tx)
        .egld_or_single_esdt(
            &EgldOrEsdtTokenIdentifier::esdt(BRIDGE_TOKEN_ID),
            0,
            &BigUint::from(500u64),
        )
        .run();

    test.world
        .query()
        .to(BRIDGE_PROXY_ADDRESS)
        .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
        .get_pending_transaction_by_id(1u32)
        .returns(ExpectValue(eth_tx))
        .run();

    test.world
        .tx()
        .from(OWNER_ADDRESS)
        .to(BRIDGE_PROXY_ADDRESS)
        .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
        .execute(1u32)
        .run();

    test.world
        .query()
        .to(CROWDFUNDING_ADDRESS)
        .typed(crowdfunding_esdt_proxy::CrowdfundingProxy)
        .get_current_funds()
        .returns(ExpectValue(500u64))
        .run();

    test.world
        .write_scenario_trace("scenarios/bridge_proxy_execute_crowdfunding.scen.json");
}

#[test]
fn multiple_deposit_test() {
    let mut test = BridgeProxyTestState::new();

    test.bridge_proxy_deploy();
    test.deploy_crowdfunding();
    test.config_bridge();

    let mut args = ManagedVec::new();

    let call_data: CallData<StaticApi> = CallData {
        endpoint: ManagedBuffer::from(b"fund"),
        gas_limit: GAS_LIMIT,
        args: ManagedOption::some(args),
    };
    let call_data = ManagedSerializer::new().top_encode_to_managed_buffer(&call_data);

    let eth_tx1 = EthTransaction {
        from: EthAddress {
            raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
        },
        to: ManagedAddress::from(CROWDFUNDING_ADDRESS.eval_to_array()),
        token_id: BRIDGE_TOKEN_ID.into(),
        amount: BigUint::from(500u64),
        tx_nonce: 1u64,
        call_data: ManagedOption::some(call_data.clone()),
    };

    let eth_tx2 = EthTransaction {
        from: EthAddress {
            raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
        },
        to: ManagedAddress::from(CROWDFUNDING_ADDRESS.eval_to_array()),
        token_id: BRIDGE_TOKEN_ID.into(),
        amount: BigUint::from(500u64),
        tx_nonce: 2u64,
        call_data: ManagedOption::some(call_data),
    };

    test.world
        .tx()
        .from(MULTI_TRANSFER_ADDRESS)
        .to(BRIDGE_PROXY_ADDRESS)
        .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
        .deposit(&eth_tx1)
        .single_esdt(
            &TokenIdentifier::from(BRIDGE_TOKEN_ID),
            0u64,
            &BigUint::from(500u64),
        )
        .run();

    test.world
        .tx()
        .from(MULTI_TRANSFER_ADDRESS)
        .to(BRIDGE_PROXY_ADDRESS)
        .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
        .deposit(&eth_tx2)
        .single_esdt(
            &TokenIdentifier::from(BRIDGE_TOKEN_ID),
            0u64,
            &BigUint::from(500u64),
        )
        .run();

    test.world
        .query()
        .to(BRIDGE_PROXY_ADDRESS)
        .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
        .get_pending_transaction_by_id(1u32)
        .returns(ExpectValue(eth_tx1))
        .run();

    test.world
        .tx()
        .from(OWNER_ADDRESS)
        .to(BRIDGE_PROXY_ADDRESS)
        .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
        .execute(1u32)
        .run();

    test.world
        .query()
        .to(CROWDFUNDING_ADDRESS)
        .typed(crowdfunding_esdt_proxy::CrowdfundingProxy)
        .get_current_funds()
        .returns(ExpectValue(500u64))
        .run();

    test.world
        .tx()
        .from(OWNER_ADDRESS)
        .to(BRIDGE_PROXY_ADDRESS)
        .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
        .execute(2u32)
        .run();

    test.world
        .query()
        .to(CROWDFUNDING_ADDRESS)
        .typed(crowdfunding_esdt_proxy::CrowdfundingProxy)
        .get_current_funds()
        .returns(ExpectValue(BigUint::from(1_000u32)))
        .run();

    test.world
        .query()
        .to(CROWDFUNDING_ADDRESS)
        .typed(crowdfunding_esdt_proxy::CrowdfundingProxy)
        .get_current_funds()
        .returns(ExpectValue(1_000u64))
        .run();
}

#[test]
fn test_lowest_tx_id() {
    let mut test = BridgeProxyTestState::new();

    test.bridge_proxy_deploy();
    test.deploy_crowdfunding();
    test.config_bridge();

    let mut args = ManagedVec::new();

    let call_data: CallData<StaticApi> = CallData {
        endpoint: ManagedBuffer::from(b"fund"),
        gas_limit: GAS_LIMIT,
        args: ManagedOption::some(args),
    };
    let call_data = ManagedSerializer::new().top_encode_to_managed_buffer(&call_data);

    // Generate 100 transactions
    let mut transactions = Vec::new();
    for i in 1..=100 {
        let eth_tx = EthTransaction {
            from: EthAddress {
                raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
            },
            to: ManagedAddress::from(CROWDFUNDING_ADDRESS.eval_to_array()),
            token_id: BRIDGE_TOKEN_ID.into(),
            amount: BigUint::from(5u64),
            tx_nonce: i as u64,
            call_data: ManagedOption::some(call_data.clone()),
        };
        transactions.push(eth_tx);
    }

    // Deposit all transactions
    for tx in &transactions {
        test.world
            .tx()
            .from(MULTI_TRANSFER_ADDRESS)
            .to(BRIDGE_PROXY_ADDRESS)
            .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
            .deposit(tx)
            .single_esdt(
                &TokenIdentifier::from(BRIDGE_TOKEN_ID),
                0u64,
                &BigUint::from(5u64),
            )
            .run();
    }

    // Check the lowest_tx_id
    test.world
        .query()
        .to(BRIDGE_PROXY_ADDRESS)
        .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
        .lowest_tx_id()
        .returns(ExpectValue(1usize))
        .run();

    // Execute the first 50 transactions
    for i in 1..=50usize {
        test.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(BRIDGE_PROXY_ADDRESS)
            .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
            .execute(i)
            .run();
    }

    // Check the lowest_tx_id again
    test.world
        .query()
        .to(BRIDGE_PROXY_ADDRESS)
        .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
        .lowest_tx_id()
        .returns(ExpectValue(51usize))
        .run();

    // Execute transactions 75 to 100
    for i in 75..=100usize {
        test.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(BRIDGE_PROXY_ADDRESS)
            .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
            .execute(i)
            .run();
    }

    // Check the lowest_tx_id one last time
    test.world
        .query()
        .to(BRIDGE_PROXY_ADDRESS)
        .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
        .lowest_tx_id()
        .returns(ExpectValue(51usize))
        .run();
}


