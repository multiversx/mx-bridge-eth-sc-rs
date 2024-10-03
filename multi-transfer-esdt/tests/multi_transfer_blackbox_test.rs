#![allow(unused)]

use bridge_proxy::{
    bridge_proxy_contract_proxy,
    config::{self, ProxyTrait as _},
    ProxyTrait as _,
};
use bridged_tokens_wrapper::ProxyTrait as _;
use esdt_safe::{EsdtSafe, ProxyTrait as _};
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
        Address, BigUint, CodeMetadata, EgldOrEsdtTokenIdentifier, EgldOrMultiEsdtPayment, EsdtLocalRole,
        ManagedAddress, ManagedBuffer, ManagedByteArray, ManagedOption, ManagedVec,
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
use token_module::ProxyTrait as _;
use transaction::{transaction_status::TransactionStatus, CallData, EthTransaction, Transaction};
use tx_batch_module::BatchStatus;

const UNIVERSAL_TOKEN_IDENTIFIER: TestTokenIdentifier = TestTokenIdentifier::new("UNIV-abc123");
const BRIDGE_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("BRIDGE-123456");
const WRAPPED_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("WRAPPED-123456");
const TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("TOKEN-123456");
const TOKEN_WITH_BURN_ROLE: TestTokenIdentifier = TestTokenIdentifier::new("TOKEN-WITH");
const TOKEN_WITHOUT_BURN_ROLE: TestTokenIdentifier = TestTokenIdentifier::new("TOKEN-WITH-OUT");

const NON_WHITELISTED_TOKEN: TestTokenIdentifier =
    TestTokenIdentifier::new("NON-WHITELISTED-123456");
const OWNER_ADDRESS_EXPR: &str = "address:owner";
const ESTD_SAFE_ADDRESS_EXPR: &str = "sc:esdt-safe";

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
        let mut world = world();

        world
            .account(OWNER_ADDRESS)
            .nonce(1)
            .esdt_balance(BRIDGE_TOKEN_ID, 1001u64)
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

        let roles = vec![
            "ESDTRoleLocalMint".to_string(),
            "ESDTRoleLocalBurn".to_string(),
        ];
        world
            .account(ESDT_SAFE_ADDRESS)
            .esdt_roles(BRIDGE_TOKEN_ID, roles.clone())
            .esdt_roles(UNIVERSAL_TOKEN_IDENTIFIER, roles.clone())
            .esdt_roles(WRAPPED_TOKEN_ID, roles.clone())
            .code(ESDT_SAFE_CODE_PATH)
            .owner(OWNER_ADDRESS);

        Self { world }
    }

    fn multi_transfer_deploy(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(multi_transfer_esdt_proxy::MultiTransferEsdtProxy)
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
        self.esdt_raw_transaction()
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
            .typed(multi_transfer_esdt_proxy::MultiTransferEsdtProxy)
            .set_esdt_safe_contract_address(OptionalValue::Some(ESDT_SAFE_ADDRESS.to_address()))
            .run();

        self.esdt_raw_transaction()
            .set_multi_transfer_contract_address(OptionalValue::Some(
                MULTI_TRANSFER_ADDRESS.to_address(),
            ))
            .run();

        self.esdt_raw_transaction()
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

        self.esdt_raw_transaction()
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

        self.esdt_raw_transaction().unpause_endpoint().run();

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .unpause_endpoint()
            .run();

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(BRIDGED_TOKENS_WRAPPER_ADDRESS)
            .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
            .unpause_endpoint()
            .run();

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(BRIDGE_PROXY_ADDRESS)
            .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
            .unpause_endpoint()
            .run();
    }

    fn config_bridged_tokens_wrapper(&mut self) {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(BRIDGED_TOKENS_WRAPPER_ADDRESS)
            .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
            .set_esdt_safe_contract_address(OptionalValue::Some(ESDT_SAFE_ADDRESS.to_address()))
            .run();

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(BRIDGED_TOKENS_WRAPPER_ADDRESS)
            .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
            .set_bridge_proxy_contract_address(OptionalValue::Some(
                BRIDGE_PROXY_ADDRESS.to_address(),
            ))
            .run();

        self.world
            .tx()
            .from(OWNER_ADDRESS)
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
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .set_eth_tx_gas_limit(0u64)
            .run();

        self.world
            .tx()
            .from(OWNER_ADDRESS)
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
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
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
            .from(OWNER_ADDRESS)
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
            .from(OWNER_ADDRESS)
            .to(BRIDGE_PROXY_ADDRESS)
            .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
            .set_bridged_tokens_wrapper_contract_address(OptionalValue::Some(
                BRIDGED_TOKENS_WRAPPER_ADDRESS.to_address(),
            ))
            .run();

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .set_bridged_tokens_wrapper_contract_address(OptionalValue::Some(
                BRIDGED_TOKENS_WRAPPER_ADDRESS.to_address(),
            ))
            .run();

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(BRIDGED_TOKENS_WRAPPER_ADDRESS)
            .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
            .add_wrapped_token(TokenIdentifier::from(UNIVERSAL_TOKEN_IDENTIFIER), 18u32)
            .run();

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(BRIDGED_TOKENS_WRAPPER_ADDRESS)
            .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
            .whitelist_token(
                TokenIdentifier::from(WRAPPED_TOKEN_ID),
                18u32,
                TokenIdentifier::from(UNIVERSAL_TOKEN_IDENTIFIER),
            )
            .run();
    }
    fn deploy_contracts(&mut self) {
        self.multi_transfer_deploy();
        self.bridge_proxy_deploy();
        self.safe_deploy(Address::zero());
        self.bridged_tokens_wrapper_deploy();
    }
}

fn config_esdtsafe(&mut self) {
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
        self.esdt_raw_transaction()
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
        self.esdt_raw_transaction()
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
        self.esdt_raw_transaction()
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
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .init_supply_mint_burn(
                TOKEN_ID,
                BigUint::from(600_000u64),
                BigUint::from(600_000u64),
            )
            .run();
    }
fn esdt_raw_transaction(
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
        .from(OWNER_ADDRESS)
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
fn batch_transfer_both_executed_test() {
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
        .from(OWNER_ADDRESS)
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
        .from(OWNER_ADDRESS)
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
        .from(OWNER_ADDRESS)
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
        .from(OWNER_ADDRESS)
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
fn esdt_safe_create_transaction() {
    let mut state = MultiTransferTestState::new();

    state.multi_transfer_deploy();
    state.bridge_proxy_deploy();
    state.safe_deploy(Address::zero());
    state.bridged_tokens_wrapper_deploy();

    state.single_transaction_should_fail(
        BRIDGE_TOKEN_ID,
        10u64,
        "Cannot create transaction while paused",
    );

    state.config_multi_transfer();

    state.single_transaction_should_fail(
        BRIDGE_TOKEN_ID,
        1u64,
        "Transaction fees cost more than the entire bridged amount",
    );

    state.config_esdtsafe();

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .set_eth_tx_gas_limit(10_000u64)
        .run();

    state.single_transaction_should_fail(TOKEN_ID, 800_000u64, "Not enough minted tokens!");

    state.single_transaction_should_fail(NON_WHITELISTED_TOKEN, 100u64, "Token not in whitelist");

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .init_supply_mint_burn(
            TOKEN_WITH_BURN_ROLE,
            BigUint::from(100u64),
            BigUint::from(100u64),
        )
        .run();

    state.single_transaction_should_fail(
        TOKEN_WITH_BURN_ROLE,
        100u64,
        "Cannot do the burn action!",
    );

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .init_supply_mint_burn(
            TOKEN_ID,
            BigUint::from(500_000u64),
            BigUint::from(100_000u64),
        )
        .run();

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .set_eth_tx_gas_limit(1000u64)
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

    assert_eq!(total_accumulated_transaction_fee, BigUint::from(10_000u64));

    state.single_transaction_should_work(TOKEN_WITHOUT_BURN_ROLE, 120_000u64);

    let total_balances = state
        .world
        .query()
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .total_balances(TOKEN_WITHOUT_BURN_ROLE)
        .returns(ReturnsResult)
        .run();

    assert_eq!(total_balances, BigUint::from(120_000u64));
}

#[test]
fn set_transaction_batch_status_test() {
    let mut state = MultiTransferTestState::new();

    state.multi_transfer_deploy();
    state.bridge_proxy_deploy();
    state.safe_deploy(Address::zero());
    state.bridged_tokens_wrapper_deploy();
    state.config_multi_transfer();
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
        .from(OWNER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .init_supply_mint_burn(
            TOKEN_ID,
            BigUint::from(100_000u64),
            BigUint::from(10_000u64),
        )
        .run();

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
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
fn add_refund_batch_test() {
    let mut state = MultiTransferTestState::new();

    state.multi_transfer_deploy();
    state.bridge_proxy_deploy();
    state.safe_deploy(Address::zero());
    state.bridged_tokens_wrapper_deploy();
    state.config_multi_transfer();
    state.config_esdtsafe();

    let eth_tx = Transaction {
        from: ManagedBuffer::from(OWNER_ADDRESS_EXPR),
        to: ManagedBuffer::from(ESTD_SAFE_ADDRESS_EXPR),
        amount: BigUint::from(100_000_000u64),
        block_nonce: 0u64,
        nonce: 0u64,
        token_identifier: TokenIdentifier::from(TOKEN_ID),
        is_refund_tx: true,
    };

    let eth_tx2 = Transaction {
        from: ManagedBuffer::from(OWNER_ADDRESS_EXPR),
        to: ManagedBuffer::from(ESTD_SAFE_ADDRESS_EXPR),
        amount: BigUint::from(100_000_000u64),
        block_nonce: 0u64,
        nonce: 0u64,
        token_identifier: TokenIdentifier::from(TOKEN_ID),
        is_refund_tx: true,
    };

    let mut transfers: ManagedVec<StaticApi, Transaction<StaticApi>> = ManagedVec::new();
    transfers.push(eth_tx);
    transfers.push(eth_tx2);

    let payments = vec![
        EsdtTokenPayment::new(TOKEN_ID.into(), 0, BigUint::from(100_000_000u64)),
        EsdtTokenPayment::new(TOKEN_ID.into(), 0, BigUint::from(100_000_000u64)),
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
        b"BRIDGE-123456",
        BigUint::from(10_000u64),
    );

    state.world.set_esdt_balance(
        MULTI_TRANSFER_ADDRESS,
        b"TOKEN-123456",
        BigUint::from(100_000_000_000u64),
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
        BRIDGE_TOKEN_ID,
        10u64,
        "Token identifiers do not match",
    );

    let payments_invalid = vec![
        EsdtTokenPayment::new(TOKEN_ID.into(), 0, BigUint::from(1_000u64)),
        EsdtTokenPayment::new(TOKEN_ID.into(), 0, BigUint::from(100u64)),
    ];
    let payment_invalid = EgldOrMultiEsdtPayment::MultiEsdt(payments_invalid.into());

    state.add_refund_batch_tx_multiple_payment_should_fail(
        MULTI_TRANSFER_ADDRESS,
        ESDT_SAFE_ADDRESS,
        transfers.clone(),
        payment_invalid.clone(),
        "Amounts do not match",
    );

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
fn claim_refund_test() {
    let mut state = MultiTransferTestState::new();

    state.multi_transfer_deploy();
    state.bridge_proxy_deploy();
    state.safe_deploy(Address::zero());
    state.bridged_tokens_wrapper_deploy();
    state.config_multi_transfer();
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
        .from(OWNER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .set_eth_tx_gas_limit(1000u64)
        .run();

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .init_supply_mint_burn(
            TOKEN_ID,
            BigUint::from(100_000u64),
            BigUint::from(10_000u64),
        )
        .run();

    state.single_transaction_should_work(TOKEN_ID, 100_000u64);
    state.set_transaction_batch_status_should_work(1, tx_statuses.clone());

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
    assert_eq!(amount, BigUint::from(90_000u64));

    let result2 = state
        .esdt_raw_transaction()
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

#[test]
fn test_unwrap_token_create_transaction_paused() {
    let mut state = MultiTransferTestState::new();

    state.deploy_contracts();

    state.config_bridged_tokens_wrapper();

    state
        .world
        .tx()
        .from(BRIDGE_PROXY_ADDRESS)
        .to(BRIDGED_TOKENS_WRAPPER_ADDRESS)
        .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
        .unwrap_token_create_transaction(
            TokenIdentifier::from(UNIVERSAL_TOKEN_IDENTIFIER),
            EthAddress::zero(),
            OptionalValue::Some(ManagedAddress::zero()),
        )
        .egld_or_single_esdt(
            &EgldOrEsdtTokenIdentifier::esdt(UNIVERSAL_TOKEN_IDENTIFIER),
            0u64,
            &BigUint::from(10u64),
        )
        .returns(ExpectError(ERROR, "Cannot create transaction while paused"))
        .run();
}

#[test]
fn test_unwrap_token_create_transaction_insufficient_liquidity() {
    let mut state = MultiTransferTestState::new();
    state.deploy_contracts();
    state.config_multi_transfer();
    state.config_bridged_tokens_wrapper();

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
            EthAddress::zero(),
            OptionalValue::Some(ManagedAddress::zero()),
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
            EthAddress::zero(),
            OptionalValue::Some(ManagedAddress::zero()),
        )
        .egld_or_single_esdt(
            &EgldOrEsdtTokenIdentifier::esdt(WRAPPED_TOKEN_ID),
            0u64,
            &BigUint::from(100u64),
        )
        .run();
}

#[test]
fn test_unwrap_token_create_transaction_amount_zero() {
    let mut state = MultiTransferTestState::new();

    state.deploy_contracts();

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
            EthAddress::zero(),
            OptionalValue::Some(ManagedAddress::zero()),
        )
        .egld_or_single_esdt(
            &EgldOrEsdtTokenIdentifier::esdt(UNIVERSAL_TOKEN_IDENTIFIER),
            0u64,
            &BigUint::from(0u64),
        )
        .returns(ExpectError(ERROR, "Must pay more than 0 tokens!"))
        .run();
}
