#![allow(unused)]

use esdt_safe_proxy::EsdtSafeProxyMethods;
use multi_transfer_esdt::*;
use multiversx_sc::{
    api::{HandleConstraints, ManagedTypeApi},
    codec::{
        multi_types::{MultiValueVec, OptionalValue},
        Empty, TopEncode,
    },
    contract_base::ManagedSerializer,
    storage::mappers::SingleValue,
    types::{
        Address, BigUint, CodeMetadata, EgldOrEsdtTokenIdentifier, EgldOrMultiEsdtPayment,
        EsdtLocalRole, ManagedAddress, ManagedBuffer, ManagedByteArray, ManagedOption, ManagedVec,
        MultiValueEncoded, ReturnsNewManagedAddress, ReturnsResult, TestAddress, TestSCAddress,
        TestTokenIdentifier, TokenIdentifier,
    },
};
use multiversx_sc_modules::pause::ProxyTrait;
use multiversx_sc_scenario::{
    api::{StaticApi, VMHooksApi, VMHooksApiBackend},
    imports::*,
    scenario_format::interpret_trait::{InterpretableFrom, InterpreterContext},
    scenario_model::*,
    ContractInfo, DebugApi, ExpectError, ExpectValue, ScenarioTxRun, ScenarioWorld,
};

use eth_address::*;
use mock_proxies::mock_multisig_proxy;
use sc_proxies::{
    bridge_proxy_contract_proxy, bridged_tokens_wrapper_proxy, esdt_safe_proxy,
    multi_transfer_esdt_proxy,
};
use token_module::ProxyTrait as _;
use transaction::{transaction_status::TransactionStatus, CallData, EthTransaction, Transaction};
use tx_batch_module::BatchStatus;

const UNIVERSAL_TOKEN_IDENTIFIER: TestTokenIdentifier = TestTokenIdentifier::new("UNIV-abc123");
const BRIDGE_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("BRIDGE-123456");
const WRAPPED_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("WRAPPED-123456");
const TOKEN_TICKER: TestTokenIdentifier = TestTokenIdentifier::new("TOKEN");
const TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("TOKEN-123456");
const TOKEN_WITH_BURN_ROLE: TestTokenIdentifier = TestTokenIdentifier::new("TOKEN-WITH");
const TOKEN_WITHOUT_BURN_ROLE: TestTokenIdentifier = TestTokenIdentifier::new("TOKEN-WITH-OUT");

const NON_WHITELISTED_TOKEN: TestTokenIdentifier =
    TestTokenIdentifier::new("NON-WHITELISTED-123456");
const OWNER_ADDRESS_EXPR: &str = "address:owner";
const ESTD_SAFE_ADDRESS_EXPR: &str = "sc:esdt-safe";
const ETH_TX_GAS_LIMIT: u64 = 150_000;

const USER_ETHEREUM_ADDRESS: &[u8] = b"0x0102030405060708091011121314151617181920";

const GAS_LIMIT: u64 = 100_000_000;

const MULTI_TRANSFER_CODE_PATH: MxscPath = MxscPath::new("output/multi-transfer-esdt.mxsc.json");
const BRIDGE_PROXY_CODE_PATH: MxscPath =
    MxscPath::new("../bridge-proxy/output/bridge-proxy.mxsc.json");
const MOCK_ESDT_SAFE_CODE_PATH: MxscPath =
    MxscPath::new("../common/mock-contracts/mock-esdt-safe/output/mock-esdt-safe.mxsc.json");
const ESDT_SAFE_CODE_PATH: MxscPath = MxscPath::new("../esdt-safe/output/esdt-safe.mxsc.json");
const BRIDGED_TOKENS_WRAPPER_CODE_PATH: MxscPath =
    MxscPath::new("../bridged-tokens-wrapper/output/bridged-tokens-wrapper.mxsc.json");
const MOCK_MULTISIG_CODE_PATH: MxscPath =
    MxscPath::new("../common/mock-contracts/mock-multisig/output/mock-multisig.mxsc.json");
const MOCK_MULTI_TRANSFER_CODE_PATH: MxscPath = MxscPath::new(
    "../common/mock-contracts/mock-multi-transfer-esdt/output/mock-multi-transfer-esdt.mxsc.json",
);
const MOCK_PRICE_AGGREGATOR_CODE_PATH: MxscPath = MxscPath::new(
    "../common/mock-contracts/mock-price-aggregator/output/mock-price-aggregator.mxsc.json",
);

const MULTI_TRANSFER_ADDRESS: TestSCAddress = TestSCAddress::new("multi-transfer");
const BRIDGE_PROXY_ADDRESS: TestSCAddress = TestSCAddress::new("bridge-proxy");
const ESDT_SAFE_ADDRESS: TestSCAddress = TestSCAddress::new("esdt-safe");
const BRIDGED_TOKENS_WRAPPER_ADDRESS: TestSCAddress = TestSCAddress::new("bridged-tokens-wrapper");
const PRICE_AGGREGATOR_ADDRESS: TestSCAddress = TestSCAddress::new("price-aggregator");
const MULTISIG_ADDRESS: TestSCAddress = TestSCAddress::new("multisig");

const ORACLE_ADDRESS: TestAddress = TestAddress::new("oracle");
const OWNER_ADDRESS: TestAddress = TestAddress::new("owner");
const USER1_ADDRESS: TestAddress = TestAddress::new("user1");
const USER2_ADDRESS: TestAddress = TestAddress::new("user2");
const RELAYER1_ADDRESS: TestAddress = TestAddress::new("relayer1");
const RELAYER2_ADDRESS: TestAddress = TestAddress::new("relayer2");

const ESDT_SAFE_ETH_TX_GAS_LIMIT: u64 = 150_000;
const MAX_AMOUNT: u64 = 100_000_000_000_000u64;

const BALANCE: &str = "2,000,000";
const ERROR: u64 = 4;
const MINTED_AMOUNT: u64 = 100_000_000_000;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(
        MULTI_TRANSFER_CODE_PATH,
        multi_transfer_esdt::ContractBuilder,
    );
    blockchain.register_contract(BRIDGE_PROXY_CODE_PATH, bridge_proxy::ContractBuilder);

    blockchain.register_contract(ESDT_SAFE_CODE_PATH, esdt_safe::ContractBuilder);
    blockchain.register_contract(MOCK_ESDT_SAFE_CODE_PATH, esdt_safe::ContractBuilder);

    blockchain.register_contract(
        BRIDGED_TOKENS_WRAPPER_CODE_PATH,
        bridged_tokens_wrapper::ContractBuilder,
    );
    blockchain.register_contract(MOCK_MULTISIG_CODE_PATH, mock_multisig::ContractBuilder);
    blockchain.register_contract(
        MOCK_PRICE_AGGREGATOR_CODE_PATH,
        mock_price_aggregator::ContractBuilder,
    );

    blockchain.register_contract(
        MOCK_MULTI_TRANSFER_CODE_PATH,
        mock_multi_transfer_esdt::ContractBuilder,
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
        let mut world = world();

        world
            .account(OWNER_ADDRESS)
            .nonce(1)
            .esdt_balance(BRIDGE_TOKEN_ID, 1001u64)
            .esdt_balance(TOKEN_TICKER, MAX_AMOUNT)
            .esdt_balance(WRAPPED_TOKEN_ID, 1001u64)
            .esdt_balance(TOKEN_ID, 1_000_000_000_000u64)
            .esdt_balance(NON_WHITELISTED_TOKEN, 1_000_000u64)
            .esdt_balance(TOKEN_WITH_BURN_ROLE, 100_000u64)
            .esdt_balance(TOKEN_WITHOUT_BURN_ROLE, 150_000u64)
            .esdt_balance(UNIVERSAL_TOKEN_IDENTIFIER, 1001u64)
            .account(USER1_ADDRESS)
            .nonce(1)
            .account(USER2_ADDRESS)
            .nonce(1);

        let roles = [
            "ESDTRoleLocalMint".to_string(),
            "ESDTRoleLocalBurn".to_string(),
        ];

        world
            .account(PRICE_AGGREGATOR_ADDRESS)
            .code(MOCK_PRICE_AGGREGATOR_CODE_PATH);

        Self { world }
    }

    fn multi_transfer_deploy(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(MULTISIG_ADDRESS)
            .typed(multi_transfer_esdt_proxy::MultiTransferEsdtProxy)
            .init()
            .code(MULTI_TRANSFER_CODE_PATH)
            .new_address(MULTI_TRANSFER_ADDRESS)
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
            .code(MOCK_MULTISIG_CODE_PATH)
            .new_address(MULTISIG_ADDRESS)
            .run();
        self
    }

    fn bridge_proxy_deploy(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(MULTISIG_ADDRESS)
            .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
            .init()
            .code(BRIDGE_PROXY_CODE_PATH)
            .new_address(BRIDGE_PROXY_ADDRESS)
            .run();

        self
    }

    fn safe_deploy(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(MULTISIG_ADDRESS)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .init(ETH_TX_GAS_LIMIT)
            .code(MOCK_ESDT_SAFE_CODE_PATH)
            .new_address(ESDT_SAFE_ADDRESS)
            .run();

        self
    }

    fn safe_deploy_real_contract(&mut self) -> &mut Self {
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

    fn bridged_tokens_wrapper_deploy(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(MULTISIG_ADDRESS)
            .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
            .init()
            .code(BRIDGED_TOKENS_WRAPPER_CODE_PATH)
            .new_address(BRIDGED_TOKENS_WRAPPER_ADDRESS)
            .run();

        self
    }

    fn config_multi_transfer(&mut self) {
        self.world.set_esdt_balance(
            MULTISIG_ADDRESS,
            b"BRIDGE-123456",
            BigUint::from(100_000_000_000u64),
        );

        self.world.set_esdt_balance(
            MULTISIG_ADDRESS,
            b"WRAPPED-123456",
            BigUint::from(100_000_000_000u64),
        );

        self.world.set_esdt_local_roles(
            ESDT_SAFE_ADDRESS,
            b"BRIDGE-123456",
            &[EsdtLocalRole::Mint, EsdtLocalRole::Burn],
        );

        self.world.set_esdt_local_roles(
            ESDT_SAFE_ADDRESS,
            b"WRAPPED-123456",
            &[EsdtLocalRole::Mint, EsdtLocalRole::Burn],
        );

        self.world.set_esdt_balance(
            MULTISIG_ADDRESS,
            b"TOKEN-123456",
            BigUint::from(100_000_000_000u64),
        );

        self.world.set_esdt_balance(
            MULTISIG_ADDRESS,
            b"TOKEN",
            BigUint::from(MAX_AMOUNT + 100000000),
        );

        self.world.set_esdt_local_roles(
            ESDT_SAFE_ADDRESS,
            b"TOKEN-123456",
            &[EsdtLocalRole::Mint, EsdtLocalRole::Burn],
        );
        self.esdt_raw_transaction_esdt_safe()
            .add_token_to_whitelist(
                TokenIdentifier::from_esdt_bytes("BRIDGE-123456"),
                "BRIDGE",
                true,
                false,
                BigUint::zero(),
                BigUint::zero(),
                BigUint::zero(),
                OptionalValue::Some(BigUint::from(ESDT_SAFE_ETH_TX_GAS_LIMIT)),
            )
            .run();

        self.esdt_raw_transaction_esdt_safe()
            .add_token_to_whitelist(
                TokenIdentifier::from_esdt_bytes("TOKEN"),
                "TOKEN",
                false,
                true,
                BigUint::from(MAX_AMOUNT),
                BigUint::zero(),
                BigUint::zero(),
                OptionalValue::Some(BigUint::from(ESDT_SAFE_ETH_TX_GAS_LIMIT)),
            )
            .single_esdt(
                &TokenIdentifier::from_esdt_bytes("TOKEN"),
                0,
                &BigUint::from(MAX_AMOUNT),
            )
            .run();

        self.world
            .tx()
            .from(MULTISIG_ADDRESS)
            .to(MULTI_TRANSFER_ADDRESS)
            .typed(multi_transfer_esdt_proxy::MultiTransferEsdtProxy)
            .set_max_bridged_amount(TOKEN_TICKER, MAX_AMOUNT - 1)
            .run();

        self.esdt_raw_transaction_esdt_safe()
            .unpause_endpoint()
            .run();

        self.world
            .tx()
            .from(MULTISIG_ADDRESS)
            .to(BRIDGED_TOKENS_WRAPPER_ADDRESS)
            .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
            .unpause_endpoint()
            .run();

        self.world
            .tx()
            .from(MULTISIG_ADDRESS)
            .to(BRIDGE_PROXY_ADDRESS)
            .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
            .unpause_endpoint()
            .run();
    }

    fn config_bridged_tokens_wrapper(&mut self) {
        self.world
            .tx()
            .from(MULTISIG_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .add_token_to_whitelist(
                TokenIdentifier::from_esdt_bytes("UNIV-abc123"),
                "BRIDGE1",
                true,
                false,
                BigUint::zero(),
                BigUint::zero(),
                BigUint::zero(),
                OptionalValue::Some(BigUint::from(ESDT_SAFE_ETH_TX_GAS_LIMIT)),
            )
            .run();
        self.world.set_esdt_balance(
            BRIDGE_PROXY_ADDRESS,
            b"UNIV-abc123",
            BigUint::from(10_000_000u64),
        );

        self.world.set_esdt_balance(
            BRIDGE_PROXY_ADDRESS,
            b"WRAPPED-123456",
            BigUint::from(10_000_000u64),
        );

        self.world.set_esdt_balance(
            BRIDGED_TOKENS_WRAPPER_ADDRESS,
            b"WRAPPED-123456",
            BigUint::from(10_000_000u64),
        );

        self.world.set_esdt_balance(
            BRIDGE_PROXY_ADDRESS,
            b"BRIDGE-123456",
            BigUint::from(10_000_000u64),
        );

        self.world.set_esdt_local_roles(
            BRIDGED_TOKENS_WRAPPER_ADDRESS,
            b"UNIV-abc123",
            &[EsdtLocalRole::Mint, EsdtLocalRole::Burn],
        );

        self.world
            .tx()
            .from(MULTISIG_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .set_eth_tx_gas_limit(0u64)
            .run();

        self.world
            .tx()
            .from(MULTISIG_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .init_supply_mint_burn(
                UNIVERSAL_TOKEN_IDENTIFIER,
                BigUint::from(600_000u64),
                BigUint::from(0u64),
            )
            .run();

        self.world
            .tx()
            .from(MULTISIG_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .init_supply_mint_burn(
                WRAPPED_TOKEN_ID,
                BigUint::from(600_000u64),
                BigUint::from(0u64),
            )
            .run();

        self.world
            .tx()
            .from(MULTISIG_ADDRESS)
            .to(BRIDGED_TOKENS_WRAPPER_ADDRESS)
            .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
            .add_wrapped_token(TokenIdentifier::from(UNIVERSAL_TOKEN_IDENTIFIER), 18u32)
            .run();

        self.world
            .tx()
            .from(MULTISIG_ADDRESS)
            .to(BRIDGED_TOKENS_WRAPPER_ADDRESS)
            .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
            .whitelist_token(
                TokenIdentifier::from(WRAPPED_TOKEN_ID),
                18u32,
                TokenIdentifier::from(UNIVERSAL_TOKEN_IDENTIFIER),
            )
            .run();
    }

    fn check_balances_on_safe(
        &mut self,
        token_id: TestTokenIdentifier,
        total_supply: BigUint<StaticApi>,
        total_minted: BigUint<StaticApi>,
        total_burned: BigUint<StaticApi>,
    ) {
        let actual_total_supply = self
            .world
            .query()
            .to(ESDT_SAFE_ADDRESS)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .total_balances(token_id)
            .returns(ReturnsResult)
            .run();

        assert_eq!(
            actual_total_supply, total_supply,
            "Total supply balance is wrong"
        );
        let actual_total_burned = self
            .world
            .query()
            .to(ESDT_SAFE_ADDRESS)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .burn_balances(token_id)
            .returns(ReturnsResult)
            .run();

        assert_eq!(
            actual_total_burned, total_burned,
            "Total burned balance is wrong"
        );

        let actual_total_minted = self
            .world
            .query()
            .to(ESDT_SAFE_ADDRESS)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .mint_balances(token_id)
            .returns(ReturnsResult)
            .run();

        assert_eq!(
            actual_total_minted, total_minted,
            "Total minted balance is wrong"
        );
    }

    fn deploy_contracts(&mut self) {
        self.multisig_deploy();
        self.multi_transfer_deploy();
        self.bridge_proxy_deploy();
        self.safe_deploy();
        self.bridged_tokens_wrapper_deploy();
    }

    fn config_esdt_safe(&mut self) {
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
        self.esdt_raw_transaction_esdt_safe()
            .add_token_to_whitelist(
                TOKEN_ID,
                "TOKEN",
                true,
                false,
                BigUint::from(0u64),
                BigUint::from(0u64),
                BigUint::from(0u64),
                OptionalValue::Some(BigUint::from(10u64)),
            )
            .run();
        self.esdt_raw_transaction_esdt_safe()
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
        self.esdt_raw_transaction_esdt_safe()
            .add_token_to_whitelist(
                TOKEN_WITHOUT_BURN_ROLE,
                "TKNW",
                false,
                true,
                BigUint::from(0u64),
                BigUint::from(0u64),
                BigUint::from(0u64),
                OptionalValue::Some(BigUint::from(0u64)),
            )
            .run();
        self.esdt_raw_transaction_esdt_safe()
            .add_token_to_whitelist(
                TokenIdentifier::from_esdt_bytes("WRAPPED-123456"),
                "BRIDGE2",
                true,
                false,
                BigUint::zero(),
                BigUint::zero(),
                BigUint::zero(),
                OptionalValue::Some(BigUint::from(ESDT_SAFE_ETH_TX_GAS_LIMIT)),
            )
            .run();

        self.world
            .tx()
            .from(MULTISIG_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .init_supply_mint_burn(
                TOKEN_ID,
                BigUint::from(600_000u64),
                BigUint::from(600_000u64),
            )
            .run();
    }

    fn single_transaction_should_fail(
        &mut self,
        token_id: TestTokenIdentifier,
        amount: u64,
        expected_error: &str,
    ) {
        self.esdt_raw_transaction_esdt_safe()
            .create_transaction(
                EthAddress::zero(),
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
        self.esdt_raw_transaction_esdt_safe()
            .create_transaction(
                EthAddress::zero(),
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
        self.esdt_raw_transaction_esdt_safe()
            .set_transaction_batch_status(batch_id, statuses)
            .returns(ExpectError(expected_status, expected_error))
            .run();
    }

    fn set_transaction_batch_status_should_work(
        &mut self,
        batch_id: u32,
        statuses: MultiValueEncoded<StaticApi, TransactionStatus>,
    ) {
        self.esdt_raw_transaction_esdt_safe()
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

    fn esdt_raw_transaction_esdt_safe(
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
fn test_config() {
    let mut state = MultiTransferTestState::new();

    state.deploy_contracts();
    state.config_esdt_safe();
    state.config_multi_transfer();
    state.config_bridged_tokens_wrapper();
}

#[test]
fn test_upgrade() {
    let mut state = MultiTransferTestState::new();
    state.deploy_contracts();
    state
        .world
        .tx()
        .from(MULTISIG_ADDRESS)
        .to(MULTI_TRANSFER_ADDRESS)
        .typed(multi_transfer_esdt_proxy::MultiTransferEsdtProxy)
        .upgrade()
        .code(MULTI_TRANSFER_CODE_PATH)
        .run();
}

#[test]
fn basic_transfer_test() {
    let mut state = MultiTransferTestState::new();
    let token_amount = BigUint::from(500u64);

    state.deploy_contracts();
    state.config_multi_transfer();

    let call_data = ManagedBuffer::from(b"add");
    call_data
        .clone()
        .concat(ManagedBuffer::from(GAS_LIMIT.to_string()));
    call_data.clone().concat(ManagedBuffer::default());

    let eth_tx = EthTransaction {
        from: EthAddress {
            raw_addr: ManagedByteArray::default(),
        },
        to: ManagedAddress::from(USER1_ADDRESS.eval_to_array()),
        token_id: TokenIdentifier::from(BRIDGE_TOKEN_ID),
        amount: token_amount.clone(),
        tx_nonce: 1u64,
        call_data: ManagedOption::some(call_data),
    };

    let mut transfers: MultiValueEncoded<StaticApi, EthTransaction<StaticApi>> =
        MultiValueEncoded::new();
    transfers.push(eth_tx);

    state
        .world
        .tx()
        .from(MULTISIG_ADDRESS)
        .to(MULTI_TRANSFER_ADDRESS)
        .typed(multi_transfer_esdt_proxy::MultiTransferEsdtProxy)
        .batch_transfer_esdt_token(1u32, transfers)
        .run();

    state
        .world
        .check_account(USER1_ADDRESS)
        .esdt_balance(BRIDGE_TOKEN_ID, token_amount);
}

#[test]
fn basic_transfer_smart_contract_dest_test() {
    let mut state = MultiTransferTestState::new();
    let token_amount = BigUint::from(500u64);

    state.deploy_contracts();
    state.config_multi_transfer();

    let call_data = ManagedBuffer::from(b"add");
    call_data
        .clone()
        .concat(ManagedBuffer::from(GAS_LIMIT.to_string()));
    call_data.clone().concat(ManagedBuffer::default());

    let eth_tx = EthTransaction {
        from: EthAddress {
            raw_addr: ManagedByteArray::default(),
        },
        to: ManagedAddress::from(MULTISIG_ADDRESS.eval_to_array()),
        token_id: TokenIdentifier::from(BRIDGE_TOKEN_ID),
        amount: token_amount.clone(),
        tx_nonce: 1u64,
        call_data: ManagedOption::some(call_data),
    };

    let mut transfers: MultiValueEncoded<StaticApi, EthTransaction<StaticApi>> =
        MultiValueEncoded::new();
    transfers.push(eth_tx);

    state
        .world
        .tx()
        .from(MULTISIG_ADDRESS)
        .to(MULTI_TRANSFER_ADDRESS)
        .typed(multi_transfer_esdt_proxy::MultiTransferEsdtProxy)
        .batch_transfer_esdt_token(1u32, transfers)
        .run();

    state
        .world
        .check_account(BRIDGE_PROXY_ADDRESS)
        .esdt_balance(BRIDGE_TOKEN_ID, token_amount);
}

#[test]
fn batch_transfer_both_executed_test() {
    let mut state = MultiTransferTestState::new();
    let token_amount = BigUint::from(500u64);

    state.deploy_contracts();
    state.config_esdt_safe();

    state.config_multi_transfer();

    let mut args = ManagedVec::new();
    args.push(ManagedBuffer::from(&[5u8]));

    let call_data: CallData<StaticApi> = CallData {
        endpoint: ManagedBuffer::from("add"),
        gas_limit: GAS_LIMIT,
        args: ManagedOption::some(args),
    };

    let call_data: ManagedBuffer<StaticApi> =
        ManagedSerializer::new().top_encode_to_managed_buffer(&call_data);

    let eth_tx1 = EthTransaction {
        from: EthAddress {
            raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
        },
        to: ManagedAddress::from(USER2_ADDRESS.eval_to_array()),
        token_id: TokenIdentifier::from(BRIDGE_TOKEN_ID),
        amount: token_amount.clone(),
        tx_nonce: 1u64,
        call_data: ManagedOption::some(call_data.clone()),
    };

    let eth_tx2 = EthTransaction {
        from: EthAddress {
            raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
        },
        to: ManagedAddress::from(USER1_ADDRESS.eval_to_array()),
        token_id: TokenIdentifier::from(WRAPPED_TOKEN_ID),
        amount: token_amount.clone(),
        tx_nonce: 2u64,
        call_data: ManagedOption::some(call_data),
    };

    let mut transfers: MultiValueEncoded<StaticApi, EthTransaction<StaticApi>> =
        MultiValueEncoded::new();
    transfers.push(eth_tx1);
    transfers.push(eth_tx2);

    state
        .world
        .tx()
        .from(MULTISIG_ADDRESS)
        .to(MULTI_TRANSFER_ADDRESS)
        .typed(multi_transfer_esdt_proxy::MultiTransferEsdtProxy)
        .batch_transfer_esdt_token(1u32, transfers)
        .run();

    state
        .world
        .check_account(USER1_ADDRESS)
        .esdt_balance(WRAPPED_TOKEN_ID, token_amount.clone());

    state
        .world
        .check_account(USER2_ADDRESS)
        .esdt_balance(BRIDGE_TOKEN_ID, token_amount);
}

#[test]
fn batch_two_transfers_same_token_test() {
    let mut state = MultiTransferTestState::new();
    let token_amount = BigUint::from(500u64);

    state.deploy_contracts();
    state.config_multi_transfer();

    let mut args = ManagedVec::new();
    args.push(ManagedBuffer::from(&[5u8]));

    let call_data: CallData<StaticApi> = CallData {
        endpoint: ManagedBuffer::from("add"),
        gas_limit: GAS_LIMIT,
        args: ManagedOption::some(args),
    };

    let call_data: ManagedBuffer<StaticApi> =
        ManagedSerializer::new().top_encode_to_managed_buffer(&call_data);

    let eth_tx1 = EthTransaction {
        from: EthAddress {
            raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
        },
        to: ManagedAddress::from(USER2_ADDRESS.eval_to_array()),
        token_id: TokenIdentifier::from(BRIDGE_TOKEN_ID),
        amount: token_amount.clone(),
        tx_nonce: 1u64,
        call_data: ManagedOption::some(call_data.clone()),
    };

    let eth_tx2 = EthTransaction {
        from: EthAddress {
            raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
        },
        to: ManagedAddress::from(USER1_ADDRESS.eval_to_array()),
        token_id: TokenIdentifier::from(BRIDGE_TOKEN_ID),
        amount: token_amount.clone(),
        tx_nonce: 2u64,
        call_data: ManagedOption::some(call_data),
    };

    let mut transfers: MultiValueEncoded<StaticApi, EthTransaction<StaticApi>> =
        MultiValueEncoded::new();
    transfers.push(eth_tx1);
    transfers.push(eth_tx2);

    state
        .world
        .tx()
        .from(MULTISIG_ADDRESS)
        .to(MULTI_TRANSFER_ADDRESS)
        .typed(multi_transfer_esdt_proxy::MultiTransferEsdtProxy)
        .batch_transfer_esdt_token(1u32, transfers)
        .run();

    state
        .world
        .check_account(USER1_ADDRESS)
        .esdt_balance(BRIDGE_TOKEN_ID, token_amount.clone());

    state
        .world
        .check_account(USER2_ADDRESS)
        .esdt_balance(BRIDGE_TOKEN_ID, token_amount);
}

#[test]
fn batch_transfer_both_failed_test() {
    let mut state = MultiTransferTestState::new();
    let token_amount = BigUint::from(500u64);

    state.deploy_contracts();
    state.config_multi_transfer();

    let mut args = ManagedVec::new();
    args.push(ManagedBuffer::from(&[5u8]));

    let call_data: CallData<StaticApi> = CallData {
        endpoint: ManagedBuffer::from("add"),
        gas_limit: GAS_LIMIT,
        args: ManagedOption::some(args),
    };

    let call_data: ManagedBuffer<StaticApi> =
        ManagedSerializer::new().top_encode_to_managed_buffer(&call_data);

    let eth_tx1 = EthTransaction {
        from: EthAddress {
            raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
        },
        to: ManagedAddress::from(BRIDGE_PROXY_ADDRESS.eval_to_array()),
        token_id: TokenIdentifier::from(BRIDGE_TOKEN_ID),
        amount: token_amount.clone(),
        tx_nonce: 1u64,
        call_data: ManagedOption::some(call_data.clone()),
    };

    let eth_tx2 = EthTransaction {
        from: EthAddress {
            raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
        },
        to: ManagedAddress::from(BRIDGE_PROXY_ADDRESS.eval_to_array()),
        token_id: TokenIdentifier::from(BRIDGE_TOKEN_ID),
        amount: token_amount.clone(),
        tx_nonce: 2u64,
        call_data: ManagedOption::some(call_data),
    };

    let mut transfers: MultiValueEncoded<StaticApi, EthTransaction<StaticApi>> =
        MultiValueEncoded::new();
    transfers.push(eth_tx1);
    transfers.push(eth_tx2);

    state
        .world
        .tx()
        .from(MULTISIG_ADDRESS)
        .to(MULTI_TRANSFER_ADDRESS)
        .typed(multi_transfer_esdt_proxy::MultiTransferEsdtProxy)
        .batch_transfer_esdt_token(1u32, transfers)
        .run();

    let first_batch = state
        .world
        .query()
        .to(MULTI_TRANSFER_ADDRESS)
        .typed(multi_transfer_esdt_proxy::MultiTransferEsdtProxy)
        .get_first_batch_any_status()
        .returns(ReturnsResult)
        .run();

    assert!(first_batch.is_none());

    state
        .world
        .tx()
        .from(MULTISIG_ADDRESS)
        .to(MULTI_TRANSFER_ADDRESS)
        .typed(multi_transfer_esdt_proxy::MultiTransferEsdtProxy)
        .move_refund_batch_to_safe()
        .run();

    let first_batch = state
        .world
        .query()
        .to(MULTI_TRANSFER_ADDRESS)
        .typed(multi_transfer_esdt_proxy::MultiTransferEsdtProxy)
        .get_first_batch_any_status()
        .returns(ReturnsResult)
        .run();

    assert!(first_batch.is_none());
}

#[test]
fn test_unwrap_token_create_transaction_paused() {
    let mut state = MultiTransferTestState::new();

    state.deploy_contracts();
    state.config_esdt_safe();
    state.config_bridged_tokens_wrapper();

    state
        .world
        .tx()
        .from(BRIDGE_PROXY_ADDRESS)
        .to(BRIDGED_TOKENS_WRAPPER_ADDRESS)
        .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
        .unwrap_token_create_transaction(
            TokenIdentifier::from(UNIVERSAL_TOKEN_IDENTIFIER),
            ESDT_SAFE_ADDRESS.to_address(),
            EthAddress::zero(),
            OptionalValue::<BigUint<StaticApi>>::None,
        )
        .egld_or_single_esdt(
            &EgldOrEsdtTokenIdentifier::esdt(UNIVERSAL_TOKEN_IDENTIFIER),
            0u64,
            &BigUint::from(10u64),
        )
        .returns(ExpectError(ERROR, "Contract is paused"))
        .run();
}

#[test]
fn test_unwrap_token_create_transaction_insufficient_liquidity() {
    let mut state = MultiTransferTestState::new();
    state.deploy_contracts();
    state.config_esdt_safe();
    state.config_multi_transfer();
    state.config_bridged_tokens_wrapper();

    state
        .world
        .tx()
        .from(MULTISIG_ADDRESS)
        .to(BRIDGED_TOKENS_WRAPPER_ADDRESS)
        .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
        .unpause_endpoint()
        .run();

    state
        .world
        .tx()
        .from(MULTISIG_ADDRESS)
        .to(BRIDGED_TOKENS_WRAPPER_ADDRESS)
        .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
        .deposit_liquidity()
        .egld_or_single_esdt(
            &EgldOrEsdtTokenIdentifier::esdt(WRAPPED_TOKEN_ID),
            0u64,
            &BigUint::from(1_000u64),
        )
        .run();

    state
        .world
        .set_esdt_balance(USER1_ADDRESS, b"UNIV-abc123", BigUint::from(5_000u64));

    state
        .world
        .tx()
        .from(USER1_ADDRESS)
        .to(BRIDGED_TOKENS_WRAPPER_ADDRESS)
        .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
        .unwrap_token_create_transaction(
            WRAPPED_TOKEN_ID,
            ESDT_SAFE_ADDRESS.to_address(),
            EthAddress::zero(),
            OptionalValue::<BigUint<StaticApi>>::None,
        )
        .egld_or_single_esdt(
            &EgldOrEsdtTokenIdentifier::esdt(UNIVERSAL_TOKEN_IDENTIFIER),
            0u64,
            &BigUint::from(2_000u64),
        )
        .returns(ExpectError(ERROR, "Contract does not have enough funds"))
        .run();
}

#[test]
fn test_unwrap_token_create_transaction_should_work() {
    let mut state = MultiTransferTestState::new();

    state.deploy_contracts();

    state.config_esdt_safe();
    state.config_multi_transfer();
    state.config_bridged_tokens_wrapper();

    state
        .world
        .tx()
        .from(MULTISIG_ADDRESS)
        .to(BRIDGED_TOKENS_WRAPPER_ADDRESS)
        .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
        .deposit_liquidity()
        .egld_or_single_esdt(
            &EgldOrEsdtTokenIdentifier::esdt(WRAPPED_TOKEN_ID),
            0u64,
            &BigUint::from(1_000u64),
        )
        .run();

    state
        .world
        .set_esdt_balance(USER1_ADDRESS, b"UNIV-abc123", BigUint::from(5_000u64));

    state.check_balances_on_safe(
        WRAPPED_TOKEN_ID,
        BigUint::zero(),
        BigUint::from(600_000u64),
        BigUint::zero(),
    );

    state
        .world
        .query()
        .to(BRIDGED_TOKENS_WRAPPER_ADDRESS)
        .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
        .token_liquidity(WRAPPED_TOKEN_ID)
        .returns(ExpectValue(BigUint::from(1_000u64)))
        .run();

    state
        .world
        .tx()
        .from(USER1_ADDRESS)
        .to(BRIDGED_TOKENS_WRAPPER_ADDRESS)
        .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
        .unwrap_token_create_transaction(
            WRAPPED_TOKEN_ID,
            ESDT_SAFE_ADDRESS.to_address(),
            EthAddress {
                raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
            },
            OptionalValue::<BigUint<StaticApi>>::None,
        )
        .egld_or_single_esdt(
            &EgldOrEsdtTokenIdentifier::esdt(UNIVERSAL_TOKEN_IDENTIFIER),
            0u64,
            &BigUint::from(900u64),
        )
        .run();

    state
        .world
        .query()
        .to(BRIDGED_TOKENS_WRAPPER_ADDRESS)
        .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
        .token_liquidity(WRAPPED_TOKEN_ID)
        .returns(ExpectValue(BigUint::from(100u64)))
        .run();

    state.check_balances_on_safe(
        WRAPPED_TOKEN_ID,
        BigUint::zero(),
        BigUint::from(600000u64),
        BigUint::from(900u64),
    );
}

#[test]
fn test_unwrap_token_create_transaction_should_fail() {
    let mut state = MultiTransferTestState::new();

    state.deploy_contracts();
    state.config_esdt_safe();
    state.config_multi_transfer();
    state.config_bridged_tokens_wrapper();

    state
        .world
        .set_esdt_balance(USER1_ADDRESS, b"TOKEN", BigUint::from(5_000u64));

    state
        .world
        .tx()
        .from(USER1_ADDRESS)
        .to(BRIDGED_TOKENS_WRAPPER_ADDRESS)
        .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
        .unwrap_token_create_transaction(
            WRAPPED_TOKEN_ID,
            ESDT_SAFE_ADDRESS.to_address(),
            EthAddress::zero(),
            OptionalValue::<BigUint<StaticApi>>::None,
        )
        .egld_or_single_esdt(
            &EgldOrEsdtTokenIdentifier::esdt(TOKEN_TICKER),
            0u64,
            &BigUint::from(1_000u64),
        )
        .returns(ExpectError(ERROR, "Esdt token unavailable"))
        .run();
}

#[test]
fn test_unwrap_token_create_transaction_amount_zero() {
    let mut state = MultiTransferTestState::new();

    state.deploy_contracts();
    state.config_esdt_safe();
    state.config_multi_transfer();
    state.config_bridged_tokens_wrapper();
    state
        .world
        .tx()
        .from(BRIDGE_PROXY_ADDRESS)
        .to(BRIDGED_TOKENS_WRAPPER_ADDRESS)
        .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
        .unwrap_token_create_transaction(
            TokenIdentifier::from(WRAPPED_TOKEN_ID),
            ESDT_SAFE_ADDRESS.to_address(),
            EthAddress::zero(),
            OptionalValue::<BigUint<StaticApi>>::None,
        )
        .egld_or_single_esdt(
            &EgldOrEsdtTokenIdentifier::esdt(UNIVERSAL_TOKEN_IDENTIFIER),
            0u64,
            &BigUint::from(0u64),
        )
        .returns(ExpectError(ERROR, "Must pay more than 0 tokens!"))
        .run();
}

#[test]
fn add_refund_batch_test_should_work() {
    let mut state: MultiTransferTestState = MultiTransferTestState::new();

    state.multisig_deploy();
    state.multi_transfer_deploy();
    state.bridge_proxy_deploy();
    state.safe_deploy_real_contract();
    state.bridged_tokens_wrapper_deploy();
    state.config_multi_transfer();

    let eth_tx = EthTransaction {
        from: EthAddress::zero(),
        to: ManagedAddress::from(USER1_ADDRESS.eval_to_array()),
        token_id: TokenIdentifier::from(TOKEN_TICKER),
        amount: BigUint::from(MAX_AMOUNT),
        tx_nonce: 1u64,
        call_data: ManagedOption::none(),
    };

    let mut transfers: MultiValueEncoded<StaticApi, EthTransaction<StaticApi>> =
        MultiValueEncoded::new();
    transfers.push(eth_tx.clone());

    let fee = state
        .world
        .query()
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .calculate_required_fee(TOKEN_TICKER)
        .returns(ReturnsResult)
        .run();

    state.check_balances_on_safe(
        TOKEN_TICKER,
        BigUint::from(MAX_AMOUNT),
        BigUint::zero(),
        BigUint::zero(),
    );

    state
        .world
        .tx()
        .from(MULTISIG_ADDRESS)
        .to(MULTI_TRANSFER_ADDRESS)
        .typed(multi_transfer_esdt_proxy::MultiTransferEsdtProxy)
        .batch_transfer_esdt_token(1u32, transfers)
        .run();

    state.check_balances_on_safe(
        TOKEN_TICKER,
        BigUint::zero(),
        BigUint::zero(),
        BigUint::zero(),
    );

    state
        .world
        .tx()
        .from(MULTISIG_ADDRESS)
        .to(MULTI_TRANSFER_ADDRESS)
        .typed(multi_transfer_esdt_proxy::MultiTransferEsdtProxy)
        .move_refund_batch_to_safe()
        .run();

    state.check_balances_on_safe(
        TOKEN_TICKER,
        BigUint::from(MAX_AMOUNT) - fee,
        BigUint::zero(),
        BigUint::zero(),
    );
}

#[test]
fn batch_transfer_esdt_token_to_address_zero() {
    let mut state: MultiTransferTestState = MultiTransferTestState::new();

    state.multisig_deploy();
    state.multi_transfer_deploy();
    state.bridge_proxy_deploy();
    state.safe_deploy_real_contract();
    state.bridged_tokens_wrapper_deploy();
    state.config_multi_transfer();

    let eth_tx = EthTransaction {
        from: EthAddress::zero(),
        to: ManagedAddress::zero(),
        token_id: TokenIdentifier::from(TOKEN_TICKER),
        amount: BigUint::from(MAX_AMOUNT),
        tx_nonce: 1u64,
        call_data: ManagedOption::none(),
    };

    let mut transfers: MultiValueEncoded<StaticApi, EthTransaction<StaticApi>> =
        MultiValueEncoded::new();
    transfers.push(eth_tx.clone());

    state
        .world
        .tx()
        .from(MULTISIG_ADDRESS)
        .to(MULTI_TRANSFER_ADDRESS)
        .typed(multi_transfer_esdt_proxy::MultiTransferEsdtProxy)
        .batch_transfer_esdt_token(1u32, transfers)
        .run();
}
