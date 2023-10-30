#![allow(unused)]

use bridge_proxy::ProxyTrait as _;
use esdt_safe::{EsdtSafe, ProxyTrait};
use multi_transfer_esdt::ProxyTrait as _;

use multiversx_sc::{
    api::ManagedTypeApi,
    codec::multi_types::{MultiValueVec, OptionalValue},
    storage::mappers::SingleValue,
    types::{
        Address, BigUint, CodeMetadata, ManagedAddress, ManagedBuffer, ManagedByteArray,
        MultiValueEncoded, TokenIdentifier,
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
const USER_ETHEREUM_ADDRESS: &[u8] = b"0x0102030405060708091011121314151617181920";

const GAS_LIMIT: u64 = 1_000_000;

const MULTI_TRANSFER_PATH_EXPR: &str = "file:output/multi-transfer-esdt.wasm";
const BRIDGE_PROXY_PATH_EXPR: &str = "file:../bridge-proxy/output/bridge-proxy.wasm";
const ESDT_SAFE_PATH_EXPR: &str = "file:../esdt-safe/output/esdt-safe.wasm";
const BRIDGED_TOKENS_WRAPPER_PATH_EXPR: &str =
    "file:../bridged-tokens-wrapper/output/bridged-tokens-wrapper.wasm";
const PRICE_AGGREGATOR_PATH_EXPR: &str = "file:../price-aggregator/price-aggregator.wasm";

const MULTI_TRANSFER_ADDRESS_EXPR: &str = "sc:multi_transfer";
const BRIDGE_PROXY_ADDRESS_EXPR: &str = "sc_bridge_proxy";
const ESDT_SAFE_ADDRESS_EXPR: &str = "sc:esdt_safe";
const BRIDGED_TOKENS_WRAPPER_ADDRESS_EXPR: &str = "sc:bridged_tokens_wrapper";
const PRICE_AGGREGATOR_ADDRESS_EXPR: &str = "sc:price_aggregator";

const ORACLE_ADDRESS_EXPR: &str = "address:oracle";

const ESDT_SAFE_ETH_TX_GAS_LIMIT: u64 = 150_000;

const BALANCE: &str = "2,000,000";
const WEGLD_TOKEN_ID_EXPR: &str = "str:BRIDGE-123456";

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(
        MULTI_TRANSFER_PATH_EXPR,
        multi_transfer_esdt::ContractBuilder,
    );
    blockchain
}

type MultiTransferContract = ContractInfo<multi_transfer_esdt::Proxy<StaticApi>>;
type BridgeProxyContract = ContractInfo<bridge_proxy::Proxy<StaticApi>>;
type EsdtSafeContract = ContractInfo<esdt_safe::Proxy<StaticApi>>;
type BridgedTokensWrapperContract = ContractInfo<bridged_tokens_wrapper::Proxy<StaticApi>>;
type PriceAggregatorContract = ContractInfo<price_aggregator::Proxy<StaticApi>>;

struct MultiTransferTestState<M: ManagedTypeApi> {
    world: ScenarioWorld,
    owner: AddressValue,
    user1: AddressValue,
    user2: AddressValue,
    eth_user: EthAddress<M>,
    multi_transfer: MultiTransferContract,
    bridge_proxy: BridgeProxyContract,
    esdt_safe: EsdtSafeContract,
    bridged_tokens_wrapper: BridgedTokensWrapperContract,
    price_aggregator: PriceAggregatorContract,
}

impl<M: ManagedTypeApi> MultiTransferTestState<M> {
    fn setup() -> Self {
        let world = world();
        let ic = &world.interpreter_context();

        let mut state: MultiTransferTestState<M> = MultiTransferTestState {
            world,
            owner: "address:owner".into(),
            user1: "address:user1".into(),
            user2: "address:user2".into(),
            eth_user: EthAddress {
                raw_addr: ManagedByteArray::default(),
            },
            multi_transfer: MultiTransferContract::new("sc:multi_transfer"),
            bridge_proxy: BridgeProxyContract::new("sc:bridge_proxy"),
            esdt_safe: EsdtSafeContract::new("sc:esdt_safe"),
            bridged_tokens_wrapper: BridgedTokensWrapperContract::new("sc:bridged_tokens_wrapper"),
            price_aggregator: PriceAggregatorContract::new("sc:price_aggregator"),
        };

        state.world.set_state_step(
            SetStateStep::new().put_account(
                &state.owner,
                Account::new()
                    .nonce(1)
                    .balance(BALANCE)
                    .esdt_balance(WEGLD_TOKEN_ID_EXPR, BALANCE),
            ),
        );

        state
    }

    fn multi_transfer_deploy(&mut self) -> &mut Self {
        self.world.set_state_step(
            SetStateStep::new()
                .put_account(&self.owner, Account::new().nonce(1))
                .new_address(&self.owner, 1, MULTI_TRANSFER_ADDRESS_EXPR),
        );

        let ic = &self.world.interpreter_context();
        let bridge_proxy_addr = self
            .bridge_proxy
            .address
            .clone()
            .unwrap_or_sc_panic("Cannot get Bridge Proxy Contract address!");

        self.world.sc_deploy(
            ScDeployStep::new()
                .from(self.owner.clone())
                .code(self.world.code_expression(MULTI_TRANSFER_PATH_EXPR))
                .call(self.multi_transfer.init()),
        );

        self
    }

    fn bridge_proxy_deploy(&mut self) -> &mut Self {
        self.world.set_state_step(
            SetStateStep::new()
                .put_account(&self.owner, Account::new().nonce(1))
                .new_address(&self.owner, 2, BRIDGE_PROXY_ADDRESS_EXPR),
        );

        let ic = &self.world.interpreter_context();
        self.world.sc_deploy(
            ScDeployStep::new()
                .from(self.owner.clone())
                .code(self.world.code_expression(BRIDGE_PROXY_PATH_EXPR))
                .call(self.bridge_proxy.init(self.multi_transfer.address)),
        );

        self
    }

    fn price_aggregator_deploy(&mut self) -> &mut Self {
        self.world.set_state_step(
            SetStateStep::new()
                .put_account(&self.owner, Account::new().nonce(1))
                .new_address(&self.owner, 3, PRICE_AGGREGATOR_ADDRESS_EXPR),
        );

        let ic = &self.world.interpreter_context();
        self.world.sc_deploy(
            ScDeployStep::new()
                .from(self.owner.clone())
                .code(self.world.code_expression(PRICE_AGGREGATOR_PATH_EXPR))
                .call(self.price_aggregator.init(1, 0, ORACLE_ADDRESS_EXPR)),
        );

        self
    }

    fn safe_deploy(&mut self, price_aggregator_contract_address: Address) -> &mut Self {
        self.world.set_state_step(
            SetStateStep::new()
                .put_account(&self.owner, Account::new().nonce(1))
                .new_address(&self.owner, 4, ESDT_SAFE_ADDRESS_EXPR),
        );

        let ic = &self.world.interpreter_context();
        self.world.sc_deploy(
            ScDeployStep::new()
                .from(self.owner.clone())
                .code(self.world.code_expression(ESDT_SAFE_PATH_EXPR))
                .call(self.esdt_safe.init(
                    price_aggregator_contract_address,
                    ESDT_SAFE_ETH_TX_GAS_LIMIT,
                )),
        );

        self
    }

    fn bridged_tokens_wrapper_deploy(&mut self) -> &mut Self {
        self.world.set_state_step(
            SetStateStep::new()
                .put_account(&self.owner, Account::new().nonce(1))
                .new_address(&self.owner, 5, &self.bridged_tokens_wrapper),
        );

        let ic = &self.world.interpreter_context();
        self.world.sc_deploy(
            ScDeployStep::new()
                .from(self.owner.clone())
                .code(self.world.code_expression(BRIDGED_TOKENS_WRAPPER_PATH_EXPR))
                .call(self.bridged_tokens_wrapper.init()),
        );

        self
    }

    fn config_multi_transfer(
        &mut self,
        bridged_tokens_wrapper_contract_address: Address,
        bridge_proxy_contract_address: Address,
    ) {
        self.world.sc_call(
            ScCallStep::new()
                .from(self.owner.clone())
                .to(&self.multi_transfer)
                .call(
                    self.multi_transfer
                        .set_wrapping_contract_address(bridged_tokens_wrapper_contract_address),
                ),
        );

        self.world.sc_call(
            ScCallStep::new()
                .from(self.owner.clone())
                .to(&self.multi_transfer)
                .call(
                    self.multi_transfer
                        .set_bridge_proxy_contract_address(bridge_proxy_contract_address),
                ),
        );
    }
}

#[test]
fn basic_setup_test() {
    let mut test = MultiTransferTestState::setup();
    let bridge_token_id_expr = "str:BRIDGE-123456"; // when specifying the token transfer

    test.multi_transfer_deploy();
    test.bridge_proxy_deploy();
    test.price_aggregator_deploy();
    test.safe_deploy(test.price_aggregator.to_address());
    test.config_multi_transfer(
        test.bridged_tokens_wrapper.to_address(),
        test.bridge_proxy.to_address(),
    );

    test.world.set_state_step(SetStateStep::new().put_account(
        &test.owner,
        Account::new().esdt_balance(bridge_token_id_expr, 1_000u64),
    ));

    let eth_tx = EthTransaction {
        from: test.eth_user,
        to: ManagedAddress::from_address(&test.user1.value),
        token_id: TokenIdentifier::from_esdt_bytes(BRIDGE_TOKEN_ID),
        amount: BigUint::from(500u64),
        tx_nonce: 1u64,
        data: ManagedBuffer::from("data"),
        gas_limit: GAS_LIMIT,
    };

    test.world
        .check_state_step(CheckStateStep::new().put_account(
            &test.multi_transfer,
            CheckAccount::new().check_storage("bridgeProxyContractAddress", "sc:bridge-proxy"),
        ));

    let mut transfers = MultiValueEncoded::new();
    transfers.push(eth_tx);

    test.world.sc_call(
        ScCallStep::new()
            .from(&test.owner)
            .to(&test.multi_transfer)
            .call(
                test.multi_transfer
                    .batch_transfer_esdt_token(1u32, transfers),
            ), // .esdt_transfer(bridge_token_id_expr, 0u64, 500u64),
    );

    // test.world.sc_query(
    //     ScQueryStep::new()
    //         .to(&test.multi_transfer)
    //         .call(test.multi_transfer.get_eth_transaction_by_id(1u32))
    //         .expect_value(eth_tx),
    // );
}
