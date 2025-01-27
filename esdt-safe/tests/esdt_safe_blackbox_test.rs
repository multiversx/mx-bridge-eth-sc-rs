#![allow(unused)]
use esdt_safe::*;

use eth_address::EthAddress;
use mock_proxies::mock_multisig_proxy;
use multiversx_sc_scenario::imports::*;
use sc_proxies::esdt_safe_proxy::{self, EsdtSafeProxyMethods};
use transaction::transaction_status::TransactionStatus;
use transaction::Transaction;
use tx_batch_module::BatchStatus;

const OWNER_ADDRESS: TestAddress = TestAddress::new("owner");

const BRIDGE_PROXY_ADDRESS: TestSCAddress = TestSCAddress::new("bridge-proxy");
const CROWDFUNDING_ADDRESS: TestSCAddress = TestSCAddress::new("crowfunding");
const MULTI_TRANSFER_ADDRESS: TestSCAddress = TestSCAddress::new("multi-transfer");
const ESDT_SAFE_ADDRESS: TestSCAddress = TestSCAddress::new("esdt-safe");
const PRICE_AGGREGATOR: TestSCAddress = TestSCAddress::new("price-aggregator");
const MULTISIG_ADDRESS: TestSCAddress = TestSCAddress::new("multisig");
const BRIDGED_TOKENS_WRAPPER_ADDRESS: TestSCAddress = TestSCAddress::new("bridged-tokens-wrapper");
const OWNER_ADDRESS_EXPR: &str = "address:owner";
const ESTD_SAFE_ADDRESS_EXPR: &str = "sc:esdt-safe";

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
const MINT_BURN_TOKEN: TestTokenIdentifier = TestTokenIdentifier::new("MINT-BURN-123456");
const NON_WHITELISTED_TOKEN: TestTokenIdentifier =
    TestTokenIdentifier::new("NON-WHITELISTED-123456");
const TOKEN_WITH_BURN_ROLE: TestTokenIdentifier = TestTokenIdentifier::new("TOKEN-WITH");
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
            .esdt_balance(NATIVE_TOKEN_ID, 300_000_000_000u64)
            .esdt_balance(MINT_BURN_TOKEN, 200000u64)
            .esdt_balance(TOKEN_WITH_BURN_ROLE, 100_000_000_000u64);

        world
            .account(PRICE_AGGREGATOR)
            .code(MOCK_PRICE_AGGREGATOR_CODE_PATH);

        world
            .account(MULTI_TRANSFER_ADDRESS)
            .esdt_balance(NATIVE_TOKEN_ID, 300_000_000_000u64)
            .esdt_balance(TOKEN_WITH_BURN_ROLE, 100_000_000_000u64)
            .esdt_balance(TOKEN_ID, 300_000_000u64)
            .code(MOCK_MULTI_TRANSFER_PATH_EXPR)
            .account(BRIDGE_PROXY_ADDRESS)
            .esdt_balance(TOKEN_ID, 100_000_000u64)
            .code(MOCK_BRIDGE_PROXY_PATH_EXPR)
            .account(BRIDGED_TOKENS_WRAPPER_ADDRESS)
            .esdt_balance(TOKEN_ID, 100_000_000u64)
            .code(MOCK_BRIDGED_TOKENS_WRAPPER_CODE_PATH_EXPR);

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
                PRICE_AGGREGATOR,
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
            ESDT_SAFE_ADDRESS,
            b"MINT-BURN-123456",
            BigUint::from(20_000_000u64),
        );

        self.world.set_esdt_balance(
            MULTISIG_ADDRESS,
            b"MINT-BURN-123456",
            BigUint::from(200_000_000_000u64),
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
            b"TOKEN-WITH",
            BigUint::from(10_000_000u64),
        );

        self.world.set_esdt_balance(
            MULTISIG_ADDRESS,
            b"ETH-123456",
            BigUint::from(10_000_000u64),
        );

        self.esdt_raw_transaction()
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

        self.esdt_raw_transaction()
            .add_token_to_whitelist(
                MINT_BURN_TOKEN,
                "TOKEN",
                true,
                false,
                BigUint::from(0u64),
                BigUint::from(2000u64),
                BigUint::from(0u64),
                OptionalValue::Some(BigUint::from(0u64)),
            )
            .run();

        self.esdt_raw_transaction()
            .add_token_to_whitelist(
                TOKEN_WITH_BURN_ROLE,
                "TKN",
                true,
                false,
                BigUint::from(0u64),
                BigUint::from(0u64),
                BigUint::from(0u64),
                OptionalValue::Some(BigUint::from(0u64)),
            )
            .run();

        self.esdt_raw_transaction()
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
        self.esdt_raw_transaction()
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
        self.esdt_raw_transaction()
            .init_supply(token_id, BigUint::from(amount))
            .egld_or_single_esdt(
                &EgldOrEsdtTokenIdentifier::esdt(tx_token_id),
                0,
                &BigUint::from(tx_amount),
            )
            .returns(ReturnsResult)
            .run();
    }
    fn add_refund_batch_tx_single_payment_should_fail(
        &mut self,
        from_address: TestSCAddress,
        to_address: TestSCAddress,
        transfers: ManagedVec<StaticApi, Transaction<StaticApi>>,
        token_id: TestTokenIdentifier,
        amount: u64,
        expected_error: &str,
    ) {
        self.world
            .tx()
            .from(from_address)
            .to(to_address)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .add_refund_batch(transfers)
            .egld_or_single_esdt(
                &EgldOrEsdtTokenIdentifier::esdt(token_id),
                0,
                &BigUint::from(amount),
            )
            .returns(ExpectError(ERROR, expected_error))
            .run();
    }

    fn add_refund_batch_tx_multiple_payment_should_fail(
        &mut self,
        from_address: TestSCAddress,
        to_address: TestSCAddress,
        transfers: ManagedVec<StaticApi, Transaction<StaticApi>>,
        payment: EgldOrMultiEsdtPayment<StaticApi>,
        expected_error: &str,
    ) {
        self.world
            .tx()
            .from(from_address)
            .to(to_address)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .add_refund_batch(transfers)
            .egld_or_multi_esdt(payment)
            .returns(ExpectError(ERROR, expected_error))
            .run();
    }

    fn single_transaction_should_fail(
        &mut self,
        token_id: TestTokenIdentifier,
        amount: u64,
        expected_error: &str,
    ) {
        self.esdt_raw_transaction()
            .create_transaction(
                EthAddress {
                    raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
                },
                OptionalValue::<BigUint<StaticApi>>::None,
            )
            .egld_or_single_esdt(
                &EgldOrEsdtTokenIdentifier::esdt(token_id),
                0,
                &BigUint::from(amount),
            )
            .returns(ExpectError(ERROR, expected_error))
            .run();
    }

    fn single_transaction_should_work(&mut self, token_id: TestTokenIdentifier, amount: u64) {
        self.esdt_raw_transaction()
            .create_transaction(
                EthAddress {
                    raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
                },
                OptionalValue::<BigUint<StaticApi>>::None,
            )
            .egld_or_single_esdt(
                &EgldOrEsdtTokenIdentifier::esdt(token_id),
                0,
                &BigUint::from(amount),
            )
            .returns(ReturnsResult)
            .run();
    }

    fn set_transaction_batch_status_should_fail(
        &mut self,
        batch_id: u32,
        statuses: MultiValueEncoded<StaticApi, TransactionStatus>,
        expected_status: u64,
        expected_error: &str,
    ) {
        self.esdt_raw_transaction()
            .set_transaction_batch_status(batch_id, statuses)
            .returns(ExpectError(expected_status, expected_error))
            .run();
    }

    fn set_transaction_batch_status_should_work(
        &mut self,
        batch_id: u32,
        statuses: MultiValueEncoded<StaticApi, TransactionStatus>,
    ) {
        self.esdt_raw_transaction()
            .set_transaction_batch_status(batch_id, statuses)
            .returns(ReturnsResult)
            .run();
    }

    fn esdt_raw_transaction(
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
fn upgrade_test() {
    let mut state = EsdtSafeTestState::new();
    state.multisig_deploy();
    state.safe_deploy();
    state.config_esdtsafe();

    let eth_tx_gas_limit_before = state
        .world
        .query()
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .eth_tx_gas_limit()
        .returns(ReturnsResult)
        .run();
    assert_eq!(
        eth_tx_gas_limit_before,
        BigUint::from(ETH_TX_GAS_LIMIT),
        "Initial eth_tx_gas_limit should match the set value"
    );

    let paused_state_before = state
        .world
        .query()
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .paused_status()
        .returns(ReturnsResult)
        .run();
    assert!(
        !paused_state_before,
        "Contract should not be paused before upgrade"
    );

    let new_eth_tx_gas_limit = BigUint::from(5000u64);
    state
        .world
        .tx()
        .from(MULTISIG_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .upgrade(new_eth_tx_gas_limit.clone())
        .code(ESDT_SAFE_CODE_PATH)
        .run();

    let eth_tx_gas_limit_after = state
        .world
        .query()
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .eth_tx_gas_limit()
        .returns(ReturnsResult)
        .run();
    assert_eq!(
        eth_tx_gas_limit_after,
        new_eth_tx_gas_limit.clone(),
        "eth_tx_gas_limit should be updated after upgrade"
    );

    let paused_state_after = state
        .world
        .query()
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .paused_status()
        .returns(ReturnsResult)
        .run();
    assert!(
        paused_state_after,
        "Contract should be paused after upgrade"
    );
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
        .esdt_raw_transaction()
        .init_supply_mint_burn(
            NON_WHITELISTED_TOKEN,
            BigUint::from(10_000u64),
            BigUint::from(10_000u64),
        )
        .with_result(ExpectError(ERROR, "Token not in whitelist"))
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
        BigUint::from(0u64),
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
        BigUint::from(0u64),
        "Total supply should be 10,000"
    );
}

#[test]
fn set_transaction_batch_status_test() {
    let mut state = EsdtSafeTestState::new();
    state.multisig_deploy();
    state.safe_deploy();
    state.config_esdtsafe();

    let mut tx_statuses = MultiValueEncoded::<StaticApi, TransactionStatus>::new();
    tx_statuses.push(TransactionStatus::Executed);
    let mut tx_multiple_statuses = MultiValueEncoded::<StaticApi, TransactionStatus>::new();
    tx_multiple_statuses.push(TransactionStatus::Executed);
    tx_multiple_statuses.push(TransactionStatus::Pending);
    let mut tx_statuses_invalid = MultiValueEncoded::<StaticApi, TransactionStatus>::new();
    tx_statuses_invalid.push(TransactionStatus::Pending);

    state
        .world
        .tx()
        .from(MULTISIG_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .set_eth_tx_gas_limit(1000u64)
        .run();

    state.single_transaction_should_work(TOKEN_ID, 100_000u64);

    state.set_transaction_batch_status_should_fail(
        5u32,
        tx_statuses.clone(),
        ERROR,
        "Batches must be processed in order",
    );

    state.set_transaction_batch_status_should_fail(
        1u32,
        tx_multiple_statuses.clone(),
        ERROR,
        "Invalid number of statuses provided",
    );

    state.set_transaction_batch_status_should_fail(
        1u32,
        tx_statuses_invalid.clone(),
        ERROR,
        "Transaction status may only be set to Executed or Rejected",
    );

    state.set_transaction_batch_status_should_work(1u32, tx_statuses.clone());

    let result = state
        .world
        .query()
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .get_batch_status(1u64)
        .returns(ReturnsResult)
        .run();
    assert_eq!(result, BatchStatus::AlreadyProcessed);
}

#[test]
fn esdt_safe_create_transaction() {
    let mut state = EsdtSafeTestState::new();
    state.multisig_deploy();
    state.safe_deploy();

    state.world.set_esdt_balance(
        MULTISIG_ADDRESS,
        b"TOKEN-WITH",
        BigUint::from(10_000_000u64),
    );

    state.single_transaction_should_fail(
        TOKEN_WITH_BURN_ROLE,
        10u64,
        "Cannot create transaction while paused",
    );

    state.config_esdtsafe();

    state.single_transaction_should_fail(
        TOKEN_WITH_BURN_ROLE,
        0u64,
        "Transaction fees cost more than the entire bridged amount",
    );

    state.single_transaction_should_fail(NON_WHITELISTED_TOKEN, 100u64, "Token not in whitelist");

    state.single_transaction_should_fail(TOKEN_WITH_BURN_ROLE, 100u64, "Not enough minted tokens!");

    state.single_transaction_should_fail(MINT_BURN_TOKEN, 10u64, "Cannot do the burn action!");

    let refund_info = sc_proxies::esdt_safe_proxy::RefundInfo::<StaticApi> {
        address: ManagedAddress::from(OWNER_ADDRESS.eval_to_array()),
        initial_batch_id: 1u64,
        initial_nonce: 1u64,
    };

    state
        .world
        .tx()
        .from(BRIDGE_PROXY_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .create_refund_transaction(
            EthAddress {
                raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
            },
            OptionalValue::Some(refund_info.clone()),
        )
        .single_esdt(&TOKEN_ID.into(), 0, &BigUint::from(10u64))
        .run();

    state
        .world
        .tx()
        .from(BRIDGED_TOKENS_WRAPPER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .create_transaction(
            EthAddress {
                raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
            },
            OptionalValue::<BigUint<StaticApi>>::None,
        )
        .single_esdt(&TOKEN_ID.into(), 0, &BigUint::from(10u64))
        .run();

    state
        .world
        .tx()
        .from(MULTISIG_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .set_eth_tx_gas_limit(1000u64)
        .run();

    state
        .world
        .tx()
        .from(MULTISIG_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .set_default_price_per_gas_unit(TOKEN_ID, 10u64)
        .run();

    state.single_transaction_should_work(TOKEN_ID, 100_000u64);

    let total_accumulated_transaction_fee = state
        .world
        .query()
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .accumulated_transaction_fees(TOKEN_ID)
        .returns(ReturnsResult)
        .run();

    assert_eq!(total_accumulated_transaction_fee, 10000u64);

    state.single_transaction_should_work(NATIVE_TOKEN_ID, 120_000u64);

    let total_balances = state
        .world
        .query()
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .total_balances(NATIVE_TOKEN_ID)
        .returns(ReturnsResult)
        .run();

    assert_eq!(total_balances, 120000u64);
}

#[test]
fn create_refund_transaction_not_from_bridge_proxy_test() {
    let mut state = EsdtSafeTestState::new();
    state.multisig_deploy();
    state.safe_deploy();

    state.world.set_esdt_balance(
        MULTISIG_ADDRESS,
        b"TOKEN-WITH",
        BigUint::from(10_000_000u64),
    );

    let refund_info = sc_proxies::esdt_safe_proxy::RefundInfo::<StaticApi> {
        address: ManagedAddress::from(OWNER_ADDRESS.eval_to_array()),
        initial_batch_id: 1u64,
        initial_nonce: 1u64,
    };

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .create_refund_transaction(
            EthAddress {
                raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
            },
            OptionalValue::Some(refund_info.clone()),
        )
        .single_esdt(&TOKEN_ID.into(), 0, &BigUint::from(10u64))
        .returns(ExpectError(
            ERROR,
            "Only BridgeProxy SC can call this endpoint",
        ))
        .run();
}

#[test]
fn add_refund_batch_test() {
    let mut state = EsdtSafeTestState::new();
    state.multisig_deploy();
    state.safe_deploy();
    state.config_esdtsafe();

    state.world.set_esdt_balance(
        MULTI_TRANSFER_ADDRESS,
        b"ESDT-123",
        BigUint::from(300_000_000_000u64),
    );

    state
        .world
        .tx()
        .from(MULTISIG_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .set_eth_tx_gas_limit(1u64)
        .run();

    state.world.set_esdt_balance(
        ESDT_SAFE_ADDRESS,
        b"ESDT-123",
        BigUint::from(300_000_000_000u64),
    );

    let eth_tx = Transaction {
        from: ManagedBuffer::from(OWNER_ADDRESS_EXPR),
        to: ManagedBuffer::from(ESTD_SAFE_ADDRESS_EXPR),
        amount: BigUint::from(1_000_000u64),
        block_nonce: 0u64,
        nonce: 0u64,
        token_identifier: TokenIdentifier::from(NATIVE_TOKEN_ID),
        is_refund_tx: true,
    };

    let eth_tx2 = Transaction {
        from: ManagedBuffer::from(OWNER_ADDRESS_EXPR),
        to: ManagedBuffer::from(ESTD_SAFE_ADDRESS_EXPR),
        amount: BigUint::from(1_000_000u64),
        block_nonce: 0u64,
        nonce: 0u64,
        token_identifier: TokenIdentifier::from(NATIVE_TOKEN_ID),
        is_refund_tx: true,
    };

    let mut transfers: ManagedVec<StaticApi, Transaction<StaticApi>> = ManagedVec::new();
    transfers.push(eth_tx);
    transfers.push(eth_tx2);

    let payments = vec![
        EsdtTokenPayment::new(NATIVE_TOKEN_ID.into(), 0, BigUint::from(1_000_000u64)),
        EsdtTokenPayment::new(NATIVE_TOKEN_ID.into(), 0, BigUint::from(1_000_000u64)),
    ];
    let payment = EgldOrMultiEsdtPayment::MultiEsdt(payments.into());

    state.world.set_esdt_balance(
        ESDT_SAFE_ADDRESS,
        b"TOKEN-123456",
        BigUint::from(100_000_000_000u64),
    );

    state.add_refund_batch_tx_multiple_payment_should_fail(
        ESDT_SAFE_ADDRESS,
        ESDT_SAFE_ADDRESS,
        transfers.clone(),
        payment.clone(),
        "Invalid caller",
    );

    let empty_transfers = ManagedVec::<StaticApi, Transaction<StaticApi>>::new();

    state.world.set_esdt_balance(
        MULTI_TRANSFER_ADDRESS,
        b"TOKEN-123456",
        BigUint::from(10_000u64),
    );

    state.world.set_esdt_balance(
        MULTI_TRANSFER_ADDRESS,
        b"TOKEN-WITH",
        BigUint::from(10_000u64),
    );

    state.world.set_esdt_balance(
        MULTI_TRANSFER_ADDRESS,
        b"MINT-BURN-123456",
        BigUint::from(10_000u64),
    );

    state
        .world
        .tx()
        .from(MULTI_TRANSFER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .add_refund_batch(empty_transfers)
        .returns(ExpectError(ERROR, "Cannot refund with no payments"))
        .run();

    state.add_refund_batch_tx_single_payment_should_fail(
        MULTI_TRANSFER_ADDRESS,
        ESDT_SAFE_ADDRESS,
        transfers.clone(),
        TOKEN_WITH_BURN_ROLE,
        10u64,
        "Token identifiers do not match",
    );

    let payments_invalid: Vec<EsdtTokenPayment<StaticApi>> = vec![
        EsdtTokenPayment::new(NATIVE_TOKEN_ID.into(), 0, BigUint::from(1_000u64)),
        EsdtTokenPayment::new(NATIVE_TOKEN_ID.into(), 0, BigUint::from(100u64)),
    ];
    let payment_invalid = EgldOrMultiEsdtPayment::MultiEsdt(payments_invalid.into());

    state.add_refund_batch_tx_multiple_payment_should_fail(
        MULTI_TRANSFER_ADDRESS,
        ESDT_SAFE_ADDRESS,
        transfers.clone(),
        payment_invalid.clone(),
        "Amounts do not match",
    );

    let eth_tx3 = Transaction {
        from: ManagedBuffer::from(OWNER_ADDRESS_EXPR),
        to: ManagedBuffer::from(ESTD_SAFE_ADDRESS_EXPR),
        amount: BigUint::from(10_000u64),
        block_nonce: 0u64,
        nonce: 0u64,
        token_identifier: TokenIdentifier::from(MINT_BURN_TOKEN),
        is_refund_tx: true,
    };

    let mut transfers2: ManagedVec<StaticApi, Transaction<StaticApi>> = ManagedVec::new();
    transfers2.push(eth_tx3);

    state
        .world
        .tx()
        .from(MULTI_TRANSFER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .add_refund_batch(transfers2)
        .single_esdt(&MINT_BURN_TOKEN.into(), 0, &BigUint::from(10_000u64))
        .returns(ExpectError(ERROR, "Not enough minted tokens!"))
        .run();

    let eth_tx4 = Transaction {
        from: ManagedBuffer::from(OWNER_ADDRESS_EXPR),
        to: ManagedBuffer::from(ESTD_SAFE_ADDRESS_EXPR),
        amount: BigUint::from(1_000u64),
        block_nonce: 0u64,
        nonce: 0u64,
        token_identifier: TokenIdentifier::from(MINT_BURN_TOKEN),
        is_refund_tx: true,
    };

    let mut transfers3: ManagedVec<StaticApi, Transaction<StaticApi>> = ManagedVec::new();
    transfers3.push(eth_tx4);

    state
        .world
        .tx()
        .from(MULTI_TRANSFER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .add_refund_batch(transfers3.clone())
        .single_esdt(&MINT_BURN_TOKEN.into(), 0, &BigUint::from(1_000u64))
        .returns(ExpectError(ERROR, "Cannot do the burn action!"))
        .run();

    state.world.set_esdt_local_roles(
        ESDT_SAFE_ADDRESS,
        b"MINT-BURN-123456",
        &[EsdtLocalRole::Mint, EsdtLocalRole::Burn],
    );

    state
        .world
        .tx()
        .from(MULTI_TRANSFER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .add_refund_batch(transfers3)
        .single_esdt(&MINT_BURN_TOKEN.into(), 0, &BigUint::from(1_000u64))
        .run();

    state
        .world
        .tx()
        .from(MULTI_TRANSFER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .add_refund_batch(transfers)
        .egld_or_multi_esdt(payment)
        .returns(ReturnsResult)
        .run();

    let result = state
        .world
        .query()
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .get_batch_status(1u64)
        .returns(ReturnsResult)
        .run();

    if let BatchStatus::PartiallyFull {
        end_block_nonce,
        tx_ids,
    } = result
    {
        assert!(!tx_ids.is_empty(), "tx_ids should not be empty");
        let expected_tx_ids = vec![1u64, 2u64, 3u64];
        let tx_ids_vec: Vec<u64> = tx_ids.into_iter().collect();
        assert_eq!(
            tx_ids_vec, expected_tx_ids,
            "tx_ids do not match expected values"
        );
    } else {
        panic!("Expected BatchStatus::PartiallyFull, got {:?}", result);
    }
}

#[test]
fn claim_refund_test() {
    let mut state = EsdtSafeTestState::new();
    state.multisig_deploy();
    state.safe_deploy();
    state.config_esdtsafe();

    let mut tx_statuses = MultiValueEncoded::<StaticApi, TransactionStatus>::new();
    tx_statuses.push(TransactionStatus::Rejected);

    state
        .esdt_raw_transaction()
        .claim_refund(TOKEN_ID)
        .with_result(ExpectStatus(ERROR))
        .returns(ExpectError(ERROR, "Nothing to refund"))
        .run();

    state
        .world
        .tx()
        .from(MULTISIG_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .set_eth_tx_gas_limit(100u64)
        .run();

    state.world.set_esdt_local_roles(
        ESDT_SAFE_ADDRESS,
        b"MINT-BURN-123456",
        &[EsdtLocalRole::Mint, EsdtLocalRole::Burn],
    );

    state.single_transaction_should_work(MINT_BURN_TOKEN, 1000u64);
    state.set_transaction_batch_status_should_work(1, tx_statuses.clone());

    let opt_tokens: OptionalValue<MultiValueEncoded<StaticApi, TokenIdentifier<StaticApi>>> =
        OptionalValue::None;
    let refund = state
        .world
        .query()
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .get_refund_amounts(MULTISIG_ADDRESS, opt_tokens.clone())
        .returns(ReturnsResult)
        .run();

    let (token_id, amount) = refund.into_iter().next().unwrap().into_tuple();
    assert_eq!(token_id, TokenIdentifier::from(MINT_BURN_TOKEN));
    assert_eq!(amount, BigUint::from(1000u64));

    let claim_refund = state
        .esdt_raw_transaction()
        .claim_refund(MINT_BURN_TOKEN)
        .returns(ReturnsResult)
        .run();

    assert_eq!(token_id, claim_refund.token_identifier);
    assert_eq!(amount, claim_refund.amount);

    let refund_after = state
        .world
        .query()
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .get_refund_amounts(OWNER_ADDRESS, opt_tokens)
        .returns(ReturnsResult)
        .run();
    assert!(refund_after.is_empty());

    state
        .esdt_raw_transaction()
        .claim_refund(MINT_BURN_TOKEN)
        .returns(ExpectError(ERROR, "Nothing to refund"))
        .run();
}

#[test]
fn withdraw_refund_fees_for_ethereum_test() {
    let mut state = EsdtSafeTestState::new();
    state.multisig_deploy();
    state.safe_deploy();
    state.config_esdtsafe();

    state
        .world
        .tx()
        .from(MULTISIG_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .set_eth_tx_gas_limit(1000u64)
        .run();

    state
        .world
        .tx()
        .from(MULTISIG_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .set_default_price_per_gas_unit(TOKEN_ID, 10u64)
        .run();

    state
        .world
        .tx()
        .from(MULTISIG_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .withdraw_refund_fees_for_ethereum(TOKEN_ID, MULTISIG_ADDRESS)
        .returns(ExpectError(ERROR, "There are no fees to withdraw"))
        .run();

    let refund_tx = Transaction {
        from: ManagedBuffer::from(OWNER_ADDRESS_EXPR),
        to: ManagedBuffer::from(ESTD_SAFE_ADDRESS_EXPR),
        amount: BigUint::from(1_000_000u64),
        block_nonce: 0u64,
        nonce: 0u64,
        token_identifier: TokenIdentifier::from(TOKEN_ID),
        is_refund_tx: true,
    };

    let mut transfers: ManagedVec<StaticApi, Transaction<StaticApi>> = ManagedVec::new();
    transfers.push(refund_tx.clone());

    let payment = EsdtTokenPayment::new(TOKEN_ID.into(), 0, BigUint::from(1_000_000u64));
    let payments = vec![payment];
    let payment = EgldOrMultiEsdtPayment::MultiEsdt(payments.into());

    state.world.set_esdt_balance(
        MULTI_TRANSFER_ADDRESS,
        b"TOKEN-123456",
        BigUint::from(1_000_000u64),
    );

    state
        .world
        .tx()
        .from(MULTI_TRANSFER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .add_refund_batch(transfers.clone())
        .egld_or_multi_esdt(payment.clone())
        .returns(ReturnsResult)
        .run();

    let refund_fees = state
        .world
        .query()
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .get_refund_fees_for_ethereum(TOKEN_ID)
        .returns(ReturnsResult)
        .run();

    assert!(
        refund_fees > BigUint::zero(),
        "refund_fees_for_ethereum should be greater than zero"
    );

    state
        .world
        .tx()
        .from(MULTISIG_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .withdraw_refund_fees_for_ethereum(TOKEN_ID, MULTISIG_ADDRESS)
        .returns(ReturnsResult)
        .run();

    let refund_fees_after = state
        .world
        .query()
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .get_refund_fees_for_ethereum(TOKEN_ID)
        .returns(ReturnsResult)
        .run();

    assert_eq!(
        refund_fees_after,
        BigUint::zero(),
        "refund_fees_for_ethereum should be zero after withdrawal"
    );
}

#[test]
fn withdraw_transaction_fees_test() {
    let mut state = EsdtSafeTestState::new();
    state.multisig_deploy();
    state.safe_deploy();
    state.config_esdtsafe();

    state
        .world
        .tx()
        .from(MULTISIG_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .set_eth_tx_gas_limit(1000u64)
        .run();

    state
        .world
        .tx()
        .from(MULTISIG_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .withdraw_transaction_fees(TOKEN_ID, MULTISIG_ADDRESS)
        .returns(ExpectError(ERROR, "There are no fees to withdraw"))
        .run();

    state
        .world
        .tx()
        .from(MULTISIG_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .set_default_price_per_gas_unit(TOKEN_ID, 10u64)
        .run();

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .create_transaction(
            EthAddress::zero(),
            OptionalValue::<BigUint<StaticApi>>::None,
        )
        .single_esdt(&TOKEN_ID.into(), 0, &BigUint::from(1_000_000u64))
        .returns(ExpectError(ERROR, "Cannot send to an empty address"))
        .run();

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .create_transaction(
            EthAddress {
                raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
            },
            OptionalValue::<BigUint<StaticApi>>::None,
        )
        .single_esdt(&TOKEN_ID.into(), 0, &BigUint::from(1_000_000u64))
        .returns(ReturnsResult)
        .run();

    let transaction_fees = state
        .world
        .query()
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .get_transaction_fees(TOKEN_ID)
        .returns(ReturnsResult)
        .run();

    assert!(
        transaction_fees > BigUint::zero(),
        "accumulated_transaction_fees should be greater than zero"
    );

    state
        .world
        .tx()
        .from(MULTISIG_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .withdraw_transaction_fees(TOKEN_ID, MULTISIG_ADDRESS)
        .returns(ReturnsResult)
        .run();

    let transaction_fees_after = state
        .world
        .query()
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .get_transaction_fees(TOKEN_ID)
        .returns(ReturnsResult)
        .run();

    assert_eq!(
        transaction_fees_after,
        BigUint::zero(),
        "accumulated_transaction_fees should be zero after withdrawal"
    );
}

#[test]
fn esdt_safe_reduce_max_tx_batch_size_test() {
    let mut state = EsdtSafeTestState::new();
    state.multisig_deploy();
    state.safe_deploy();

    state.world.set_esdt_balance(
        MULTISIG_ADDRESS,
        b"TOKEN-WITH",
        BigUint::from(10_000_000u64),
    );

    state.config_esdtsafe();

    // Batch size is default 10
    state
        .world
        .tx()
        .from(BRIDGED_TOKENS_WRAPPER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .create_transaction(
            EthAddress {
                raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
            },
            OptionalValue::<BigUint<StaticApi>>::None,
        )
        .single_esdt(&TOKEN_ID.into(), 0, &BigUint::from(10u64))
        .run();

    state
        .world
        .tx()
        .from(BRIDGED_TOKENS_WRAPPER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .create_transaction(
            EthAddress {
                raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
            },
            OptionalValue::<BigUint<StaticApi>>::None,
        )
        .single_esdt(&TOKEN_ID.into(), 0, &BigUint::from(10u64))
        .run();

    state
        .world
        .tx()
        .from(BRIDGED_TOKENS_WRAPPER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .create_transaction(
            EthAddress {
                raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
            },
            OptionalValue::<BigUint<StaticApi>>::None,
        )
        .single_esdt(&TOKEN_ID.into(), 0, &BigUint::from(10u64))
        .run();

    state
        .world
        .tx()
        .from(BRIDGED_TOKENS_WRAPPER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .create_transaction(
            EthAddress {
                raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
            },
            OptionalValue::<BigUint<StaticApi>>::None,
        )
        .single_esdt(&TOKEN_ID.into(), 0, &BigUint::from(10u64))
        .run();

    //get_batch_status
    let batch_id = 1u64;
    let batch_status = state
        .world
        .tx()
        .from(BRIDGED_TOKENS_WRAPPER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .get_batch_status(batch_id)
        .returns(ReturnsResult)
        .run();

    assert_eq!(
        batch_status,
        BatchStatus::PartiallyFull {
            end_block_nonce: 100,
            tx_ids: ManagedVec::from(vec![1u64, 2u64, 3u64, 4u64])
        },
        "Incorrect batch status"
    );

    let new_max_batch_status = 2usize;
    state
        .world
        .tx()
        .from(MULTISIG_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .set_max_tx_batch_size(new_max_batch_status)
        .run();

    state
        .world
        .tx()
        .from(BRIDGED_TOKENS_WRAPPER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .create_transaction(
            EthAddress {
                raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
            },
            OptionalValue::<BigUint<StaticApi>>::None,
        )
        .single_esdt(&TOKEN_ID.into(), 0, &BigUint::from(10u64))
        .run();

    //First batch should be full
    let batch_id = 1u64;
    let batch_status = state
        .world
        .tx()
        .from(BRIDGED_TOKENS_WRAPPER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .get_batch_status(batch_id)
        .returns(ReturnsResult)
        .run();

    assert_eq!(
        batch_status,
        BatchStatus::WaitingForSignatures,
        "Incorrect batch status"
    );

    //A new batch should be created
    let batch_id = 2u64;
    let batch_status = state
        .world
        .tx()
        .from(BRIDGED_TOKENS_WRAPPER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .get_batch_status(batch_id)
        .returns(ReturnsResult)
        .run();

    assert_eq!(
        batch_status,
        BatchStatus::PartiallyFull {
            end_block_nonce: 100,
            tx_ids: ManagedVec::from(vec![5u64])
        },
        "Incorrect batch status"
    );
}
