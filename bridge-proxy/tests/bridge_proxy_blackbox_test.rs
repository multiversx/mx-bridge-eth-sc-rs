#![allow(unused)]

use std::collections::LinkedList;

use adder::{Adder, ProxyTrait as _};
use bridge_proxy::config::ProxyTrait as _;
use bridge_proxy::ProxyTrait;

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
use multiversx_sc_scenario::{
    api::StaticApi,
    rust_biguint,
    scenario_format::interpret_trait::{InterpretableFrom, InterpreterContext},
    scenario_model::*,
    ContractInfo, ScenarioWorld,
};

use eth_address::*;
use transaction::{call_data::CallData, EthTransaction, EthTransactionPayment};

const BRIDGE_TOKEN_ID: &[u8] = b"BRIDGE-123456";
const GAS_LIMIT: u64 = 1_000_000;
const BRIDGE_PROXY_PATH_EXPR: &str = "file:output/bridge-proxy.wasm";
const ADDER_BOGUS_PATH_EXPR: &str = "file:bogus-path.wasm";

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(BRIDGE_PROXY_PATH_EXPR, bridge_proxy::ContractBuilder);
    blockchain.register_contract(ADDER_BOGUS_PATH_EXPR, adder::ContractBuilder);

    blockchain
}

type BridgeProxyContract = ContractInfo<bridge_proxy::Proxy<StaticApi>>;
type AdderContract = ContractInfo<adder::Proxy<StaticApi>>;

struct BridgeProxyTestState<M: ManagedTypeApi> {
    world: ScenarioWorld,
    owner: AddressValue,
    user: AddressValue,
    eth_user: EthAddress<M>,
    bridge_proxy_contract: BridgeProxyContract,
    adder_contract: AdderContract,
}

impl<M: ManagedTypeApi> BridgeProxyTestState<M> {
    fn setup() -> Self {
        let world = world();
        let ic = &world.interpreter_context();

        let mut state = BridgeProxyTestState {
            world,
            owner: "address:owner".into(),
            user: "address:user".into(),
            eth_user: EthAddress {
                raw_addr: ManagedByteArray::default(),
            },
            bridge_proxy_contract: BridgeProxyContract::new("sc:bridge_proxy"),
            adder_contract: AdderContract::new("sc:adder"),
        };

        state
            .world
            .set_state_step(SetStateStep::new().put_account(&state.owner, Account::new().nonce(1)));

        state
    }

    fn bridge_proxy_deploy(&mut self) -> &mut Self {
        self.world.set_state_step(
            SetStateStep::new()
                .put_account(&self.owner, Account::new().nonce(1))
                .new_address(&self.owner, 1, &self.bridge_proxy_contract),
        );

        let ic = &self.world.interpreter_context();
        self.world.sc_deploy(
            ScDeployStep::new()
                .from(self.owner.clone())
                .code(self.world.code_expression(BRIDGE_PROXY_PATH_EXPR))
                .call(self.bridge_proxy_contract.init(ManagedAddress::zero())),
        );

        self
    }

    fn deploy_adder(&mut self) -> &mut Self {
        self.world.set_state_step(SetStateStep::new().new_address(
            &self.owner,
            2,
            &self.adder_contract,
        ));

        self.world.sc_deploy(
            ScDeployStep::new()
                .from(self.owner.clone())
                .code(self.world.code_expression(ADDER_BOGUS_PATH_EXPR))
                .call(self.adder_contract.init(BigUint::zero())),
        );

        self
    }
}

#[test]
fn deploy_deposit_test() {
    let mut test = BridgeProxyTestState::setup();
    let bridge_token_id_expr = "str:BRIDGE-123456"; // when specifying the token transfer

    test.bridge_proxy_deploy();
    test.deploy_adder();

    let mut args = ManagedVec::new();
    args.push(ManagedBuffer::from(&[5u8]));

    let eth_tx = EthTransaction {
        from: test.eth_user.clone(),
        to: ManagedAddress::from_address(&test.adder_contract.to_address()),
        token_id: TokenIdentifier::from_esdt_bytes(BRIDGE_TOKEN_ID),
        amount: BigUint::from(500u64),
        tx_nonce: 1u64,
        call_data: Some(CallData {
            endpoint: ManagedBuffer::from(b"add"),
            gas_limit: GAS_LIMIT,
            args,
        }),
    };

    test.world.set_state_step(SetStateStep::new().put_account(
        &test.owner,
        Account::new().esdt_balance(bridge_token_id_expr, 1_000u64),
    ));

    test.world.sc_call(
        ScCallStep::new()
            .from(&test.owner)
            .to(&test.bridge_proxy_contract)
            .call(test.bridge_proxy_contract.deposit(&eth_tx))
            .esdt_transfer(bridge_token_id_expr, 0u64, 500u64),
    );

    test.world.sc_query(
        ScQueryStep::new()
            .to(&test.bridge_proxy_contract)
            .call(test.bridge_proxy_contract.get_eth_transaction_by_id(1u32))
            .expect_value(eth_tx),
    );

    test.world.sc_call(
        ScCallStep::new()
            .from(&test.owner)
            .to(&test.bridge_proxy_contract)
            .call(test.bridge_proxy_contract.execute_with_async(1u32)),
    );

    test.world.sc_query(
        ScQueryStep::new()
            .to(&test.adder_contract)
            .call(test.adder_contract.sum())
            .expect_value(SingleValue::from(BigUint::from(5u32))),
    );
}

#[test]
fn multiple_deposit_test() {
    let mut test = BridgeProxyTestState::setup();
    let bridge_token_id_expr = "str:BRIDGE-123456"; // when specifying the token transfer

    test.bridge_proxy_deploy();
    test.deploy_adder();

    let mut args1 = ManagedVec::new();
    args1.push(ManagedBuffer::from(&[5u8]));

    let eth_tx1 = EthTransaction {
        from: test.eth_user.clone(),
        to: ManagedAddress::from_address(&test.adder_contract.to_address()),
        token_id: TokenIdentifier::from_esdt_bytes(BRIDGE_TOKEN_ID),
        amount: BigUint::from(500u64),
        tx_nonce: 1u64,
        call_data: Some(CallData {
            endpoint: ManagedBuffer::from(b"add"),
            gas_limit: GAS_LIMIT,
            args: args1,
        }),
    };

    let mut args2 = ManagedVec::new();
    args2.push(ManagedBuffer::from(&[15u8]));

    let eth_tx2 = EthTransaction {
        from: test.eth_user.clone(),
        to: ManagedAddress::from_address(&test.adder_contract.to_address()),
        token_id: TokenIdentifier::from_esdt_bytes(BRIDGE_TOKEN_ID),
        amount: BigUint::zero(),
        tx_nonce: 1u64,
        call_data: Some(CallData {
            endpoint: ManagedBuffer::from(b"add"),
            gas_limit: GAS_LIMIT,
            args: args2,
        }),
    };

    test.world.set_state_step(SetStateStep::new().put_account(
        &test.owner,
        Account::new().esdt_balance(bridge_token_id_expr, 1_000u64),
    ));

    test.world.sc_call(
        ScCallStep::new()
            .from(&test.owner)
            .to(&test.bridge_proxy_contract)
            .call(test.bridge_proxy_contract.deposit(&eth_tx1))
            .esdt_transfer(bridge_token_id_expr, 0u64, 500u64),
    );

    test.world.sc_call(
        ScCallStep::new()
            .from(&test.owner)
            .to(&test.bridge_proxy_contract)
            .call(test.bridge_proxy_contract.deposit(&eth_tx2))
            .esdt_transfer(bridge_token_id_expr, 0u64, 0u64),
    );

    test.world.sc_query(
        ScQueryStep::new()
            .to(&test.bridge_proxy_contract)
            .call(test.bridge_proxy_contract.get_eth_transaction_by_id(1u32))
            .expect_value(eth_tx1),
    );

    test.world.sc_query(
        ScQueryStep::new()
            .to(&test.bridge_proxy_contract)
            .call(test.bridge_proxy_contract.get_eth_transaction_by_id(2u32))
            .expect_value(eth_tx2),
    );

    test.world.sc_call(
        ScCallStep::new()
            .from(&test.owner)
            .to(&test.bridge_proxy_contract)
            .call(test.bridge_proxy_contract.execute_with_async(1u32)),
    );

    test.world.sc_query(
        ScQueryStep::new()
            .to(&test.adder_contract)
            .call(test.adder_contract.sum())
            .expect_value(SingleValue::from(BigUint::from(5u32))),
    );

    test.world.sc_call(
        ScCallStep::new()
            .from(&test.owner)
            .to(&test.bridge_proxy_contract)
            .call(test.bridge_proxy_contract.execute_with_async(2u32)),
    );

    test.world.sc_query(
        ScQueryStep::new()
            .to(&test.adder_contract)
            .call(test.adder_contract.sum())
            .expect_value(SingleValue::from(BigUint::from(20u32))),
    );
}
