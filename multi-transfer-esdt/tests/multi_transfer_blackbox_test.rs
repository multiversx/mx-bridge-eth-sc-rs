#![allow(unused)]

use bridge_proxy::{
    bridge_proxy_contract_proxy, config::ProxyTrait as _, esdt_safe_proxy, ProxyTrait as _,
};
use bridged_tokens_wrapper::ProxyTrait as _;
use esdt_safe::{EsdtSafe, ProxyTrait as _};
use multi_transfer_esdt::{bridged_tokens_wrapper_proxy, multi_transfer_proxy, ProxyTrait as _};

use multiversx_sc::{
    api::{HandleConstraints, ManagedTypeApi},
    codec::{
        multi_types::{MultiValueVec, OptionalValue},
        Empty,
    },
    storage::mappers::SingleValue,
    types::{
        Address, BigUint, CodeMetadata, ManagedAddress, ManagedBuffer, ManagedByteArray,
        ManagedVec, MultiValueEncoded, ReturnsNewManagedAddress, ReturnsRawResult, TestAddress,
        TestSCAddress, TestTokenIdentifier, TokenIdentifier,
    },
};
use multiversx_sc_modules::pause::ProxyTrait;
use multiversx_sc_scenario::{
    api::{StaticApi, VMHooksApi, VMHooksApiBackend},
    imports::MxscPath,
    scenario_format::interpret_trait::{InterpretableFrom, InterpreterContext},
    scenario_model::*,
    ContractInfo, DebugApi, ExpectValue, ScenarioTxRun, ScenarioWorld,
};

use eth_address::*;
use token_module::ProxyTrait as _;
use transaction::{CallData, EthTransaction};

const BRIDGE_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("BRIDGE-123456");

const USER_ETHEREUM_ADDRESS: &[u8] = b"0x0102030405060708091011121314151617181920";

const GAS_LIMIT: u64 = 100_000_000;

const MULTI_TRANSFER_CODE_PATH: MxscPath = MxscPath::new("output/multi-transfer-esdt.mxsc.json");
const BRIDGE_PROXY_CODE_PATH: MxscPath =
    MxscPath::new("../bridge-proxy/output/bridge-proxy.mxsc.json");
const ESDT_SAFE_CODE_PATH: MxscPath = MxscPath::new("../esdt-safe/output/esdt-safe.mxsc.json");
const BRIDGED_TOKENS_WRAPPER_CODE_PATH: MxscPath =
    MxscPath::new("../bridged-tokens-wrapper/output/bridged-tokens-wrapper.mxsc.json");
const PRICE_AGGREGATOR_CODE_PATH: MxscPath =
    MxscPath::new("../price-aggregator/price-aggregator.mxsc.json");

const MULTI_TRANSFER_ADDRESS: TestSCAddress = TestSCAddress::new("multi-transfer");
const BRIDGE_PROXY_ADDRESS: TestSCAddress = TestSCAddress::new("bridge-proxy");
const ESDT_SAFE_ADDRESS: TestSCAddress = TestSCAddress::new("esdt-safe");
const BRIDGED_TOKENS_WRAPPER_ADDRESS: TestSCAddress = TestSCAddress::new("bridged-tokens-wrapper");
const PRICE_AGGREGATOR_ADDRESS: TestSCAddress = TestSCAddress::new("price-aggregator");

const ORACLE_ADDRESS: TestAddress = TestAddress::new("oracle");
const OWNER_ADDRESS: TestAddress = TestAddress::new("owner");
const USER1_ADDRESS: TestAddress = TestAddress::new("user1");
const USER2_ADDRESS: TestAddress = TestAddress::new("user2");

const ESDT_SAFE_ETH_TX_GAS_LIMIT: u64 = 150_000;

const BALANCE: &str = "2,000,000";

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(
        MULTI_TRANSFER_CODE_PATH,
        multi_transfer_esdt::ContractBuilder,
    );
    blockchain.register_contract(BRIDGE_PROXY_CODE_PATH, bridge_proxy::ContractBuilder);

    blockchain.register_contract(ESDT_SAFE_CODE_PATH, esdt_safe::ContractBuilder);

    blockchain.register_contract(
        BRIDGED_TOKENS_WRAPPER_CODE_PATH,
        bridged_tokens_wrapper::ContractBuilder,
    );

    blockchain
}

type MultiTransferContract = ContractInfo<multi_transfer_esdt::Proxy<StaticApi>>;
type BridgeProxyContract = ContractInfo<bridge_proxy::Proxy<StaticApi>>;
type EsdtSafeContract = ContractInfo<esdt_safe::Proxy<StaticApi>>;
type BridgedTokensWrapperContract = ContractInfo<bridged_tokens_wrapper::Proxy<StaticApi>>;

struct MultiTransferTestState {
    world: ScenarioWorld,
}

impl MultiTransferTestState {
    fn new() -> Self {
        // let world = world();
        // let ic = &world.interpreter_context();

        // let mut state: MultiTransferTestState<M> = MultiTransferTestState {
        //     world,
        //     owner: "address:owner".into(),
        //     user1: "address:user1".into(),
        //     user2: "address:user2".into(),
        //     eth_user: EthAddress {
        //         raw_addr: ManagedByteArray::default(),
        //     },
        //     multi_transfer: MultiTransferContract::new("sc:multi_transfer"),
        //     bridge_proxy: BridgeProxyContract::new("sc:bridge_proxy"),
        //     esdt_safe: EsdtSafeContract::new("sc:esdt_safe"),
        //     bridged_tokens_wrapper: BridgedTokensWrapperContract::new("sc:bridged_tokens_wrapper"),
        // };

        // let multi_transfer_code = state.world.code_expression(MULTI_TRANSFER_PATH_EXPR);
        // let bridge_proxy_code = state.world.code_expression(BRIDGE_PROXY_PATH_EXPR);
        // let esdt_safe_code = state.world.code_expression(ESDT_SAFE_PATH_EXPR);
        // let bridged_tokens_wrapper_code = state
        //     .world
        //     .code_expression(BRIDGED_TOKENS_WRAPPER_PATH_EXPR);

        // let roles = vec![
        //     "ESDTRoleLocalMint".to_string(),
        //     "ESDTRoleLocalBurn".to_string(),
        // ];

        // state.world.set_state_step(
        //     SetStateStep::new()
        //         .put_account(
        //             &state.owner,
        //             Account::new()
        //                 .nonce(1)
        //                 .balance(BALANCE)
        //                 .esdt_balance(BRIDGE_TOKEN_ID_EXPR, BALANCE),
        //         )
        //         .put_account(&state.user1, Account::new().nonce(1))
        //         .new_address(&state.owner, 1, MULTI_TRANSFER_ADDRESS_EXPR)
        //         .new_address(&state.owner, 2, BRIDGE_PROXY_ADDRESS_EXPR)
        //         .new_address(&state.owner, 3, ESDT_SAFE_ADDRESS_EXPR)
        //         .put_account(
        //             ESDT_SAFE_ADDRESS_EXPR,
        //             Account::new()
        //                 .code(&esdt_safe_code)
        //                 .owner(&state.owner)
        //                 .esdt_roles(BRIDGE_TOKEN_ID_EXPR, roles)
        //                 .esdt_balance(BRIDGE_TOKEN_ID_EXPR, "1_000"),
        //         )
        //         .new_address(&state.owner, 4, BRIDGED_TOKENS_WRAPPER_ADDRESS_EXPR),
        // );
        // state

        let mut world = world();

        world
            .account(OWNER_ADDRESS)
            .nonce(1)
            .esdt_balance(BRIDGE_TOKEN_ID, 1000u64)
            .account(USER1_ADDRESS)
            .nonce(1)
            .account(USER2_ADDRESS)
            .nonce(1);

        let roles = vec![
            "ESDTRoleLocalMint".to_string(),
            "ESDTRoleLocalBurn".to_string(),
        ];
        world
            .account(ESDT_SAFE_ADDRESS)
            .esdt_roles(BRIDGE_TOKEN_ID, roles)
            .code(ESDT_SAFE_CODE_PATH)
            .owner(OWNER_ADDRESS);

        Self { world }
    }

    fn multi_transfer_deploy(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(multi_transfer_proxy::MultiTransferEsdtProxy)
            .init()
            .code(MULTI_TRANSFER_CODE_PATH)
            .new_address(MULTI_TRANSFER_ADDRESS)
            .run();
        self
    }

    fn bridge_proxy_deploy(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
            .init(OptionalValue::Some(MULTI_TRANSFER_ADDRESS.to_address()))
            .code(BRIDGE_PROXY_CODE_PATH)
            .new_address(BRIDGE_PROXY_ADDRESS)
            .run();

        self
    }

    fn safe_deploy(&mut self, price_aggregator_contract_address: Address) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .upgrade(
                ManagedAddress::zero(),
                MULTI_TRANSFER_ADDRESS.to_address(),
                ESDT_SAFE_ETH_TX_GAS_LIMIT,
            )
            .code(ESDT_SAFE_CODE_PATH)
            .run();

        self
    }

    fn bridged_tokens_wrapper_deploy(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
            .init()
            .code(BRIDGED_TOKENS_WRAPPER_CODE_PATH)
            .new_address(BRIDGED_TOKENS_WRAPPER_ADDRESS)
            .run();

        self
    }

    fn config_multi_transfer(&mut self) {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(MULTI_TRANSFER_ADDRESS)
            .typed(multi_transfer_proxy::MultiTransferEsdtProxy)
            .set_wrapping_contract_address(OptionalValue::Some(
                BRIDGED_TOKENS_WRAPPER_ADDRESS.to_address(),
            ))
            .run();

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(MULTI_TRANSFER_ADDRESS)
            .typed(multi_transfer_proxy::MultiTransferEsdtProxy)
            .set_bridge_proxy_contract_address(OptionalValue::Some(
                BRIDGE_PROXY_ADDRESS.to_address(),
            ))
            .run();

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(MULTI_TRANSFER_ADDRESS)
            .typed(multi_transfer_proxy::MultiTransferEsdtProxy)
            .set_esdt_safe_contract_address(OptionalValue::Some(ESDT_SAFE_ADDRESS.to_address()))
            .run();

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .set_multi_transfer_contract_address(OptionalValue::Some(
                MULTI_TRANSFER_ADDRESS.to_address(),
            ))
            .run();

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .add_token_to_whitelist(
                TokenIdentifier::from_esdt_bytes("BRIDGE-123456"),
                "BRIDGE",
                true,
                false,
                OptionalValue::Some(BigUint::from(ESDT_SAFE_ETH_TX_GAS_LIMIT)),
            )
            .run();
    }
}

#[test]
fn basic_transfer_test() {
    let mut state = MultiTransferTestState::new();
    let token_amount = BigUint::from(500u64);

    state.multi_transfer_deploy();
    state.bridge_proxy_deploy();
    state.safe_deploy(Address::zero());
    state.bridged_tokens_wrapper_deploy();
    state.config_multi_transfer();

    let eth_tx = EthTransaction {
        from: EthAddress {
            raw_addr: ManagedByteArray::default(),
        },
        to: ManagedAddress::from(USER1_ADDRESS.eval_to_array()), // TODO: solve this
        token_id: TokenIdentifier::from(BRIDGE_TOKEN_ID),
        amount: token_amount.clone(),
        tx_nonce: 1u64,
        call_data: Some(CallData {
            endpoint: ManagedBuffer::from("data"),
            gas_limit: GAS_LIMIT,
            args: ManagedVec::new(),
        }),
    };

    // state
    //     .world
    //     .check_account(MULTI_TRANSFER_ADDRESS)
    //     .check_storage(
    //         "str:bridgeProxyContractAddress",
    //         BRIDGE_PROXY_ADDRESS.to_address().,
    //     )
    //     .check_storage("str:lastBatchId", "0x01")
    //     .check_storage("str:wrappingContractAddress", "sc:bridged_tokens_wrapper")
    //     .check_storage("str:maxTxBatchBlockDuration", "0xffffffffffffffff")
    //     .check_storage("str:maxTxBatchSize", "10")
    //     .check_storage("str:firstBatchId", "0x01")
    //     .check_storage("str:esdtSafeContractAddress", "sc:esdt_safe");

    let mut transfers = MultiValueEncoded::new();
    transfers.push(eth_tx);

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .unpause_endpoint()
        .run();

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(BRIDGED_TOKENS_WRAPPER_ADDRESS)
        .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
        .unpause_endpoint()
        .run();

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTI_TRANSFER_ADDRESS)
        .typed(multi_transfer_proxy::MultiTransferEsdtProxy)
        .batch_transfer_esdt_token(1u32, transfers)
        .run();

    state
        .world
        .check_account(USER1_ADDRESS)
        .esdt_balance(BRIDGE_TOKEN_ID, token_amount);
}
