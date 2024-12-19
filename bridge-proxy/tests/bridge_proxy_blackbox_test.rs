#![allow(unused)]

use std::collections::LinkedList;
use std::ops::Add;

use bridge_proxy::config::ProxyTrait as _;

use crowdfunding_esdt::crowdfunding_esdt_proxy;
use multiversx_sc::codec::NestedEncode;
use multiversx_sc::contract_base::ManagedSerializer;
use multiversx_sc::sc_print;
use multiversx_sc::types::{
    EgldOrEsdtTokenIdentifier, EsdtTokenPayment, ManagedOption, MultiValueEncoded,
    ReturnsNewAddress, ReturnsResult, TestAddress, TestSCAddress, TestTokenIdentifier,
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
use multiversx_sc_scenario::{ExpectError, ExpectValue, ScenarioTxRun};

use eth_address::*;
use mock_proxies::mock_multisig_proxy;
use sc_proxies::{bridge_proxy_contract_proxy, bridged_tokens_wrapper_proxy, esdt_safe_proxy};
use transaction::{CallData, EthTransaction};

const BRIDGE_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("BRIDGE-123456");
const WBRIDGE_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("WBRIDGE-123456");

const DELAY_BEFORE_OWNER_CAN_REFUND_TRANSACTION: u64 = 300;
const GAS_LIMIT: u64 = 10_000_000;
const TOO_SMALL_GAS_LIMIT: u64 = 1_000_000;

const CF_DEADLINE: u64 = 7 * 24 * 60 * 60; // 1 week in seconds
const INITIAL_BALANCE: u64 = 10_000u64;

const OWNER_ADDRESS: TestAddress = TestAddress::new("owner");
const USER_ADDRESS: TestAddress = TestAddress::new("user");
const BRIDGE_PROXY_ADDRESS: TestSCAddress = TestSCAddress::new("bridge-proxy");
const CROWDFUNDING_ADDRESS: TestSCAddress = TestSCAddress::new("crowfunding");
const MULTI_TRANSFER_ADDRESS: TestSCAddress = TestSCAddress::new("multi-transfer");
const ESDT_SAFE_ADDRESS: TestSCAddress = TestSCAddress::new("esdt-safe");
const FEE_ESTIMATOR_ADDRESS: TestSCAddress = TestSCAddress::new("fee-estimator");
const MULTISIG_ADDRESS: TestSCAddress = TestSCAddress::new("multisig");
const BRIDGED_TOKENS_WRAPPER_ADDRESS: TestSCAddress = TestSCAddress::new("bridged-tokens-wrapper");
const NO_INIT_SC_ADDRESS: TestSCAddress = TestSCAddress::new("no-init-sc");

const BRIDGE_PROXY_PATH_EXPR: MxscPath = MxscPath::new("output/bridge-proxy.mxsc.json");
const CROWDFUNDING_PATH_EXPR: MxscPath =
    MxscPath::new("tests/test-contract/crowdfunding-esdt.mxsc.json");
const MOCK_MULTI_TRANSFER_PATH_EXPR: MxscPath = MxscPath::new(
    "../common/mock-contracts/mock-multi-transfer-esdt/output/mock-multi-transfer-esdt.mxsc.json",
);
const MOCK_ESDT_SAFE_PATH_EXPR: MxscPath =
    MxscPath::new("../common/mock-contrats/mock-esdt-safe/output/mock-esdt-safe.mxsc.json");
const MOCK_BRIDGED_TOKENS_WRAPPER_CODE_PATH_EXPR: MxscPath =
    MxscPath::new("../common/mock-contracts/mock-bridged-tokens-wrapper/output/mock-bridged-tokens-wrapper.mxsc.json");
const MOCK_MULTISIG_CODE_PATH: MxscPath =
    MxscPath::new("../common/mock-contracts/mock-multisig/output/mock-multisig.mxsc.json");
const MOCK_PRICE_AGGREGATOR_CODE_PATH: MxscPath = MxscPath::new(
    "../common/mock-contracts/mock-price-aggregator/output/mock-price-aggregator.mxsc.json",
);
const USER1_ADDRESS: TestAddress = TestAddress::new("user1");
const USER2_ADDRESS: TestAddress = TestAddress::new("user2");
const RELAYER1_ADDRESS: TestAddress = TestAddress::new("relayer1");
const RELAYER2_ADDRESS: TestAddress = TestAddress::new("relayer2");

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(BRIDGE_PROXY_PATH_EXPR, bridge_proxy::ContractBuilder);
    blockchain.register_contract(CROWDFUNDING_PATH_EXPR, crowdfunding_esdt::ContractBuilder);
    blockchain.register_contract(
        MOCK_BRIDGED_TOKENS_WRAPPER_CODE_PATH_EXPR,
        mock_bridged_tokens_wrapper::ContractBuilder,
    );
    blockchain.register_contract(
        MOCK_PRICE_AGGREGATOR_CODE_PATH,
        mock_price_aggregator::ContractBuilder,
    );
    blockchain.register_contract(
        MOCK_MULTI_TRANSFER_PATH_EXPR,
        mock_multi_transfer_esdt::ContractBuilder,
    );
    blockchain.register_contract(MOCK_ESDT_SAFE_PATH_EXPR, mock_esdt_safe::ContractBuilder);
    blockchain.register_contract(MOCK_MULTISIG_CODE_PATH, mock_multisig::ContractBuilder);

    blockchain
}

type BridgeProxyContract = ContractInfo<bridge_proxy::Proxy<StaticApi>>;
type CrowdfundingContract = ContractInfo<crowdfunding_esdt::Proxy<StaticApi>>;

struct BridgeProxyTestState {
    world: ScenarioWorld,
}

impl BridgeProxyTestState {
    fn new() -> Self {
        let mut world = world();
        let multi_transfer_code =
            world.code_expression(MOCK_MULTI_TRANSFER_PATH_EXPR.eval_to_expr().as_str());
        let esdt_safe_code =
            world.code_expression(MOCK_ESDT_SAFE_PATH_EXPR.eval_to_expr().as_str());

        world
            .account(OWNER_ADDRESS)
            .nonce(1)
            .esdt_balance(TokenIdentifier::from(BRIDGE_TOKEN_ID), INITIAL_BALANCE)
            .account(USER_ADDRESS)
            .nonce(1)
            .esdt_balance(TokenIdentifier::from(BRIDGE_TOKEN_ID), INITIAL_BALANCE)
            .account(MULTI_TRANSFER_ADDRESS)
            .esdt_balance(TokenIdentifier::from(WBRIDGE_TOKEN_ID), INITIAL_BALANCE)
            .esdt_balance(TokenIdentifier::from(BRIDGE_TOKEN_ID), INITIAL_BALANCE)
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
            .code(MOCK_BRIDGED_TOKENS_WRAPPER_CODE_PATH_EXPR)
            .owner(OWNER_ADDRESS);

        Self { world }
    }

    fn deploy_bridge_proxy(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(MULTISIG_ADDRESS)
            .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
            .init()
            .code(BRIDGE_PROXY_PATH_EXPR)
            .new_address(BRIDGE_PROXY_ADDRESS)
            .run();

        self
    }

    fn multisig_deploy(&mut self) -> &mut Self {
        let mut board: MultiValueEncoded<StaticApi, ManagedAddress<StaticApi>> =
            MultiValueEncoded::new();
        board.push(ManagedAddress::from(RELAYER1_ADDRESS.eval_to_array()));
        board.push(ManagedAddress::from(RELAYER2_ADDRESS.eval_to_array()));
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(mock_multisig_proxy::MockMultisigProxy)
            .init(
                ESDT_SAFE_ADDRESS,
                MULTI_TRANSFER_ADDRESS,
                BRIDGE_PROXY_ADDRESS,
                BRIDGED_TOKENS_WRAPPER_ADDRESS,
                FEE_ESTIMATOR_ADDRESS,
                1_000u64,
                500u64,
                2usize,
                board,
            )
            .code(MOCK_MULTISIG_CODE_PATH)
            .new_address(MULTISIG_ADDRESS)
            .run();
        self
    }

    fn deploy_esdt_safe(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(MULTISIG_ADDRESS)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .init(BigUint::zero())
            .code(MOCK_ESDT_SAFE_PATH_EXPR)
            .new_address(BRIDGE_PROXY_ADDRESS)
            .run();

        self
    }

    fn deploy_bridged_tokens_wrapper(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(MULTISIG_ADDRESS)
            .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
            .init()
            .code(MOCK_BRIDGED_TOKENS_WRAPPER_CODE_PATH_EXPR)
            .new_address(BRIDGED_TOKENS_WRAPPER_ADDRESS)
            .run();

        self
    }

    fn deploy_crowdfunding(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(MULTISIG_ADDRESS)
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
            .from(MULTISIG_ADDRESS)
            .to(BRIDGE_PROXY_ADDRESS)
            .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
            .unpause_endpoint()
            .run();

        self
    }
    fn set_block_round(&mut self, block_round_expr: u64) {
        self.world.current_block().block_round(block_round_expr);
    }
}

#[test]
fn deploy_test() {
    let mut test = BridgeProxyTestState::new();
    test.multisig_deploy();
    test.deploy_bridge_proxy();
    test.deploy_crowdfunding();

    test.config_bridge();

    test.world
        .query()
        .to(MULTISIG_ADDRESS)
        .typed(mock_multisig_proxy::MockMultisigProxy)
        .multi_transfer_esdt_address()
        .returns(ExpectValue(MULTI_TRANSFER_ADDRESS))
        .run();

    test.world
        .query()
        .to(MULTISIG_ADDRESS)
        .typed(mock_multisig_proxy::MockMultisigProxy)
        .esdt_safe_address()
        .returns(ExpectValue(ESDT_SAFE_ADDRESS))
        .run();

    test.world
        .query()
        .to(MULTISIG_ADDRESS)
        .typed(mock_multisig_proxy::MockMultisigProxy)
        .bridged_tokens_wrapper_address()
        .returns(ExpectValue(BRIDGED_TOKENS_WRAPPER_ADDRESS))
        .run();
}

#[test]
fn bridge_proxy_execute_crowdfunding_test() {
    let mut test = BridgeProxyTestState::new();
    test.multisig_deploy();

    test.deploy_bridge_proxy();
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
        .deposit(&eth_tx, 1u64)
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
        .gas(200_000_000)
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
}

#[test]
fn multiple_deposit_test() {
    let mut test = BridgeProxyTestState::new();

    test.multisig_deploy();
    test.deploy_bridge_proxy();
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
        .deposit(&eth_tx1, 1u64)
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
        .deposit(&eth_tx2, 1u64)
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
        .gas(200_000_000)
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
        .gas(200_000_000)
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
fn test_highest_tx_id() {
    let mut test = BridgeProxyTestState::new();

    test.multisig_deploy();
    test.deploy_bridge_proxy();
    test.deploy_crowdfunding();
    test.config_bridge();

    let mut args = ManagedVec::new();

    let call_data: CallData<StaticApi> = CallData {
        endpoint: ManagedBuffer::from(b"fund"),
        gas_limit: GAS_LIMIT,
        args: ManagedOption::some(args),
    };
    let call_data = ManagedSerializer::new().top_encode_to_managed_buffer(&call_data);

    // Generate 1600 transactions
    let mut transactions = Vec::new();
    for i in 1..=1600 {
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
    test.world
        .query()
        .to(BRIDGE_PROXY_ADDRESS)
        .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
        .highest_tx_id()
        .returns(ExpectValue(0usize))
        .run();

    // Deposit all transactions
    let mut expected_tx_id = 1usize;
    for tx in &transactions {
        test.world
            .tx()
            .from(MULTI_TRANSFER_ADDRESS)
            .to(BRIDGE_PROXY_ADDRESS)
            .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
            .deposit(tx, 1u64)
            .single_esdt(
                &TokenIdentifier::from(BRIDGE_TOKEN_ID),
                0u64,
                &BigUint::from(5u64),
            )
            .run();

        test.world
            .query()
            .to(BRIDGE_PROXY_ADDRESS)
            .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
            .highest_tx_id()
            .returns(ExpectValue(expected_tx_id))
            .run();
        expected_tx_id += 1;
    }

    // Execute all transactions
    for i in (1..=1600usize).rev() {
        test.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(BRIDGE_PROXY_ADDRESS)
            .gas(200_000_000)
            .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
            .execute(i)
            .run();
    }
}

#[test]
fn bridge_proxy_wrong_formatting_sc_call_test() {
    let mut test = BridgeProxyTestState::new();

    test.multisig_deploy();
    test.deploy_bridge_proxy();
    test.deploy_crowdfunding();
    test.config_bridge();

    let eth_tx = EthTransaction {
        from: EthAddress {
            raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
        },
        to: ManagedAddress::from(NO_INIT_SC_ADDRESS.eval_to_array()),
        token_id: BRIDGE_TOKEN_ID.into(),
        amount: BigUint::from(500u64),
        tx_nonce: 1u64,
        call_data: ManagedOption::none(),
    };

    let amount = BigUint::from(500u64);

    test.world
        .tx()
        .from(MULTI_TRANSFER_ADDRESS)
        .to(BRIDGE_PROXY_ADDRESS)
        .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
        .deposit(&eth_tx, 1u64)
        .egld_or_single_esdt(
            &EgldOrEsdtTokenIdentifier::esdt(BRIDGE_TOKEN_ID),
            0,
            &amount,
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

    // Refund: Funds are transfered to BridgedTokensWrapper
    test.world
        .check_account(BRIDGE_PROXY_ADDRESS)
        .esdt_balance(BRIDGE_TOKEN_ID, amount);

    test.world
        .check_account(BRIDGE_PROXY_ADDRESS)
        .check_storage("str:refundTransactions.mapped|u32:1", "0x3031303230333034303530363037303830393130000000000000000005006e6f2d696e69742d73635f5f5f5f5f5f5f5f5f5f5f5f0000000d4252494447452d3132333435360000000201f4000000000000000100")
        .check_storage("str:refundTransactions.value|u32:1", "0x01")
        .check_storage("str:refundTransactions.node_id|u32:1", "0x01")
        .check_storage("str:refundTransactions.info", "0x00000001000000010000000100000001")
        .check_storage("str:refundTransactions.node_links|u32:1", "0x0000000000000000")
        .check_storage("str:batchId|u32:1", "1")
        .check_storage("str:highestTxId", "1")
        .check_storage("str:payments|u32:1", "nested:str:BRIDGE-123456|u64:0|biguint:500");
}

#[test]
fn bridge_proxy_wrong_endpoint_sc_call_test() {
    let mut test = BridgeProxyTestState::new();

    test.multisig_deploy();
    test.deploy_bridge_proxy();
    test.deploy_crowdfunding();
    test.config_bridge();

    let amount = BigUint::from(500u64);

    // Wrong endpoint for callData
    let mut args = ManagedVec::new();
    let call_data: CallData<StaticApi> = CallData {
        endpoint: ManagedBuffer::from(b"nofunc"),
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
        amount: amount.clone(),
        tx_nonce: 2u64,
        call_data: ManagedOption::some(call_data),
    };

    test.world
        .tx()
        .from(MULTI_TRANSFER_ADDRESS)
        .to(BRIDGE_PROXY_ADDRESS)
        .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
        .deposit(&eth_tx, 1u64)
        .egld_or_single_esdt(
            &EgldOrEsdtTokenIdentifier::esdt(BRIDGE_TOKEN_ID),
            0,
            &amount,
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
        .with_gas_limit(GAS_LIMIT * 2)
        .run();

    test.world
        .check_account(BRIDGE_PROXY_ADDRESS)
        .esdt_balance(BRIDGE_TOKEN_ID, amount);

    test.world
    .check_account(BRIDGE_PROXY_ADDRESS)
    .check_storage("str:refundTransactions.mapped|u32:1", "0x30313032303330343035303630373038303931300000000000000000050063726f7766756e64696e675f5f5f5f5f5f5f5f5f5f5f0000000d4252494447452d3132333435360000000201f400000000000000020100000017000000066e6f66756e6300000000009896800100000000")
    .check_storage("str:refundTransactions.value|u32:1", "0x01")
    .check_storage("str:refundTransactions.node_id|u32:1", "0x01")
    .check_storage("str:refundTransactions.info", "0x00000001000000010000000100000001")
    .check_storage("str:refundTransactions.node_links|u32:1", "0x0000000000000000")
    .check_storage("str:batchId|u32:1", "1")
    .check_storage("str:highestTxId", "1")
    .check_storage("str:payments|u32:1", "nested:str:BRIDGE-123456|u64:0|biguint:500");
}

#[test]
fn bridge_proxy_wrong_args_sc_call_test() {
    let mut test = BridgeProxyTestState::new();

    test.multisig_deploy();
    test.deploy_bridge_proxy();
    test.deploy_crowdfunding();
    test.config_bridge();

    let amount = BigUint::from(500u64);

    // Wrong args
    let mut args = ManagedVec::new();
    args.push(ManagedBuffer::from(b"wrongargs"));

    let call_data: CallData<StaticApi> = CallData {
        endpoint: ManagedBuffer::from(b"fund"),
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
        amount: amount.clone(),
        tx_nonce: 3u64,
        call_data: ManagedOption::some(call_data),
    };

    test.world
        .tx()
        .from(MULTI_TRANSFER_ADDRESS)
        .to(BRIDGE_PROXY_ADDRESS)
        .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
        .deposit(&eth_tx, 1u64)
        .egld_or_single_esdt(
            &EgldOrEsdtTokenIdentifier::esdt(BRIDGE_TOKEN_ID),
            0,
            &amount,
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
        .with_gas_limit(GAS_LIMIT * 2)
        .run();

    // Refund: Funds are transfered to BridgedTokensWrapper
    test.world
        .check_account(BRIDGE_PROXY_ADDRESS)
        .esdt_balance(BRIDGE_TOKEN_ID, amount);

    test.world
        .check_account(BRIDGE_PROXY_ADDRESS)
        .check_storage("str:refundTransactions.mapped|u32:1", "0x30313032303330343035303630373038303931300000000000000000050063726f7766756e64696e675f5f5f5f5f5f5f5f5f5f5f0000000d4252494447452d3132333435360000000201f4000000000000000301000000220000000466756e64000000000098968001000000010000000977726f6e6761726773")
        .check_storage("str:refundTransactions.value|u32:1", "0x01")
        .check_storage("str:refundTransactions.node_id|u32:1", "0x01")
        .check_storage("str:refundTransactions.info", "0x00000001000000010000000100000001")
        .check_storage("str:refundTransactions.node_links|u32:1", "0x0000000000000000")
        .check_storage("str:batchId|u32:1", "1")
        .check_storage("str:highestTxId", "1")
        .check_storage("str:payments|u32:1", "nested:str:BRIDGE-123456|u64:0|biguint:500");
}

#[test]
fn bridge_proxy_too_small_gas_sc_call_test() {
    let mut test = BridgeProxyTestState::new();

    test.world.start_trace();

    test.multisig_deploy();
    test.deploy_bridge_proxy();
    test.deploy_crowdfunding();
    test.config_bridge();

    let mut args = ManagedVec::new();
    let call_data: CallData<StaticApi> = CallData {
        endpoint: ManagedBuffer::from(b"fund"),
        gas_limit: TOO_SMALL_GAS_LIMIT,
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

    let amount = BigUint::from(500u64);

    test.world
        .tx()
        .from(MULTI_TRANSFER_ADDRESS)
        .to(BRIDGE_PROXY_ADDRESS)
        .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
        .deposit(&eth_tx, 1u64)
        .egld_or_single_esdt(
            &EgldOrEsdtTokenIdentifier::esdt(BRIDGE_TOKEN_ID),
            0,
            &amount,
        )
        .run();

    test.world
        .query()
        .to(BRIDGE_PROXY_ADDRESS)
        .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
        .get_pending_transaction_by_id(1u32)
        .returns(ExpectValue(eth_tx.clone()))
        .run();

    test.world
        .tx()
        .from(OWNER_ADDRESS)
        .to(BRIDGE_PROXY_ADDRESS)
        .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
        .execute(1u32)
        .run();

    test.world
        .check_account(BRIDGE_PROXY_ADDRESS)
        .check_storage("str:refundTransactions.mapped|u32:1", "0x30313032303330343035303630373038303931300000000000000000050063726f7766756e64696e675f5f5f5f5f5f5f5f5f5f5f0000000d4252494447452d3132333435360000000201f4000000000000000101000000150000000466756e6400000000000f42400100000000")
        .check_storage("str:refundTransactions.value|u32:1", "0x01")
        .check_storage("str:refundTransactions.node_id|u32:1", "0x01")
        .check_storage("str:refundTransactions.info", "0x00000001000000010000000100000001")
        .check_storage("str:refundTransactions.node_links|u32:1", "0x0000000000000000")
        .check_storage("str:batchId|u32:1", "1")
        .check_storage("str:highestTxId", "1")
        .check_storage("str:payments|u32:1", "nested:str:BRIDGE-123456|u64:0|biguint:500");
}

#[test]
fn bridge_proxy_empty_endpoint_with_args_test() {
    let mut test = BridgeProxyTestState::new();

    test.world.start_trace();

    test.multisig_deploy();
    test.deploy_bridge_proxy();
    test.deploy_crowdfunding();
    test.config_bridge();

    let mut args = ManagedVec::new();
    let call_data: CallData<StaticApi> = CallData {
        endpoint: ManagedBuffer::new(),
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

    let amount = BigUint::from(500u64);

    test.world
        .tx()
        .from(MULTI_TRANSFER_ADDRESS)
        .to(BRIDGE_PROXY_ADDRESS)
        .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
        .deposit(&eth_tx, 1u64)
        .egld_or_single_esdt(
            &EgldOrEsdtTokenIdentifier::esdt(BRIDGE_TOKEN_ID),
            0,
            &amount,
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
        .check_account(BRIDGE_PROXY_ADDRESS)
        .check_storage("str:refundTransactions.mapped|u32:1", "0x30313032303330343035303630373038303931300000000000000000050063726f7766756e64696e675f5f5f5f5f5f5f5f5f5f5f0000000d4252494447452d3132333435360000000201f4000000000000000101000000110000000000000000009896800100000000")
        .check_storage("str:refundTransactions.value|u32:1", "0x01")
        .check_storage("str:refundTransactions.node_id|u32:1", "0x01")
        .check_storage("str:refundTransactions.info", "0x00000001000000010000000100000001")
        .check_storage("str:refundTransactions.node_links|u32:1", "0x0000000000000000")
        .check_storage("str:batchId|u32:1", "1")
        .check_storage("str:highestTxId", "1")
        .check_storage(
            "str:payments|u32:1",
            "nested:str:BRIDGE-123456|u64:0|biguint:500",
        );
}

#[test]
fn bridge_proxy_empty_endpoint_with_gas_test() {
    let mut test = BridgeProxyTestState::new();

    test.world.start_trace();

    test.multisig_deploy();
    test.deploy_bridge_proxy();
    test.deploy_crowdfunding();
    test.config_bridge();

    let call_data: CallData<StaticApi> = CallData {
        endpoint: ManagedBuffer::new(),
        gas_limit: GAS_LIMIT,
        args: ManagedOption::none(),
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

    let amount = BigUint::from(500u64);

    test.world
        .tx()
        .from(MULTI_TRANSFER_ADDRESS)
        .to(BRIDGE_PROXY_ADDRESS)
        .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
        .deposit(&eth_tx, 1u64)
        .egld_or_single_esdt(
            &EgldOrEsdtTokenIdentifier::esdt(BRIDGE_TOKEN_ID),
            0,
            &amount,
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
        .check_account(BRIDGE_PROXY_ADDRESS)
        .check_storage("str:refundTransactions.mapped|u32:1", "0x30313032303330343035303630373038303931300000000000000000050063726f7766756e64696e675f5f5f5f5f5f5f5f5f5f5f0000000d4252494447452d3132333435360000000201f40000000000000001010000000d00000000000000000098968000")
        .check_storage("str:refundTransactions.value|u32:1", "0x01")
        .check_storage("str:refundTransactions.node_id|u32:1", "0x01")
        .check_storage("str:refundTransactions.info", "0x00000001000000010000000100000001")
        .check_storage("str:refundTransactions.node_links|u32:1", "0x0000000000000000")
        .check_storage("str:batchId|u32:1", "1")
        .check_storage("str:highestTxId", "1")
        .check_storage(
            "str:payments|u32:1",
            "nested:str:BRIDGE-123456|u64:0|biguint:500",
        );
}

#[test]
fn bridge_proxy_refund_tx_test() {
    let mut test = BridgeProxyTestState::new();

    test.world.start_trace();

    test.multisig_deploy();
    test.deploy_bridge_proxy();
    test.deploy_crowdfunding();
    test.config_bridge();

    let mut args = ManagedVec::new();
    let call_data: CallData<StaticApi> = CallData {
        endpoint: ManagedBuffer::new(),
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

    let amount = BigUint::from(500u64);

    test.world
        .tx()
        .from(MULTI_TRANSFER_ADDRESS)
        .to(BRIDGE_PROXY_ADDRESS)
        .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
        .deposit(&eth_tx, 1u64)
        .egld_or_single_esdt(
            &EgldOrEsdtTokenIdentifier::esdt(BRIDGE_TOKEN_ID),
            0,
            &amount,
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
        .from(USER_ADDRESS)
        .to(BRIDGE_PROXY_ADDRESS)
        .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
        .execute(1u32)
        .run();

    test.world
        .check_account(BRIDGE_PROXY_ADDRESS)
        .check_storage("str:refundTransactions.mapped|u32:1", "0x30313032303330343035303630373038303931300000000000000000050063726f7766756e64696e675f5f5f5f5f5f5f5f5f5f5f0000000d4252494447452d3132333435360000000201f4000000000000000101000000110000000000000000009896800100000000")
        .check_storage("str:refundTransactions.value|u32:1", "0x01")
        .check_storage("str:refundTransactions.node_id|u32:1", "0x01")
        .check_storage("str:refundTransactions.info", "0x00000001000000010000000100000001")
        .check_storage("str:refundTransactions.node_links|u32:1", "0x0000000000000000")
        .check_storage("str:batchId|u32:1", "1")
        .check_storage("str:highestTxId", "1")
        .check_storage(
            "str:payments|u32:1",
            "nested:str:BRIDGE-123456|u64:0|biguint:500",
        );

    test.world
        .check_account(BRIDGE_PROXY_ADDRESS)
        .esdt_balance(BRIDGE_TOKEN_ID, amount.clone());

    test.world
        .check_account(USER_ADDRESS)
        .esdt_balance(BRIDGE_TOKEN_ID, BigUint::from(INITIAL_BALANCE));

    test.world
        .tx()
        .from(USER_ADDRESS)
        .to(BRIDGE_PROXY_ADDRESS)
        .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
        .execute_refund_transaction(1u32)
        .run();

    test.world
        .check_account(BRIDGE_PROXY_ADDRESS)
        .esdt_balance(BRIDGE_TOKEN_ID, BigUint::zero());

    test.world
        .check_account(BRIDGE_PROXY_ADDRESS)
        .check_storage("str:highestTxId", "1");
}
