#![allow(unused)]
use esdt_safe::*;
use eth_address::EthAddress;

use multiversx_sc_scenario::imports::*;
use token_module::TokenModule;
use transaction::transaction_status::TransactionStatus;
use transaction::Transaction;
use tx_batch_module::BatchStatus;

const OWNER_ADDRESS: TestAddress = TestAddress::new("owner");
const OWNER_ADDRESS_EXPR: &str = "address:owner";

const ESDT_SAFE_ADDRESS: TestSCAddress = TestSCAddress::new("esdt-safe");
const ESTD_SAFE_ADDRESS_EXPR: &str = "sc:esdt-safe";
const MULTI_TRANSFER_ADDRESS: TestSCAddress = TestSCAddress::new("multi-transfer");

const ESDT_SAFE_CODE_PATH: MxscPath = MxscPath::new("output/esdt-safe.mxsc.json");
const MULTI_TRANSFER_CODE_PATH: MxscPath =
    MxscPath::new("../multi-transfer-esdt/multi-transfer-esdt.mxsc.json");
const TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("TOKEN-123456");
const NON_WHITELISTED_TOKEN: TestTokenIdentifier =
    TestTokenIdentifier::new("NON-WHITELISTED-123456");
const NATIVE_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("ESDT-123");
const ETH_TX_GAS_LIMIT: u64 = 150_000;
const ETH_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("ETH-123456");
const MINTED_AMOUNT: u64 = 100_000_000_000;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();
    blockchain.register_contract(ESDT_SAFE_CODE_PATH, esdt_safe::ContractBuilder);
    blockchain
}

struct EsdtSafeTestState {
    world: ScenarioWorld,
    esdt_safe_whitebox: WhiteboxContract<ContractObj<DebugApi>>,
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
            .esdt_balance(TestTokenIdentifier::new("NEW-TOKEN-123456"), 1000000000u64)
            .esdt_balance(NATIVE_TOKEN_ID, 100_000u64);

        let roles = vec![
            "ESDTRoleLocalMint".to_string(),
            "ESDTRoleLocalBurn".to_string(),
        ];
        world
            .account(MULTI_TRANSFER_ADDRESS)
            .esdt_roles(ETH_TOKEN_ID, roles.clone())
            .code(MULTI_TRANSFER_CODE_PATH)
            .nonce(1)
            .esdt_balance(ETH_TOKEN_ID, 1001u64)
            .esdt_balance(TOKEN_ID, 1000000000u64)
            .owner(OWNER_ADDRESS);

        world
            .account(ESDT_SAFE_ADDRESS)
            .esdt_roles(ETH_TOKEN_ID, roles)
            .code(ESDT_SAFE_CODE_PATH)
            .owner(OWNER_ADDRESS);

        let esdt_safe_whitebox =
            WhiteboxContract::new(ESTD_SAFE_ADDRESS_EXPR, esdt_safe::contract_obj);

        Self {
            world,
            esdt_safe_whitebox,
        }
    }

    fn safe_deploy(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .upgrade(
                ManagedAddress::zero(),
                MULTI_TRANSFER_ADDRESS.to_address(),
                ETH_TX_GAS_LIMIT,
            )
            .code(ESDT_SAFE_CODE_PATH)
            .run();

        self
    }

    fn config_esdtsafe(&mut self) {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .unpause_endpoint()
            .run();
        self.world.set_esdt_balance(
            ESDT_SAFE_ADDRESS,
            b"TOKEN-123456",
            BigUint::from(10_000_000u64),
        );
        self.world.set_esdt_local_roles(
            ESDT_SAFE_ADDRESS,
            b"TOKEN-123456",
            &[EsdtLocalRole::Mint, EsdtLocalRole::Burn],
        );

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .add_token_to_whitelist(
                TOKEN_ID,
                "TOKEN",
                true,
                false,
                OptionalValue::Some(BigUint::from(0u64)),
            )
            .run();

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .add_token_to_whitelist(
                NATIVE_TOKEN_ID,
                "NATIVE",
                false,
                true,
                OptionalValue::Some(BigUint::from(0u64)),
            )
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
    }

    fn set_balances(&mut self) {
        self.world.whitebox_call(
            &self.esdt_safe_whitebox,
            ScCallStep::new().from(OWNER_ADDRESS),
            |sc| {
                sc.mint_balances(&TokenIdentifier::from(TOKEN_ID))
                    .set(BigUint::from(MINTED_AMOUNT))
            },
        );

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .set_total_balances(TOKEN_ID, BigUint::from(MINTED_AMOUNT))
            .run();
    }

    fn multiple_transactions(&mut self) -> ManagedVec<StaticApi, Transaction<StaticApi>> {
        let eth_tx = Transaction {
            from: ManagedBuffer::from(OWNER_ADDRESS_EXPR),
            to: ManagedBuffer::from(ESTD_SAFE_ADDRESS_EXPR),
            amount: BigUint::from(100_000u64),
            block_nonce: 0u64,
            nonce: 0u64,
            token_identifier: TokenIdentifier::from(TOKEN_ID),
            is_refund_tx: true,
        };

        let eth_tx2 = Transaction {
            from: ManagedBuffer::from(OWNER_ADDRESS_EXPR),
            to: ManagedBuffer::from(ESTD_SAFE_ADDRESS_EXPR),
            amount: BigUint::from(100_000u64),
            block_nonce: 0u64,
            nonce: 0u64,
            token_identifier: TokenIdentifier::from(TOKEN_ID),
            is_refund_tx: true,
        };

        let mut transfers: ManagedVec<StaticApi, Transaction<StaticApi>> = ManagedVec::new();
        transfers.push(eth_tx);
        transfers.push(eth_tx2);

        transfers
    }

    fn single_transaction_should_fail(
        &mut self,
        from_address: TestAddress,
        to_address: TestSCAddress,
        token_id: TestTokenIdentifier,
        amount: u64,
        expected_error: &str,
    ) {
        self.world
            .tx()
            .from(from_address)
            .to(to_address)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .create_transaction(EthAddress::zero())
            .egld_or_single_esdt(
                &EgldOrEsdtTokenIdentifier::esdt(token_id),
                0,
                &BigUint::from(amount),
            )
            .returns(ExpectError(4u64, expected_error))
            .run();
    }

    fn single_transaction_should_work(
        &mut self,
        from_address: TestAddress,
        to_address: TestSCAddress,
        token_id: TestTokenIdentifier,
        amount: u64,
    ) {
        self.world
            .tx()
            .from(from_address)
            .to(to_address)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .create_transaction(EthAddress::zero())
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
        from_address: TestAddress,
        to_address: TestSCAddress,
        batch_id: u32,
        statuses: MultiValueEncoded<StaticApi, TransactionStatus>,
        expected_status: u64,
        expected_error: &str,
    ) {
        let mut tx = self
            .world
            .tx()
            .from(from_address)
            .to(to_address)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .set_transaction_batch_status(batch_id, statuses)
            .returns(ExpectError(expected_status, expected_error))
            .run();
    }

    fn set_transaction_batch_status_should_work(
        &mut self,
        from_address: TestAddress,
        to_address: TestSCAddress,
        batch_id: u32,
        statuses: MultiValueEncoded<StaticApi, TransactionStatus>,
    ) {
        let mut tx = self
            .world
            .tx()
            .from(from_address)
            .to(to_address)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .set_transaction_batch_status(batch_id, statuses)
            .returns(ReturnsResult)
            .run();
    }

    fn add_refund_batch_tx_multiple_payment_should_fail(
        &mut self,
        from_address: TestSCAddress,
        to_address: TestSCAddress,
        transfers: ManagedVec<StaticApi, Transaction<StaticApi>>,
        payment: EgldOrMultiEsdtPayment<StaticApi>,
        expected_status: u64,
        expected_error: &str,
    ) {
        self.world
            .tx()
            .from(from_address)
            .to(to_address)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .add_refund_batch(transfers)
            .egld_or_multi_esdt(payment)
            .returns(ExpectError(expected_status, expected_error))
            .run();
    }

    fn add_refund_batch_tx_multiple_payment_should_work(
        &mut self,
        from_address: TestSCAddress,
        to_address: TestSCAddress,
        transfers: ManagedVec<StaticApi, Transaction<StaticApi>>,
        payment: EgldOrMultiEsdtPayment<StaticApi>,
    ) {
        self.world
            .tx()
            .from(from_address)
            .to(to_address)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .add_refund_batch(transfers)
            .egld_or_multi_esdt(payment)
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
        expected_status: u64,
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
            .returns(ExpectError(expected_status, expected_error))
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
            .returns(ExpectError(expected_status, expected_error))
            .run();
    }

    fn init_supply_should_work(
        &mut self,
        from_address: TestAddress,
        to_address: TestSCAddress,
        token_id: TestTokenIdentifier,
        tx_token_id: TestTokenIdentifier,
        tx_amount: u64,
        amount: u64,
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
            .returns(ReturnsResult)
            .run();
    }
}

#[test]
fn config_test() {
    let mut state = EsdtSafeTestState::new();
    state.safe_deploy();
    state.config_esdtsafe();
}

#[test]
fn create_transaction_test() {
    let mut state = EsdtSafeTestState::new();
    state.safe_deploy();

    state.single_transaction_should_fail(
        OWNER_ADDRESS,
        ESDT_SAFE_ADDRESS,
        TOKEN_ID,
        10_000_000u64,
        "Cannot create transaction while paused",
    );

    state.config_esdtsafe();
    state.set_balances();

    state.single_transaction_should_fail(
        OWNER_ADDRESS,
        ESDT_SAFE_ADDRESS,
        TOKEN_ID,
        0u64,
        "Transaction fees cost more than the entire bridged amount",
    );

    state.single_transaction_should_fail(
        OWNER_ADDRESS,
        ESDT_SAFE_ADDRESS,
        TOKEN_ID,
        200_000_000_000u64,
        "Not enough minted tokens!",
    );

    state.single_transaction_should_fail(
        OWNER_ADDRESS,
        ESDT_SAFE_ADDRESS,
        NON_WHITELISTED_TOKEN,
        100u64,
        "Token not in whitelist",
    );

    state.single_transaction_should_work(OWNER_ADDRESS, ESDT_SAFE_ADDRESS, TOKEN_ID, 10_000_000u64);
}

#[test]
fn set_transaction_batch_status_test() {
    let mut state = EsdtSafeTestState::new();
    state.safe_deploy();
    state.config_esdtsafe();

    let mut tx_statuses = MultiValueEncoded::<StaticApi, TransactionStatus>::new();
    tx_statuses.push(TransactionStatus::Executed);
    let mut tx_multiple_statuses = MultiValueEncoded::<StaticApi, TransactionStatus>::new();
    tx_multiple_statuses.push(TransactionStatus::Executed);
    tx_multiple_statuses.push(TransactionStatus::Pending);
    let mut tx_statuses_invalid = MultiValueEncoded::<StaticApi, TransactionStatus>::new();
    tx_statuses_invalid.push(TransactionStatus::Pending);

    state.set_balances();

    state.single_transaction_should_work(OWNER_ADDRESS, ESDT_SAFE_ADDRESS, TOKEN_ID, 10000u64);

    state.set_transaction_batch_status_should_fail(
        OWNER_ADDRESS,
        ESDT_SAFE_ADDRESS,
        5u32,
        tx_statuses.clone(),
        4,
        "Batches must be processed in order",
    );

    state.set_transaction_batch_status_should_fail(
        OWNER_ADDRESS,
        ESDT_SAFE_ADDRESS,
        1u32,
        tx_multiple_statuses.clone(),
        4,
        "Invalid number of statuses provided",
    );

    state.set_transaction_batch_status_should_fail(
        OWNER_ADDRESS,
        ESDT_SAFE_ADDRESS,
        1u32,
        tx_statuses_invalid.clone(),
        4,
        "Transaction status may only be set to Executed or Rejected",
    );

    state.set_transaction_batch_status_should_work(
        OWNER_ADDRESS,
        ESDT_SAFE_ADDRESS,
        1u32,
        tx_statuses.clone(),
    );

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
fn add_refund_batch_test() {
    let mut state = EsdtSafeTestState::new();
    state.safe_deploy();
    state.config_esdtsafe();

    let transfers = state.multiple_transactions();
    let payments = vec![
        EsdtTokenPayment::new(TOKEN_ID.into(), 0, BigUint::from(100_000u64)),
        EsdtTokenPayment::new(TOKEN_ID.into(), 0, BigUint::from(100_000u64)),
    ];
    let payment = EgldOrMultiEsdtPayment::MultiEsdt(payments.into());

    state.add_refund_batch_tx_multiple_payment_should_fail(
        ESDT_SAFE_ADDRESS,
        ESDT_SAFE_ADDRESS,
        transfers.clone(),
        payment.clone(),
        4,
        "Invalid caller",
    );

    let empty_transfers = ManagedVec::<StaticApi, Transaction<StaticApi>>::new();

    state
        .world
        .tx()
        .from(MULTI_TRANSFER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .add_refund_batch(empty_transfers)
        .returns(ExpectError(4u64, "Cannot refund with no payments"))
        .run();

    state.add_refund_batch_tx_single_payment_should_fail(
        MULTI_TRANSFER_ADDRESS,
        ESDT_SAFE_ADDRESS,
        transfers.clone(),
        ETH_TOKEN_ID,
        10u64,
        4,
        "Token identifiers do not match",
    );

    let payments_invalid = vec![
        EsdtTokenPayment::new(TOKEN_ID.into(), 0, BigUint::from(1_000u64)),
        EsdtTokenPayment::new(TOKEN_ID.into(), 0, BigUint::from(100_000u64)),
    ];
    let payment_invalid = EgldOrMultiEsdtPayment::MultiEsdt(payments_invalid.into());

    state.add_refund_batch_tx_multiple_payment_should_fail(
        MULTI_TRANSFER_ADDRESS,
        ESDT_SAFE_ADDRESS,
        transfers.clone(),
        payment_invalid.clone(),
        4,
        "Amounts do not match",
    );
    state.add_refund_batch_tx_multiple_payment_should_work(
        MULTI_TRANSFER_ADDRESS,
        ESDT_SAFE_ADDRESS,
        transfers,
        payment,
    );

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
        let expected_tx_ids = vec![1u64, 2u64];
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
        4u64,
        "Invalid token ID",
    );

    state.init_supply_should_fail(
        OWNER_ADDRESS,
        ESDT_SAFE_ADDRESS,
        TOKEN_ID,
        TOKEN_ID,
        10_000u64,
        10_000u64,
        4u64,
        "Cannot init for non native tokens",
    );

    state.init_supply_should_work(
        OWNER_ADDRESS,
        ESDT_SAFE_ADDRESS,
        NATIVE_TOKEN_ID,
        NATIVE_TOKEN_ID,
        10_000u64,
        10_000u64,
    );

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
fn claim_refund_test() {
    let mut state = EsdtSafeTestState::new();
    state.safe_deploy();
    state.config_esdtsafe();
    state.set_balances();

    let mut tx_statuses = MultiValueEncoded::<StaticApi, TransactionStatus>::new();
    tx_statuses.push(TransactionStatus::Rejected);

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .claim_refund(TOKEN_ID)
        .with_result(ExpectStatus(4))
        .returns(ExpectError(4u64, "Nothing to refund"))
        .run();

    state.single_transaction_should_work(OWNER_ADDRESS, ESDT_SAFE_ADDRESS, TOKEN_ID, 10_000_000u64);

    state.set_transaction_batch_status_should_work(
        OWNER_ADDRESS,
        ESDT_SAFE_ADDRESS,
        1,
        tx_statuses.clone(),
    );

    let result = state
        .world
        .query()
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .get_refund_amounts(OWNER_ADDRESS)
        .returns(ReturnsResult)
        .run();

    let (token_id, amount) = result.into_iter().next().unwrap().into_tuple();
    assert_eq!(token_id, TokenIdentifier::from(TOKEN_ID));
    assert_eq!(amount, BigUint::from(10_000_000u64));

    let result2 = state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .claim_refund(TOKEN_ID)
        .returns(ReturnsResult)
        .run();

    assert_eq!(token_id, result2.token_identifier);
    assert_eq!(amount, result2.amount);

    let result3 = state
        .world
        .query()
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .get_refund_amounts(OWNER_ADDRESS)
        .returns(ReturnsResult)
        .run();
    assert!(result3.is_empty());
}
