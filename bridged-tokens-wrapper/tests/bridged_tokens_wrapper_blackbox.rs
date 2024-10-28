#![allow(unused)]
use multiversx_sc::types::{
    BigUint, EsdtLocalRole, ManagedAddress, MultiValueEncoded, TokenIdentifier,
};
use multiversx_sc::types::{TestAddress, TestSCAddress, TestTokenIdentifier};
use multiversx_sc_scenario::api::StaticApi;
use multiversx_sc_scenario::imports::MxscPath;
use multiversx_sc_scenario::{ExpectError, ScenarioTxRun, ScenarioWorld};

use sc_proxies::{bridged_tokens_wrapper_proxy, mock_multisig_proxy};

const BRIDGE_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("BRIDGE-123456");
const WBRIDGE_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("WBRIDGE-123456");
const RELAYER1_ADDRESS: TestAddress = TestAddress::new("relayer1");
const RELAYER2_ADDRESS: TestAddress = TestAddress::new("relayer2");
const GAS_LIMIT: u64 = 10_000_000;

const OWNER_ADDRESS: TestAddress = TestAddress::new("owner");
const MULTI_TRANSFER_ADDRESS: TestSCAddress = TestSCAddress::new("multi-transfer");
const BRIDGE_PROXY_ADDRESS: TestSCAddress = TestSCAddress::new("bridge-proxy");
const ESDT_SAFE_ADDRESS: TestSCAddress = TestSCAddress::new("esdt-safe");
const BRIDGED_TOKENS_WRAPPER_ADDRESS: TestSCAddress = TestSCAddress::new("bridged-tokens-wrapper");
const PRICE_AGGREGATOR_ADDRESS: TestSCAddress = TestSCAddress::new("price-aggregator");
const MULTISIG_ADDRESS: TestSCAddress = TestSCAddress::new("multisig");

const MOCK_BRIDGE_PROXY_PATH_EXPR: MxscPath =
    MxscPath::new("../common/mock-bridge-proxy/output/bridge-proxy.mxsc.json");
const MOCK_MULTI_TRANSFER_PATH_EXPR: MxscPath = MxscPath::new("mxsc:../common/mock-contracts/mock-multi-transfer-esdt/output/mock-multi-transfer-esdt.mxsc.json");
const MOCK_ESDT_SAFE_PATH_EXPR: MxscPath =
    MxscPath::new("mxsc:../common/mock-contrats/mock-esdt-safe/output/mock-esdt-safe.mxsc.json");
const BRIDGED_TOKENS_WRAPPER_CODE_PATH_EXPR: MxscPath =
    MxscPath::new("/output/mock-bridged-tokens-wrapper.mxsc.json");
const MOCK_MULTISIG_PATH_EXPR: MxscPath =
    MxscPath::new("mxsc:../common/mock-contracts/mock-multisig/output/mock-multisig.mxsc.json");
const MOCK_PRICE_AGGREGATOR_CODE_PATH: MxscPath = MxscPath::new(
    "../common/mock-contracts/mock-price-aggregator/output/mock-price-aggregator.mxsc.json",
);

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(
        MOCK_BRIDGE_PROXY_PATH_EXPR,
        mock_bridge_proxy::ContractBuilder,
    );
    blockchain.register_contract(
        BRIDGED_TOKENS_WRAPPER_CODE_PATH_EXPR,
        bridged_tokens_wrapper::ContractBuilder,
    );
    blockchain.register_contract(
        MOCK_MULTI_TRANSFER_PATH_EXPR,
        mock_multi_transfer_esdt::ContractBuilder,
    );
    blockchain.register_contract(MOCK_ESDT_SAFE_PATH_EXPR, mock_esdt_safe::ContractBuilder);
    blockchain.register_contract(MOCK_MULTISIG_PATH_EXPR, mock_multisig::ContractBuilder);
    blockchain.register_contract(
        MOCK_PRICE_AGGREGATOR_CODE_PATH,
        mock_price_aggregator::ContractBuilder,
    );

    blockchain
}

struct BridgedTokensWrapperTestState {
    world: ScenarioWorld,
}

impl BridgedTokensWrapperTestState {
    fn new() -> Self {
        let mut world = world();

        world
            .account(OWNER_ADDRESS)
            .nonce(1)
            .esdt_balance(TokenIdentifier::from(BRIDGE_TOKEN_ID), 10_000u64)
            .esdt_balance(TokenIdentifier::from(WBRIDGE_TOKEN_ID), 10_000u64)
            .esdt_balance(TokenIdentifier::from(BRIDGE_TOKEN_ID), 10_000u64)
            .nonce(1);

        world
            .account(BRIDGE_PROXY_ADDRESS)
            .code(MOCK_BRIDGE_PROXY_PATH_EXPR);

        world
            .account(PRICE_AGGREGATOR_ADDRESS)
            .code(MOCK_PRICE_AGGREGATOR_CODE_PATH);

        Self { world }
    }

    fn deploy_bridged_tokens_wrapper(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(MULTISIG_ADDRESS)
            .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
            .init()
            .code(BRIDGED_TOKENS_WRAPPER_CODE_PATH_EXPR)
            .new_address(BRIDGED_TOKENS_WRAPPER_ADDRESS)
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
                PRICE_AGGREGATOR_ADDRESS,
                1_000u64,
                500u64,
                2usize,
                board,
            )
            .code(MOCK_MULTISIG_PATH_EXPR)
            .new_address(MULTISIG_ADDRESS)
            .run();
        self
    }

    fn config(&mut self) {
        self.world.set_esdt_balance(
            MULTISIG_ADDRESS,
            b"BRIDGE-123456",
            BigUint::from(100_000_000_000u64),
        );

        self.world.set_esdt_local_roles(
            BRIDGED_TOKENS_WRAPPER_ADDRESS,
            b"BRIDGE-123456",
            &[EsdtLocalRole::Mint, EsdtLocalRole::Burn],
        );
    }

    fn deploy_contracts(&mut self) {
        self.multisig_deploy();
        self.deploy_bridged_tokens_wrapper();
    }
}

#[test]
fn test_deploy_bridge_proxy() {
    let mut state = BridgedTokensWrapperTestState::new();
    state.multisig_deploy();
    state.deploy_bridged_tokens_wrapper();
}

#[test]
fn test_add_update_wrapped_token() {
    let mut state = BridgedTokensWrapperTestState::new();
    state.deploy_contracts();
    state.config();

    state
        .world
        .tx()
        .from(MULTISIG_ADDRESS)
        .to(BRIDGED_TOKENS_WRAPPER_ADDRESS)
        .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
        .update_wrapped_token(BRIDGE_TOKEN_ID, 16u32)
        .returns(ExpectError(4, "Universal token was not added yet"))
        .run();

    state
        .world
        .tx()
        .from(MULTISIG_ADDRESS)
        .to(BRIDGED_TOKENS_WRAPPER_ADDRESS)
        .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
        .add_wrapped_token(BRIDGE_TOKEN_ID, 16u32)
        .run();

    state
        .world
        .tx()
        .from(MULTISIG_ADDRESS)
        .to(BRIDGED_TOKENS_WRAPPER_ADDRESS)
        .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
        .update_wrapped_token(BRIDGE_TOKEN_ID, 16u32)
        .run();
}
