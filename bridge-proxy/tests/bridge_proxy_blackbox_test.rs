#![allow(unused)]

use std::collections::LinkedList;

use adder::{adder_proxy, Adder, ProxyTrait as _};
use bridge_proxy::ProxyTrait;
use bridge_proxy::{bridge_proxy_contract_proxy, config::ProxyTrait as _};

use multiversx_sc::codec::NestedEncode;
use multiversx_sc::contract_base::ManagedSerializer;
use multiversx_sc::sc_print;
use multiversx_sc::types::{
    EgldOrEsdtTokenIdentifier, EsdtTokenPayment, ReturnsNewAddress, TestAddress, TestSCAddress,
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

const BRIDGE_TOKEN_ID: &[u8] = b"BRIDGE-123456";
const GAS_LIMIT: u64 = 10_000_000;

const OWNER_ADDRESS: TestAddress = TestAddress::new("owner");
const BRIDGE_PROXY_ADDRESS: TestSCAddress = TestSCAddress::new("bridge-proxy");
const ADDER_ADDRESS: TestSCAddress = TestSCAddress::new("adder");
const MULTI_TRANSFER_ADDRESS: TestSCAddress = TestSCAddress::new("multi-transfer");
const ESDT_SAFE_ADDRESS: TestSCAddress = TestSCAddress::new("esdt-safe");

const BRIDGE_PROXY_PATH_EXPR: MxscPath = MxscPath::new("output/bridge-proxy.mxsc.json");
const ADDER_PATH_EXPR: MxscPath = MxscPath::new("tests/test-contract/adder.mxsc.json");
const MULTI_TRANSFER_PATH_EXPR: &str =
    "mxsc:../multi-transfer-esdt/output/multi-transfer-esdt.mxsc.json";
const ESDT_SAFE_PATH_EXPR: &str = "mxsc:../esdt-safe/output/esdt-safe.mxsc.json";

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(BRIDGE_PROXY_PATH_EXPR, bridge_proxy::ContractBuilder);
    blockchain.register_contract(ADDER_PATH_EXPR, adder::ContractBuilder);

    blockchain
}

type BridgeProxyContract = ContractInfo<bridge_proxy::Proxy<StaticApi>>;
type AdderContract = ContractInfo<adder::Proxy<StaticApi>>;

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
            .esdt_balance(TokenIdentifier::from(BRIDGE_TOKEN_ID), 1_000u64)
            .account(MULTI_TRANSFER_ADDRESS)
            .esdt_balance(TokenIdentifier::from(BRIDGE_TOKEN_ID), 1_000u64)
            .code(multi_transfer_code)
            .account(ESDT_SAFE_ADDRESS)
            .code(esdt_safe_code);

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

    fn deploy_adder(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(adder_proxy::AdderProxy)
            .init(BigUint::zero())
            .code(ADDER_PATH_EXPR)
            .new_address(ADDER_ADDRESS)
            .run();
        self
    }

    fn config_bridge(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(BRIDGE_PROXY_ADDRESS)
            .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
            .set_esdt_safe_contract_address(OptionalValue::Some(ESDT_SAFE_ADDRESS))
            .run();

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(BRIDGE_PROXY_ADDRESS)
            .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
            .unpause_endpoint()
            .run();

        self
    }
}

#[test]
fn deploy_test() {
    let mut test = BridgeProxyTestState::new();

    test.bridge_proxy_deploy();
    test.deploy_adder();
    test.config_bridge();
}

#[test]
fn deploy_deposit_test() {
    let mut test = BridgeProxyTestState::new();

    test.bridge_proxy_deploy();
    test.deploy_adder();
    test.config_bridge();

    let mut args = ManagedVec::new();
    args.push(ManagedBuffer::from(&[5u8]));

    let call_data: CallData<StaticApi> = CallData {
        endpoint: ManagedBuffer::from("add"),
        gas_limit: GAS_LIMIT,
        args,
    };

    let call_data: ManagedBuffer<StaticApi> =
        ManagedSerializer::new().top_encode_to_managed_buffer(&call_data);

    let eth_tx = EthTransaction {
        from: EthAddress {
            raw_addr: ManagedByteArray::default(),
        },
        to: ManagedAddress::from(ADDER_ADDRESS.eval_to_array()),
        token_id: TokenIdentifier::from_esdt_bytes(BRIDGE_TOKEN_ID),
        amount: BigUint::from(0u64),
        tx_nonce: 1u64,
        call_data,
    };
    // let mut buf: ManagedBuffer<StaticApi> = ManagedBuffer::new();
    // eth_tx.dep_encode(&mut buf);

    // println!("{:?}", buf);

    test.world
        .tx()
        .from(MULTI_TRANSFER_ADDRESS)
        .to(BRIDGE_PROXY_ADDRESS)
        .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
        .deposit(&eth_tx)
        .egld_or_single_esdt(
            &EgldOrEsdtTokenIdentifier::esdt(BRIDGE_TOKEN_ID),
            0,
            &BigUint::from(1u64),
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
        .check_account(ADDER_ADDRESS)
        .check_storage("str:sum", "5");
}

// #[test]
fn multiple_deposit_test() {
    let mut test = BridgeProxyTestState::new();

    test.bridge_proxy_deploy();
    test.deploy_adder();
    test.config_bridge();

    let mut args = ManagedVec::new();
    args.push(ManagedBuffer::from(&[5u8]));

    let call_data: CallData<StaticApi> = CallData {
        endpoint: ManagedBuffer::from(b"add"),
        gas_limit: GAS_LIMIT,
        args,
    };
    let call_data = ManagedSerializer::new().top_encode_to_managed_buffer(&call_data);

    let eth_tx1 = EthTransaction {
        from: EthAddress {
            raw_addr: ManagedByteArray::default(),
        },
        to: ManagedAddress::from(ADDER_ADDRESS.eval_to_array()),
        token_id: TokenIdentifier::from_esdt_bytes(BRIDGE_TOKEN_ID),
        amount: BigUint::from(500u64),
        tx_nonce: 1u64,
        call_data,
    };

    let mut args = ManagedVec::new();
    args.push(ManagedBuffer::from(&[15u8]));

    let call_data: CallData<StaticApi> = CallData {
        endpoint: ManagedBuffer::from(b"add"),
        gas_limit: GAS_LIMIT,
        args,
    };
    let call_data = ManagedSerializer::new().top_encode_to_managed_buffer(&call_data);

    let eth_tx2 = EthTransaction {
        from: EthAddress {
            raw_addr: ManagedByteArray::default(),
        },
        to: ManagedAddress::from(ADDER_ADDRESS.eval_to_array()),
        token_id: TokenIdentifier::from_esdt_bytes(BRIDGE_TOKEN_ID),
        amount: BigUint::zero(),
        tx_nonce: 1u64,
        call_data,
    };

    test.world
        .tx()
        .from(OWNER_ADDRESS)
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
        .from(OWNER_ADDRESS)
        .to(BRIDGE_PROXY_ADDRESS)
        .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
        .deposit(&eth_tx2)
        .single_esdt(
            &TokenIdentifier::from(BRIDGE_TOKEN_ID),
            0u64,
            &BigUint::zero(),
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
        .check_account(ADDER_ADDRESS)
        .check_storage("str:sum", "5");

    test.world
        .tx()
        .from(OWNER_ADDRESS)
        .to(BRIDGE_PROXY_ADDRESS)
        .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
        .execute(2u32)
        .run();

    test.world
        .query()
        .to(ADDER_ADDRESS)
        .typed(adder_proxy::AdderProxy)
        .sum()
        .returns(ExpectValue(BigUint::from(20u32)))
        .run();

    test.world
        .check_account(ADDER_ADDRESS)
        .check_storage("str:sum", "20");
}
