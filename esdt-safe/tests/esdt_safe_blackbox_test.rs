#![allow(unused)]
use esdt_safe::*;

use multiversx_sc_scenario::imports::*;
use sc_proxies::esdt_safe_proxy::{self, EsdtSafeProxyMethods};
use sc_proxies::mock_multisig_proxy;

const OWNER_ADDRESS: TestAddress = TestAddress::new("owner");

const BRIDGE_PROXY_ADDRESS: TestSCAddress = TestSCAddress::new("bridge-proxy");
const CROWDFUNDING_ADDRESS: TestSCAddress = TestSCAddress::new("crowfunding");
const MULTI_TRANSFER_ADDRESS: TestSCAddress = TestSCAddress::new("multi-transfer");
const ESDT_SAFE_ADDRESS: TestSCAddress = TestSCAddress::new("esdt-safe");
const FEE_ESTIMATOR_ADDRESS: TestSCAddress = TestSCAddress::new("fee-estimator");
const MULTISIG_ADDRESS: TestSCAddress = TestSCAddress::new("multisig");
const BRIDGED_TOKENS_WRAPPER_ADDRESS: TestSCAddress = TestSCAddress::new("bridged-tokens-wrapper");

const ESDT_SAFE_CODE_PATH: MxscPath = MxscPath::new("output/esdt-safe.mxsc.json");
const MOCK_MULTI_TRANSFER_PATH_EXPR: MxscPath = MxscPath::new(
    "../common/mock-contracts/mock-multi-transfer-esdt/output/mock-multi-transfer-esdt.mxsc.json",
);
const MOCK_BRIDGED_TOKENS_WRAPPER_CODE_PATH_EXPR: MxscPath =
    MxscPath::new("../common/mock-contracts/mock-bridged-tokens-wrapper/output/mock-bridged-tokens-wrapper.mxsc.json");
const MOCK_MULTISIG_CODE_PATH: MxscPath =
    MxscPath::new("../common/mock-contracts/mock-multisig/output/mock-multisig.mxsc.json");
const MOCK_PRICE_AGGREGATOR_CODE_PATH: MxscPath = MxscPath::new(
    "../common/mock-contracts/mock-price-aggregator/output/mock-price-aggregator.mxsc.json",
);
const MOCK_BRIDGE_PROXY_PATH_EXPR: MxscPath =
    MxscPath::new("../common/mock-contracts/mock-bridge-proxy/output/mock-bridge-proxy.mxsc.json");

const TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("TOKEN-123456");
const NON_WHITELISTED_TOKEN: TestTokenIdentifier =
    TestTokenIdentifier::new("NON-WHITELISTED-123456");
const TOKEN_WITH_BURN_ROLE: TestTokenIdentifier = TestTokenIdentifier::new("TOKEN-WITH-OUT");
const NATIVE_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("ESDT-123");
const ETH_TX_GAS_LIMIT: u64 = 150_000;
const ETH_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("ETH-123456");
const ERROR: u64 = 4;
const USER1_ADDRESS: TestAddress = TestAddress::new("user1");
const USER2_ADDRESS: TestAddress = TestAddress::new("user2");
const RELAYER1_ADDRESS: TestAddress = TestAddress::new("relayer1");
const RELAYER2_ADDRESS: TestAddress = TestAddress::new("relayer2");

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();
    blockchain.register_contract(ESDT_SAFE_CODE_PATH, esdt_safe::ContractBuilder);
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
    blockchain.register_contract(MOCK_MULTISIG_CODE_PATH, mock_multisig::ContractBuilder);
    blockchain.register_contract(
        MOCK_BRIDGE_PROXY_PATH_EXPR,
        mock_bridge_proxy::ContractBuilder,
    );

    blockchain
}

struct EsdtSafeTestState {
    world: ScenarioWorld,
}

impl EsdtSafeTestState {
    fn new() -> Self {
        let mut world = world();
        world
            .account(OWNER_ADDRESS)
            .nonce(1)
            .esdt_balance(ETH_TOKEN_ID, 1001u64)
            .esdt_balance(TOKEN_ID, 1_000_000_000_000u64)
            .esdt_balance(NON_WHITELISTED_TOKEN, 1001u64)
            .esdt_balance(NATIVE_TOKEN_ID, 100_000u64)
            .esdt_balance(TOKEN_WITH_BURN_ROLE, 100_000u64);

        Self { world }
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

    fn safe_deploy(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(MULTISIG_ADDRESS)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .init(ETH_TX_GAS_LIMIT)
            .code(ESDT_SAFE_CODE_PATH)
            .new_address(ESDT_SAFE_ADDRESS)
            .run();

        self
    }

    fn config_esdtsafe(&mut self) {
        self.world
            .tx()
            .from(MULTISIG_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .unpause_endpoint()
            .run();

        self.world.set_esdt_local_roles(
            ESDT_SAFE_ADDRESS,
            b"TOKEN-123456",
            &[EsdtLocalRole::Mint, EsdtLocalRole::Burn],
        );
        self.world.set_esdt_balance(
            ESDT_SAFE_ADDRESS,
            b"TOKEN-123456",
            BigUint::from(10_000_000u64),
        );

        self.world.set_esdt_balance(
            MULTISIG_ADDRESS,
            b"TOKEN-123456",
            BigUint::from(10_000_000u64),
        );

        self.world.set_esdt_balance(
            MULTISIG_ADDRESS,
            b"NON-WHITELISTED-123456",
            BigUint::from(10_000_000u64),
        );

        self.world
            .set_esdt_balance(MULTISIG_ADDRESS, b"ESDT-123", BigUint::from(10_000_000u64));

        self.world.set_esdt_balance(
            MULTISIG_ADDRESS,
            b"TOKEN-WITH-OUT",
            BigUint::from(10_000_000u64),
        );

        self.world.set_esdt_balance(
            MULTISIG_ADDRESS,
            b"ETH-123456",
            BigUint::from(10_000_000u64),
        );

        self.esdt_raw_transction()
            .add_token_to_whitelist(
                TOKEN_ID,
                "TOKEN",
                false,
                true,
                BigUint::from(0u64),
                BigUint::from(0u64),
                BigUint::from(0u64),
                OptionalValue::Some(BigUint::from(0u64)),
            )
            .run();

        self.esdt_raw_transction()
            .add_token_to_whitelist(
                TOKEN_WITH_BURN_ROLE,
                "TKN",
                true,
                true,
                BigUint::from(0u64),
                BigUint::from(0u64),
                BigUint::from(0u64),
                OptionalValue::Some(BigUint::from(0u64)),
            )
            .run();

        self.esdt_raw_transction()
            .add_token_to_whitelist(
                NATIVE_TOKEN_ID,
                "NATIVE",
                false,
                true,
                BigUint::from(0u64),
                BigUint::from(0u64),
                BigUint::from(0u64),
                OptionalValue::Some(BigUint::from(0u64)),
            )
            .run();
    }

    fn init_supply_should_fail(
        &mut self,
        token_id: TestTokenIdentifier,
        tx_token_id: TestTokenIdentifier,
        tx_amount: u64,
        amount: u64,
        expected_status: u64,
        expected_error: &str,
    ) {
        self.esdt_raw_transction()
            .init_supply(token_id, BigUint::from(amount))
            .egld_or_single_esdt(
                &EgldOrEsdtTokenIdentifier::esdt(tx_token_id),
                0,
                &BigUint::from(tx_amount),
            )
            .with_result(ExpectError(expected_status, expected_error))
            .run();
    }

    fn init_supply_should_work(
        &mut self,
        token_id: TestTokenIdentifier,
        tx_token_id: TestTokenIdentifier,
        tx_amount: u64,
        amount: u64,
    ) {
        self.esdt_raw_transction()
            .init_supply(token_id, BigUint::from(amount))
            .egld_or_single_esdt(
                &EgldOrEsdtTokenIdentifier::esdt(tx_token_id),
                0,
                &BigUint::from(tx_amount),
            )
            .returns(ReturnsResult)
            .run();
    }

    fn esdt_raw_transction(
        &mut self,
    ) -> EsdtSafeProxyMethods<ScenarioEnvExec<'_>, TestSCAddress, TestSCAddress, ()> {
        self.world
            .tx()
            .from(MULTISIG_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
    }
}

#[test]
fn config_test() {
    let mut state = EsdtSafeTestState::new();
    state.multisig_deploy();
    state.safe_deploy();
    state.config_esdtsafe();
}

#[test]
fn init_supply_test() {
    let mut state = EsdtSafeTestState::new();
    state.multisig_deploy();
    state.safe_deploy();
    state.config_esdtsafe();

    state.init_supply_should_fail(
        NON_WHITELISTED_TOKEN,
        NATIVE_TOKEN_ID,
        10_000u64,
        10_000u64,
        ERROR,
        "Invalid token ID",
    );

    state.init_supply_should_fail(
        NATIVE_TOKEN_ID,
        NATIVE_TOKEN_ID,
        10_000u64,
        1000u64,
        ERROR,
        "Invalid amount",
    );

    state.init_supply_should_fail(
        NON_WHITELISTED_TOKEN,
        NON_WHITELISTED_TOKEN,
        1000u64,
        1000u64,
        ERROR,
        "Token not in whitelist",
    );

    state.init_supply_should_fail(
        TOKEN_WITH_BURN_ROLE,
        TOKEN_WITH_BURN_ROLE,
        1_000u64,
        1_000u64,
        ERROR,
        "Cannot init for mintable/burnable tokens",
    );

    state.init_supply_should_work(NATIVE_TOKEN_ID, NATIVE_TOKEN_ID, 10_000u64, 10_000u64);

    let total_supply = state
        .world
        .query()
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .total_balances(NATIVE_TOKEN_ID)
        .returns(ReturnsResult)
        .run();

    assert_eq!(
        total_supply,
        BigUint::from(10_000u64),
        "Total supply should be 10,000"
    );
    let total_burned = state
        .world
        .query()
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .burn_balances(NATIVE_TOKEN_ID)
        .returns(ReturnsResult)
        .run();

    assert_eq!(
        total_burned,
        BigUint::from(0u64),
        "Total supply should be 0"
    )
}

#[test]
fn init_supply_test_mint_burn() {
    let mut state = EsdtSafeTestState::new();
    state.multisig_deploy();
    state.safe_deploy();
    state.config_esdtsafe();

    state
        .esdt_raw_transction()
        .init_supply_mint_burn(
            NON_WHITELISTED_TOKEN,
            BigUint::from(10_000u64),
            BigUint::from(10_000u64),
        )
        .with_result(ExpectError(ERROR, "Token not in whitelist"))
        .run();

    state
        .esdt_raw_transction()
        .init_supply_mint_burn(TOKEN_ID, BigUint::from(10_000u64), BigUint::from(10_000u64))
        .with_result(ExpectError(
            ERROR,
            "Cannot init for non mintable/burnable tokens",
        ))
        .run();

    state
        .esdt_raw_transction()
        .init_supply_mint_burn(
            TOKEN_WITH_BURN_ROLE,
            BigUint::from(10_000u64),
            BigUint::from(10_000u64),
        )
        .with_result(ReturnsResult)
        .run();

    let total_minted = state
        .world
        .query()
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .mint_balances(TOKEN_WITH_BURN_ROLE)
        .returns(ReturnsResult)
        .run();

    assert_eq!(
        total_minted,
        BigUint::from(10_000u64),
        "Total supply should be 10,000"
    );

    let total_burned = state
        .world
        .query()
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .burn_balances(TOKEN_WITH_BURN_ROLE)
        .returns(ReturnsResult)
        .run();

    assert_eq!(
        total_burned,
        BigUint::from(10_000u64),
        "Total supply should be 10,000"
    );
}
