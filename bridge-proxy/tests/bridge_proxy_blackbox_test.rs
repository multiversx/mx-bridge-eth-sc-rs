#![allow(unused)]

use std::collections::LinkedList;

use adder::{adder_proxy, Adder, ProxyTrait as _};
use bridge_proxy::ProxyTrait;
use bridge_proxy::{bridge_proxy_contract_proxy, config::ProxyTrait as _};

use multiversx_sc::types::{ReturnsNewAddress, TestAddress};
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
use transaction::EthTransaction;

const BRIDGE_TOKEN_ID: &[u8] = b"BRIDGE-123456";
const GAS_LIMIT: u64 = 1_000_000;

const OWNER_ADDRESS: TestAddress = TestAddress::new("owner");
const BRIDGE_PROXY_ADDRESS: TestAddress = TestAddress::new("bridge-proxy");
const ADDER_ADDRESS: TestAddress = TestAddress::new("adder");

const BRIDGE_PROXY_PATH_EXPR: MxscPath = MxscPath::new("output/bridge-proxy.mxsc.json");
const ADDER_PATH_EXPR: MxscPath = MxscPath::new("test-contract/adder.mxsc.json");

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

        world.account(OWNER_ADDRESS).nonce(1);
        Self { world }
    }

    fn bridge_proxy_deploy(&mut self) -> &mut Self {
        self.world.account(OWNER_ADDRESS).nonce(1);

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
            .init(OptionalValue::Some(ManagedAddress::default()))
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
}

// #[test]
fn deploy_deposit_test() {
    let mut test = BridgeProxyTestState::new();
    let bridge_token_id_expr: &str = "str:BRIDGE-123456"; // when specifying the token transfer

    test.bridge_proxy_deploy();
    test.deploy_adder();

    let mut args = ManagedVec::new();
    args.push(ManagedBuffer::from(&[5u8]));

    let eth_tx = EthTransaction {
        from: EthAddress {
            raw_addr: ManagedByteArray::default(),
        },
        to: ManagedAddress::from(ADDER_ADDRESS.eval_to_array()),
        token_id: TokenIdentifier::from_esdt_bytes(BRIDGE_TOKEN_ID),
        amount: BigUint::from(500u64),
        tx_nonce: 1u64,
        call_endpoint: ManagedBuffer::from(b"add"),
        call_gas_limit: GAS_LIMIT,
        call_args: args,
    };
    test.world
        .account(OWNER_ADDRESS)
        .esdt_balance(TokenIdentifier::from(BRIDGE_TOKEN_ID), 1_000u64);

    test.world
        .tx()
        .from(OWNER_ADDRESS)
        .to(BRIDGE_PROXY_ADDRESS)
        .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
        .deposit(&eth_tx)
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
        .to(ADDER_ADDRESS)
        .typed(adder_proxy::AdderProxy)
        .sum()
        .returns(ExpectValue(BigUint::from(5u32)))
        .run();
}

// #[test]
fn multiple_deposit_test() {
    let mut test = BridgeProxyTestState::new();
    let bridge_token_id_expr = "str:BRIDGE-123456"; // when specifying the token transfer

    test.bridge_proxy_deploy();
    test.deploy_adder();

    let mut args1 = ManagedVec::new();
    args1.push(ManagedBuffer::from(&[5u8]));

    let eth_tx1 = EthTransaction {
        from: EthAddress {
            raw_addr: ManagedByteArray::default(),
        },
        to: ManagedAddress::from(ADDER_ADDRESS.eval_to_array()),
        token_id: TokenIdentifier::from_esdt_bytes(BRIDGE_TOKEN_ID),
        amount: BigUint::from(500u64),
        tx_nonce: 1u64,
        call_endpoint: ManagedBuffer::from(b"add"),
        call_gas_limit: GAS_LIMIT,
        call_args: args1,
    };

    let mut args2 = ManagedVec::new();
    args2.push(ManagedBuffer::from(&[15u8]));

    let eth_tx2 = EthTransaction {
        from: EthAddress {
            raw_addr: ManagedByteArray::default(),
        },
        to: ManagedAddress::from(ADDER_ADDRESS.eval_to_array()),
        token_id: TokenIdentifier::from_esdt_bytes(BRIDGE_TOKEN_ID),
        amount: BigUint::zero(),
        tx_nonce: 1u64,
        call_endpoint: ManagedBuffer::from(b"add"),
        call_gas_limit: GAS_LIMIT,
        call_args: args2,
    };

    test.world
        .account(OWNER_ADDRESS)
        .esdt_balance(TokenIdentifier::from(BRIDGE_TOKEN_ID), 1_000u64);

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
        .query()
        .to(ADDER_ADDRESS)
        .typed(adder_proxy::AdderProxy)
        .sum()
        .returns(ExpectValue(BigUint::from(5u32)))
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
        .to(ADDER_ADDRESS)
        .typed(adder_proxy::AdderProxy)
        .sum()
        .returns(ExpectValue(BigUint::from(20u32)))
        .run();
}
