#![allow(unused)]

use std::collections::LinkedList;

use bridge_proxy::config::ProxyTrait as _;
use bridge_proxy::ProxyTrait;

use multiversx_sc::{
    api::ManagedTypeApi,
    codec::multi_types::{MultiValueVec, OptionalValue},
    storage::mappers::SingleValue,
    types::{
        Address, BigUint, CodeMetadata, ManagedAddress, ManagedBuffer, ManagedByteArray,
        TokenIdentifier,
    },
};
use multiversx_sc_scenario::{
    api::StaticApi,
    scenario_format::interpret_trait::{InterpretableFrom, InterpreterContext},
    scenario_model::*,
    ContractInfo, ScenarioWorld,
};

use eth_address::*;
use transaction::{EthTransaction, EthTransactionPayment};

const BRIDGE_TOKEN_ID: &[u8] = b"BRIDGE-123456";
const MULTI_TRANSFER_CONTRACT_ADDRESS: &str =
    "0x0000000000000000000000000000000000000000000000000000000000000000";

const USER_ETHEREUM_ADDRESS: &[u8] = b"0x0102030405060708091011121314151617181920";

const GAS_LIMIT: u64 = 1_000_000;

const BRIDGE_PROXY_PATH_EXPR: &str = "file:output/bridge-proxy.wasm";
// const MULTI_TRANSFER_PATH_EXPR: &str = "file:../multi-transfer-esdt/output/multi-transfer-esdt.wasm";
// const ADDER_PATH_EXPR: &str = "file:test-contracts/adder.wasm";

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(BRIDGE_PROXY_PATH_EXPR, bridge_proxy::ContractBuilder);
    blockchain
}

#[test]
fn basic_setup_test() {
    let mut test = BridgeProxyTestState::setup();
    let bridge_token_id_expr = "str:BRIDGE-123456"; // when specifying the token transfer

    test.bridge_proxy_deploy();

    let eth_tx = EthTransaction {
        from: test.eth_user,
        to: ManagedAddress::from_address(&test.user.value),
        token_id: TokenIdentifier::from_esdt_bytes(BRIDGE_TOKEN_ID),
        amount: BigUint::from(500u64),
        tx_nonce: 1u64,
        data: ManagedBuffer::from("data"),
        gas_limit: GAS_LIMIT,
    };

    test.world.set_state_step(SetStateStep::new().put_account(
        &test.owner,
        Account::new().esdt_balance(bridge_token_id_expr, 1_000u64),
    ));

    test.world.sc_call(
        ScCallStep::new()
            .from(&test.owner)
            .to(&test.bridge_proxy)
            .call(test.bridge_proxy.deposit(&eth_tx))
            .esdt_transfer(bridge_token_id_expr, 0u64, 500u64),
    );

    test.world.sc_query(
        ScQueryStep::new()
            .to(&test.bridge_proxy)
            .call(test.bridge_proxy.get_eth_transaction_by_id(1u32))
            .expect_value(eth_tx.data));
        // |tr| {
        //     let respose: LinkedList<EthTransactionPayment<StaticApi>> = tr.result.unwrap();
        //     let reponse_eth_tx = respose.pop_front();

        //     let eth_tx_payment = EthTransactionPayment {
        //         token_id: TokenIdentifier::from_esdt_bytes(BRIDGE_TOKEN_ID),
        //         nonce: 0u64,
        //         amount: BigUint::from(500u64),
        //         eth_tx,
        //     };
        //     match reponse_eth_tx {
        //         Some(tx) => assert!(tx.eq(&eth_tx_payment), "Transactions not equal!"),
        //         None => panic!("No transaction registered!"),
        //     }
        // },
    // );
}

type BridgeProxyContract = ContractInfo<bridge_proxy::Proxy<StaticApi>>;

struct BridgeProxyTestState<M: ManagedTypeApi> {
    world: ScenarioWorld,
    owner: AddressValue,
    user: AddressValue,
    eth_user: EthAddress<M>,
    bridge_proxy: BridgeProxyContract,
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
            bridge_proxy: BridgeProxyContract::new("sc:bridge_proxy"),
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
                .new_address(&self.owner, 1, &self.bridge_proxy),
        );

        let ic = &self.world.interpreter_context();
        self.world.sc_deploy(
            ScDeployStep::new()
                .from(self.owner.clone())
                .code(self.world.code_expression(BRIDGE_PROXY_PATH_EXPR))
                .call(self.bridge_proxy.init(ManagedAddress::zero())),
        );

        self
    }
}
