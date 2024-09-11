#![allow(unused)]
use esdt_safe::*;
use esdt_safe_proxy::EsdtSafeProxyMethods;

use multiversx_sc_scenario::imports::*;

const OWNER_ADDRESS: TestAddress = TestAddress::new("owner");

const ESDT_SAFE_ADDRESS: TestSCAddress = TestSCAddress::new("esdt-safe");
const MULTI_TRANSFER_ADDRESS: TestSCAddress = TestSCAddress::new("multi-transfer-esdt");
const FEE_ESTIMATOR_ADDRESS: TestSCAddress = TestSCAddress::new("price-aggregator");

const ESDT_SAFE_CODE_PATH: MxscPath = MxscPath::new("output/esdt-safe.mxsc.json");
const TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("TOKEN-123456");
const NON_WHITELISTED_TOKEN: TestTokenIdentifier =
    TestTokenIdentifier::new("NON-WHITELISTED-123456");
const TOKEN_WITHOUT_BURN_ROLE: TestTokenIdentifier = TestTokenIdentifier::new("TOKEN-WITH-OUT");
const NATIVE_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("ESDT-123");
const ETH_TX_GAS_LIMIT: u64 = 150_000;
const ETH_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("ETH-123456");
const ERROR: u64 = 4;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();
    blockchain.register_contract(ESDT_SAFE_CODE_PATH, esdt_safe::ContractBuilder);
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
            .esdt_balance(TOKEN_WITHOUT_BURN_ROLE, 100_000u64);

        Self { world }
    }

    fn safe_deploy(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .init(
                FEE_ESTIMATOR_ADDRESS.to_address(),
                MULTI_TRANSFER_ADDRESS.to_address(),
                ETH_TX_GAS_LIMIT,
            )
            .code(ESDT_SAFE_CODE_PATH)
            .new_address(ESDT_SAFE_ADDRESS)
            .run();

        self
    }

    fn config_esdtsafe(&mut self) {
        self.esdt_raw_transction().unpause_endpoint().run();
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

        self.esdt_raw_transction()
            .add_token_to_whitelist(
                TOKEN_ID,
                "TOKEN",
                true,
                false,
                OptionalValue::Some(BigUint::from(0u64)),
            )
            .run();

        self.esdt_raw_transction()
            .add_token_to_whitelist(
                TOKEN_WITHOUT_BURN_ROLE,
                "TOKEN",
                true,
                true,
                OptionalValue::Some(BigUint::from(0u64)),
            )
            .run();

        self.esdt_raw_transction()
            .add_token_to_whitelist(
                NATIVE_TOKEN_ID,
                "NATIVE",
                false,
                true,
                OptionalValue::Some(BigUint::from(0u64)),
            )
            .run();

        self.esdt_raw_transction()
            .set_multi_transfer_contract_address(OptionalValue::Some(
                MULTI_TRANSFER_ADDRESS.to_address(),
            ))
            .run();
    }

    fn init_supply_should_fail(
        &mut self,
        from_address: TestAddress,
        to_address: TestSCAddress,
        token_id: TestTokenIdentifier,
        tx_token_id: TestTokenIdentifier,
        tx_amount: u64,
        amount: u64,
        expected_status: u64,
        expected_error: &str,
    ) {
        self.world
            .tx()
            .from(from_address)
            .to(to_address)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
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
    ) -> EsdtSafeProxyMethods<ScenarioEnvExec<'_>, TestAddress, TestSCAddress, ()> {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
    }
}

#[test]
fn config_test() {
    let mut state = EsdtSafeTestState::new();
    state.safe_deploy();
    state.config_esdtsafe();
}

#[test]
fn init_supply_test() {
    let mut state = EsdtSafeTestState::new();
    state.safe_deploy();
    state.config_esdtsafe();

    state.init_supply_should_fail(
        OWNER_ADDRESS,
        ESDT_SAFE_ADDRESS,
        NON_WHITELISTED_TOKEN,
        NATIVE_TOKEN_ID,
        10_000u64,
        10_000u64,
        ERROR,
        "Invalid token ID",
    );

    state.init_supply_should_fail(
        OWNER_ADDRESS,
        ESDT_SAFE_ADDRESS,
        TOKEN_ID,
        TOKEN_ID,
        10_000u64,
        10_000u64,
        ERROR,
        "Cannot init for non native tokens",
    );

    state.init_supply_should_fail(
        OWNER_ADDRESS,
        ESDT_SAFE_ADDRESS,
        TOKEN_WITHOUT_BURN_ROLE,
        TOKEN_WITHOUT_BURN_ROLE,
        1_000u64,
        1_000u64,
        ERROR,
        "Cannot do the burn action!",
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
