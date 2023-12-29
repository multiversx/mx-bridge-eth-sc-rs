#![allow(unused)]

use bridge_proxy::{config::ProxyTrait as _, ProxyTrait as _};
use bridged_tokens_wrapper::ProxyTrait as _;
use esdt_safe::{EsdtSafe, ProxyTrait as _};
use multi_transfer_esdt::ProxyTrait as _;

use multiversx_sc::{
    api::{HandleConstraints, ManagedTypeApi},
    codec::{
        multi_types::{MultiValueVec, OptionalValue},
        Empty,
    },
    storage::mappers::SingleValue,
    types::{
        Address, BigUint, CodeMetadata, ManagedAddress, ManagedBuffer, ManagedByteArray,
        MultiValueEncoded, TokenIdentifier,
    },
};
use multiversx_sc_modules::pause::ProxyTrait;
use multiversx_sc_scenario::{
    api::{StaticApi, VMHooksApi},
    scenario_format::interpret_trait::{InterpretableFrom, InterpreterContext},
    scenario_model::*,
    ContractInfo, DebugApi, ScenarioWorld,
};

use eth_address::*;
use token_module::ProxyTrait as _;
use transaction::{EthTransaction, EthTransactionPayment};

const BRIDGE_TOKEN_ID: &[u8] = b"BRIDGE-123456";
const BRIDGE_TOKEN_ID_EXPR: &str = "str:BRIDGE-123456";

const USER_ETHEREUM_ADDRESS: &[u8] = b"0x0102030405060708091011121314151617181920";

const GAS_LIMIT: u64 = 100_000_000;

const MULTI_TRANSFER_PATH_EXPR: &str = "file:output/multi-transfer-esdt.wasm";
const BRIDGE_PROXY_PATH_EXPR: &str = "file:../bridge-proxy/output/bridge-proxy.wasm";
const ESDT_SAFE_PATH_EXPR: &str = "file:../esdt-safe/output/esdt-safe.wasm";
const BRIDGED_TOKENS_WRAPPER_PATH_EXPR: &str =
    "file:../bridged-tokens-wrapper/output/bridged-tokens-wrapper.wasm";
const PRICE_AGGREGATOR_PATH_EXPR: &str = "file:../price-aggregator/price-aggregator.wasm";

const MULTI_TRANSFER_ADDRESS_EXPR: &str = "sc:multi_transfer";
const BRIDGE_PROXY_ADDRESS_EXPR: &str = "sc:bridge_proxy";
const ESDT_SAFE_ADDRESS_EXPR: &str = "sc:esdt_safe";
const BRIDGED_TOKENS_WRAPPER_ADDRESS_EXPR: &str = "sc:bridged_tokens_wrapper";
const PRICE_AGGREGATOR_ADDRESS_EXPR: &str = "sc:price_aggregator";

const ORACLE_ADDRESS_EXPR: &str = "address:oracle";
const OWNER_ADDRESS_EXPR: &str = "address:owner";

const ESDT_SAFE_ETH_TX_GAS_LIMIT: u64 = 150_000;

const BALANCE: &str = "2,000,000";

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(
        MULTI_TRANSFER_PATH_EXPR,
        multi_transfer_esdt::ContractBuilder,
    );
    blockchain.register_contract(BRIDGE_PROXY_PATH_EXPR, bridge_proxy::ContractBuilder);

    blockchain.register_contract(ESDT_SAFE_PATH_EXPR, esdt_safe::ContractBuilder);

    blockchain.register_contract(
        BRIDGED_TOKENS_WRAPPER_PATH_EXPR,
        bridged_tokens_wrapper::ContractBuilder,
    );

    blockchain
}

type MultiTransferContract = ContractInfo<multi_transfer_esdt::Proxy<StaticApi>>;
type BridgeProxyContract = ContractInfo<bridge_proxy::Proxy<StaticApi>>;
type EsdtSafeContract = ContractInfo<esdt_safe::Proxy<StaticApi>>;
type BridgedTokensWrapperContract = ContractInfo<bridged_tokens_wrapper::Proxy<StaticApi>>;

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
        };

        let multi_transfer_code = state.world.code_expression(MULTI_TRANSFER_PATH_EXPR);
        let bridge_proxy_code = state.world.code_expression(BRIDGE_PROXY_PATH_EXPR);
        let esdt_safe_code = state.world.code_expression(ESDT_SAFE_PATH_EXPR);
        let bridged_tokens_wrapper_code = state
            .world
            .code_expression(BRIDGED_TOKENS_WRAPPER_PATH_EXPR);

        let roles = vec![
            "ESDTRoleLocalMint".to_string(),
            "ESDTRoleLocalBurn".to_string(),
        ];

        state.world.set_state_step(
            SetStateStep::new()
                .put_account(
                    &state.owner,
                    Account::new()
                        .nonce(1)
                        .balance(BALANCE)
                        .esdt_balance(BRIDGE_TOKEN_ID_EXPR, BALANCE),
                )
                .put_account(&state.user1, Account::new().nonce(1))
                .new_address(&state.owner, 1, MULTI_TRANSFER_ADDRESS_EXPR)
                .new_address(&state.owner, 2, BRIDGE_PROXY_ADDRESS_EXPR)
                .new_address(&state.owner, 3, ESDT_SAFE_ADDRESS_EXPR)
                .put_account(
                    ESDT_SAFE_ADDRESS_EXPR,
                    Account::new()
                        .code(&esdt_safe_code)
                        .owner(&state.owner)
                        .esdt_roles(BRIDGE_TOKEN_ID_EXPR, roles)
                        .esdt_balance(BRIDGE_TOKEN_ID_EXPR, "1_000"),
                )
                .new_address(&state.owner, 4, BRIDGED_TOKENS_WRAPPER_ADDRESS_EXPR),
        );
        state
    }

    fn multi_transfer_deploy(&mut self) -> &mut Self {
        self.world.sc_deploy(
            ScDeployStep::new()
                .from(self.owner.clone())
                .code(self.world.code_expression(MULTI_TRANSFER_PATH_EXPR))
                .call(self.multi_transfer.init()),
        );

        self
    }

    fn bridge_proxy_deploy(&mut self) -> &mut Self {
        self.world.sc_deploy(
            ScDeployStep::new()
                .from(self.owner.clone())
                .code(self.world.code_expression(BRIDGE_PROXY_PATH_EXPR))
                .call(self.bridge_proxy.init(self.multi_transfer.to_address())),
        );

        self
    }

    fn safe_deploy(&mut self, price_aggregator_contract_address: Address) -> &mut Self {
        self.world.sc_call(
            ScCallStep::new().from(self.owner.clone()).call(
                self.esdt_safe
                    .upgrade(ManagedAddress::zero(), ESDT_SAFE_ETH_TX_GAS_LIMIT),
            ),
        );

        self
    }

    fn bridged_tokens_wrapper_deploy(&mut self) -> &mut Self {
        self.world.sc_deploy(
            ScDeployStep::new()
                .from(self.owner.clone())
                .code(self.world.code_expression(BRIDGED_TOKENS_WRAPPER_PATH_EXPR))
                .call(self.bridged_tokens_wrapper.init()),
        );

        self
    }

    fn config_multi_transfer(&mut self) {
        self.world
            .sc_call(
                ScCallStep::new()
                    .from(self.owner.clone())
                    .to(&self.multi_transfer)
                    .call(
                        self.multi_transfer.set_wrapping_contract_address(
                            self.bridged_tokens_wrapper.to_address(),
                        ),
                    ),
            )
            .sc_call(
                ScCallStep::new()
                    .from(self.owner.clone())
                    .to(&self.multi_transfer)
                    .call(
                        self.multi_transfer
                            .set_bridge_proxy_contract_address(self.bridge_proxy.to_address()),
                    ),
            )
            .sc_call(
                ScCallStep::new()
                    .from(self.owner.clone())
                    .to(&self.multi_transfer)
                    .call(
                        self.multi_transfer
                            .set_esdt_safe_contract_address(self.esdt_safe.to_address()),
                    ),
            )
            .sc_call(
                ScCallStep::new()
                    .from(self.owner.clone())
                    .to(&self.esdt_safe)
                    .call(
                        self.esdt_safe
                            .set_multi_transfer_contract_address(self.multi_transfer.to_address()),
                    ),
            )
            .sc_call(
                ScCallStep::new()
                    .from(self.owner.clone())
                    .to(&self.esdt_safe)
                    .call(self.esdt_safe.add_token_to_whitelist(
                        TokenIdentifier::from_esdt_bytes("BRIDGE-123456"),
                        "BRIDGE",
                        true,
                        BigUint::from(ESDT_SAFE_ETH_TX_GAS_LIMIT),
                    )),
            )
            .sc_call(
                ScCallStep::new()
                    .from(self.owner.clone())
                    .to(&self.esdt_safe)
                    .call(self.esdt_safe.set_accumulated_burned_tokens(
                        TokenIdentifier::from_esdt_bytes("BRIDGE-123456"),
                        BigUint::from(1_000u64),
                    )),
            );

        //mint_burn_allowed

        // .sc_call(
        //     ScCallStep::new()
        //         .from(self.owner.clone())
        //         .to(&self.bridge_proxy)
        //         .call(
        //             self.bridge_proxy
        //                 .set_multi_transfer_contract_address(self.multi_transfer.to_address()),
        //         ),
        // );
    }
}

#[test]
fn basic_setup_test() {
    let mut test: MultiTransferTestState<StaticApi> = MultiTransferTestState::setup();
    let bridge_token_id_expr = "str:BRIDGE-123456"; // when specifying the token transfer

    test.multi_transfer_deploy();
    test.bridge_proxy_deploy();
    test.safe_deploy(Address::zero());
    test.bridged_tokens_wrapper_deploy();
    test.config_multi_transfer();

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

    test.world.check_state_step(
        CheckStateStep::new().put_account(
            &test.multi_transfer,
            CheckAccount::new()
                .check_storage("str:bridgeProxyContractAddress", "sc:bridge_proxy")
                .check_storage("str:lastBatchId", "0x01")
                .check_storage("str:wrappingContractAddress", "sc:bridged_tokens_wrapper")
                .check_storage("str:maxTxBatchBlockDuration", "0xffffffffffffffff")
                .check_storage("str:maxTxBatchSize", "10")
                .check_storage("str:firstBatchId", "0x01")
                .check_storage("str:esdtSafeContractAddress", "sc:esdt_safe"),
        ),
    );
}

#[test]
fn basic_transfer_test() {
    let mut test: MultiTransferTestState<StaticApi> = MultiTransferTestState::setup();
    let token_amount = BigUint::from(500u64);

    test.multi_transfer_deploy();
    test.bridge_proxy_deploy();
    test.safe_deploy(Address::zero());
    test.bridged_tokens_wrapper_deploy();
    test.config_multi_transfer();

    let eth_tx = EthTransaction {
        from: test.eth_user,
        to: ManagedAddress::from_address(&test.user1.value),
        token_id: TokenIdentifier::from_esdt_bytes(BRIDGE_TOKEN_ID),
        amount: token_amount.clone(),
        tx_nonce: 1u64,
        data: ManagedBuffer::from("data"),
        gas_limit: GAS_LIMIT,
    };

    test.world.check_state_step(
        CheckStateStep::new().put_account(
            &test.multi_transfer,
            CheckAccount::new()
                .check_storage("str:bridgeProxyContractAddress", "sc:bridge_proxy")
                .check_storage("str:lastBatchId", "0x01")
                .check_storage("str:wrappingContractAddress", "sc:bridged_tokens_wrapper")
                .check_storage("str:maxTxBatchBlockDuration", "0xffffffffffffffff")
                .check_storage("str:maxTxBatchSize", "10")
                .check_storage("str:firstBatchId", "0x01")
                .check_storage("str:esdtSafeContractAddress", "sc:esdt_safe"),
        ),
    );

    let mut transfers = MultiValueEncoded::new();
    transfers.push(eth_tx);

    test.world.sc_call(
        ScCallStep::new()
            .from(&test.owner)
            .to(&test.esdt_safe)
            .call(test.esdt_safe.unpause_endpoint()),
    );

    test.world.sc_call(
        ScCallStep::new()
            .from(&test.owner)
            .to(&test.bridged_tokens_wrapper)
            .call(test.bridged_tokens_wrapper.unpause_endpoint()),
    );

    test.world.sc_call(
        ScCallStep::new()
            .from(&test.owner)
            .to(&test.multi_transfer)
            .call(
                test.multi_transfer
                    .batch_transfer_esdt_token(1u32, transfers),
            ),
    );

    test.world
        .check_state_step(CheckStateStep::new().put_account(
            test.user1,
            CheckAccount::new().esdt_balance(BRIDGE_TOKEN_ID_EXPR, token_amount),
        ));
}
