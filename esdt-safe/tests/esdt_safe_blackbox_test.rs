use esdt_safe::*;
use eth_address::EthAddress;

use multiversx_sc_scenario::imports::*;
use transaction::transaction_status::TransactionStatus;
use transaction::Transaction;

const OWNER_ADDRESS: TestAddress = TestAddress::new("owner");
const OWNER_ADDRESS_EXPR: &str = "address:owner";

const ESDT_SAFE_ADDRESS: TestSCAddress = TestSCAddress::new("esdt-safe");
const ESTD_SAFE_ADDRESS_EXPR: &str = "sc:esdt-safe";
const MULTI_TRANSFER_ADDRESS: TestSCAddress = TestSCAddress::new("multi-transfer");

const ESDT_SAFE_CODE_PATH: MxscPath = MxscPath::new("output/esdt-safe.mxsc.json");
const MULTI_TRANSFER_CODE_PATH: MxscPath =
    MxscPath::new("../multi-transfer/multi-transfer-esdt.mxsc.json");
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
            .esdt_roles(ETH_TOKEN_ID.clone(), roles.clone())
            .code(MULTI_TRANSFER_CODE_PATH)
            .nonce(1)
            .esdt_balance(ETH_TOKEN_ID, 1001u64)
            .esdt_balance(TOKEN_ID, 1000000000u64)
            .owner(OWNER_ADDRESS);

        world
            .account(ESDT_SAFE_ADDRESS)
            .esdt_roles(ETH_TOKEN_ID.clone(), roles)
            .code(ESDT_SAFE_CODE_PATH)
            .owner(OWNER_ADDRESS);

        Self { world }
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
            &BigUint::from(10_000_000u64),
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
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .set_mint_balances(TOKEN_ID, BigUint::from(MINTED_AMOUNT))
            .run();

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

    fn single_transaction(
        &mut self,
        from_address: TestAddress,
        to_address: TestSCAddress,
        token_id: TestTokenIdentifier,
        amount: u64,
        expected_status: u64,
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
            .with_result(ExpectStatus(expected_status))
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

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .create_transaction(EthAddress::zero())
        .egld_or_single_esdt(
            &EgldOrEsdtTokenIdentifier::esdt(TOKEN_ID),
            0,
            &BigUint::from(10_000_000u64),
        )
        .with_result(ExpectStatus(4))
        .returns(ExpectError(4u64, "Cannot create transaction while paused"))
        .run();

    state.config_esdtsafe();
    state.set_balances();

    //transaction fees cost more than the entire bridged amount
    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .create_transaction(EthAddress::zero())
        .egld_or_single_esdt(
            &EgldOrEsdtTokenIdentifier::esdt(TOKEN_ID),
            0,
            &BigUint::from(0u64),
        )
        .with_result(ExpectStatus(4))
        //.returns(ExpectError(4u64, "Transaction fees cost more than the entire bridged amount"))
        .run();

    // not enough minted tokens
    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .create_transaction(EthAddress::zero())
        .egld_or_single_esdt(
            &EgldOrEsdtTokenIdentifier::esdt(TOKEN_ID),
            0,
            &BigUint::from(200_000_000_000u64),
        )
        .with_result(ExpectStatus(4))
        //.returns(ExpectError(10u64, "Not enough minted tokens!"))
        .run();

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .create_transaction(EthAddress::zero())
        .egld_or_single_esdt(
            &EgldOrEsdtTokenIdentifier::esdt(TOKEN_ID),
            0,
            &BigUint::from(100000000000000u64),
        )
        .with_result(ExpectStatus(10))
        //.with_result(ExpectMessage("insufficient funds"))
        .run();

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .create_transaction(EthAddress::zero())
        .egld_or_single_esdt(
            &EgldOrEsdtTokenIdentifier::esdt(NON_WHITELISTED_TOKEN),
            0,
            &BigUint::from(100u64),
        )
        .with_result(ExpectStatus(4))
        //.with_result(ExpectMessage(expected_error))
        //.returns(ExpectError(4u64, "Token not whitelisted"))
        .run();

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .create_transaction(EthAddress::zero())
        .egld_or_single_esdt(
            &EgldOrEsdtTokenIdentifier::esdt(TOKEN_ID),
            0,
            &BigUint::from(10_000_000u64),
        )
        .run();
}

#[test]
fn set_transaction_batch_status_test() {
    let mut state = EsdtSafeTestState::new();
    state.safe_deploy();
    state.config_esdtsafe();

    let mut tx_statuses = MultiValueEncoded::<StaticApi, TransactionStatus>::new();
    tx_statuses.push(TransactionStatus::Executed);
    let mut tx_statuses_invalid = MultiValueEncoded::<StaticApi, TransactionStatus>::new();
    tx_statuses_invalid.push(TransactionStatus::Executed);
    tx_statuses_invalid.push(TransactionStatus::Pending);

    state.set_balances();

    state.single_transaction(OWNER_ADDRESS, ESDT_SAFE_ADDRESS, TOKEN_ID, 10000u64, 0);

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .set_transaction_batch_status(5u32, tx_statuses.clone())
        .with_result(ExpectStatus(4))
        //.returns(ExpectError(4u64, "Batches must be processed in order"))
        .run();

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .set_transaction_batch_status(5u32, tx_statuses_invalid.clone())
        .with_result(ExpectStatus(4))
        //.returns(ExpectError(4u64, "Invalid number of statuses provided"))
        .run();

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .set_transaction_batch_status(1u32, tx_statuses_invalid.clone())
        .with_result(ExpectStatus(4))
        //.returns(ExpectError(4u64, "Transaction status may only be set to Executed or Rejected"))
        .run();

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .set_transaction_batch_status(1u32, tx_statuses)
        .run();

    let result = state
        .world
        .query()
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .get_batch_status(1u64)
        .returns(ReturnsResult)
        .run();
    //assert_eq!(result, tx_statuses);
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

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .add_refund_batch(transfers.clone())
        .egld_or_multi_esdt(payment.clone())
        .with_result(ExpectStatus(4))
        //.returns(ExpectError(4u64, "Invalid caller"))
        .run();

    let empty_transfers = ManagedVec::<StaticApi, Transaction<StaticApi>>::new();
    state
        .world
        .tx()
        .from(MULTI_TRANSFER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .add_refund_batch(empty_transfers)
        .with_result(ExpectStatus(4))
        //.returns(ExpectError(4u64, "Cannot refund with no payments"))
        .run();

    state
        .world
        .tx()
        .from(MULTI_TRANSFER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .add_refund_batch(transfers.clone())
        .egld_or_single_esdt(
            &EgldOrEsdtTokenIdentifier::esdt(ETH_TOKEN_ID),
            0,
            &BigUint::from(10u64),
        )
        .with_result(ExpectStatus(4))
        //.returns(ExpectError(4u64, "Token identifiers do not match"))
        .run();

    let payments_invalid = vec![
        EsdtTokenPayment::new(TOKEN_ID.into(), 0, BigUint::from(1_000u64)),
        EsdtTokenPayment::new(TOKEN_ID.into(), 0, BigUint::from(100_000u64)),
    ];
    let payment_invalid = EgldOrMultiEsdtPayment::MultiEsdt(payments_invalid.into());

    state
        .world
        .tx()
        .from(MULTI_TRANSFER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .add_refund_batch(transfers.clone())
        .egld_or_multi_esdt(payment_invalid.clone())
        .with_result(ExpectStatus(4))
        //.returns(ExpectError(4u64, "Amounts do not match"))
        .run();

    state
        .world
        .tx()
        .from(MULTI_TRANSFER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .add_refund_batch(transfers.clone())
        .egld_or_multi_esdt(payment.clone())
        .run();

    let result = state
        .world
        .query()
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .get_batch_status(1u64)
        .returns(ReturnsResult)
        .run();

    //assert_eq!(result, tx_statuses);
}

#[test]
fn init_supply_test() {
    let mut state = EsdtSafeTestState::new();
    state.safe_deploy();
    state.config_esdtsafe();

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .init_supply(NON_WHITELISTED_TOKEN, BigUint::from(10_000u64))
        .egld_or_single_esdt(
            &EgldOrEsdtTokenIdentifier::esdt(NATIVE_TOKEN_ID),
            0,
            &BigUint::from(10_000u64),
        )
        .with_result(ExpectStatus(4))
        //.returns(ExpectError(4u64, "Invalid token ID"))
        .run();

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .init_supply(NATIVE_TOKEN_ID, BigUint::from(1_000u64))
        .egld_or_single_esdt(
            &EgldOrEsdtTokenIdentifier::esdt(NATIVE_TOKEN_ID),
            0,
            &BigUint::from(10_000u64),
        )
        .with_result(ExpectStatus(4))
        //.returns(ExpectError(4u64, "Invalid amount))
        .run();

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .init_supply(TOKEN_ID, BigUint::from(10_000u64))
        .egld_or_single_esdt(
            &EgldOrEsdtTokenIdentifier::esdt(TOKEN_ID),
            0,
            &BigUint::from(10_000u64),
        )
        .with_result(ExpectStatus(4))
        //.returns(ExpectError(4u64, "Cannot init for non native tokens"))
        .run();

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .init_supply(NATIVE_TOKEN_ID, BigUint::from(10_000u64))
        .egld_or_single_esdt(
            &EgldOrEsdtTokenIdentifier::esdt(NATIVE_TOKEN_ID),
            0,
            &BigUint::from(10_000u64),
        )
        .run();
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
        //.returns(ExpectError(4u64, "Nothing to refund"))
        .run();

    //create transaction
    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .create_transaction(EthAddress::zero())
        .egld_or_single_esdt(
            &EgldOrEsdtTokenIdentifier::esdt(TOKEN_ID),
            0,
            &BigUint::from(10_000_000u64),
        )
        .run();

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .set_transaction_batch_status(1u32, tx_statuses)
        .run();

    let result = state
        .world
        .query()
        //.from(OWNER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .get_refund_amounts(OWNER_ADDRESS)
        .returns(ReturnsResult)
        .run();

    println!("result: {:?}", result);

    let result2 = state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .claim_refund(TOKEN_ID)
        .returns(ReturnsResult)
        .run();

    println!("result2: {:?}", result2);

    let result3 = state
        .world
        .query()
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .get_refund_amounts(OWNER_ADDRESS)
        .returns(ReturnsResult)
        .run();

    println!("result3: {:?}", result3);
}
