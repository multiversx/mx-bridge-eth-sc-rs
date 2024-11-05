#![allow(unused)]

use std::ops::Add;

use bridge_proxy::{config::ProxyTrait as _, ProxyTrait as _};
use esdt_safe::{EsdtSafe, ProxyTrait as _};

use multisig::__endpoints_5__::multi_transfer_esdt_address;
use multiversx_sc::{
    api::{HandleConstraints, ManagedTypeApi},
    codec::{
        multi_types::{MultiValueVec, OptionalValue},
        Empty,
    },
    contract_base::ManagedSerializer,
    hex_literal::hex,
    imports::MultiValue2,
    storage::mappers::SingleValue,
    types::{
        Address, BigUint, CodeMetadata, EgldOrEsdtTokenIdentifier, EgldOrMultiEsdtPayment,
        EsdtLocalRole, EsdtTokenPayment, ManagedAddress, ManagedBuffer, ManagedByteArray,
        ManagedOption, ManagedType, ManagedVec, MultiValueEncoded, ReturnsNewManagedAddress,
        ReturnsResult, TestAddress, TestSCAddress, TestTokenIdentifier, TokenIdentifier,
    },
};
use multiversx_sc_modules::pause::ProxyTrait;
use multiversx_sc_scenario::{
    api::{StaticApi, VMHooksApi, VMHooksApiBackend},
    imports::MxscPath,
    scenario_format::interpret_trait::{InterpretableFrom, InterpreterContext},
    scenario_model::*,
    ContractInfo, DebugApi, ExpectError, ExpectValue, ScenarioTxRun, ScenarioWorld,
};

use eth_address::*;
use sc_proxies::{
    bridge_proxy_contract_proxy, bridged_tokens_wrapper_proxy, esdt_safe_proxy,
    multi_transfer_esdt_proxy, multisig_proxy,
};
use token_module::ProxyTrait as _;
use transaction::{
    transaction_status::TransactionStatus, CallData, EthTransaction, EthTxAsMultiValue,
    Transaction, TxBatchSplitInFields,
};

const WEGLD_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("WEGLD-123456");
const ETH_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("ETH-123456");
const NATIVE_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("EGLD-123456");

const USER_ETHEREUM_ADDRESS: &[u8] = b"0x0102030405060708091011121314151617181920";

const GAS_LIMIT: u64 = 100_000_000;
const ETH_TX_GAS_LIMIT: u64 = 150_000;

const MULTISIG_CODE_PATH: MxscPath = MxscPath::new("output/multisig.mxsc.json");
const MULTI_TRANSFER_CODE_PATH: MxscPath =
    MxscPath::new("../multi-transfer-esdt/output/multi-transfer-esdt.mxsc.json");
const MOCK_MULTI_TRANSFER_PATH_EXPR: MxscPath = MxscPath::new(
    "../common/mock-contracts/mock-multi-transfer-esdt/output/mock-multi-transfer-esdt.mxsc.json",
);
const MOCK_ESDT_SAFE_PATH_EXPR: MxscPath =
    MxscPath::new("../common/mock-contracts/mock-esdt-safe/output/mock-esdt-safe.mxsc.json");
const BRIDGE_PROXY_CODE_PATH: MxscPath =
    MxscPath::new("../bridge-proxy/output/bridge-proxy.mxsc.json");
const ESDT_SAFE_CODE_PATH: MxscPath = MxscPath::new("../esdt-safe/output/esdt-safe.mxsc.json");
const BRIDGED_TOKENS_WRAPPER_CODE_PATH: MxscPath =
    MxscPath::new("../bridged-tokens-wrapper/output/bridged-tokens-wrapper.mxsc.json");
const PRICE_AGGREGATOR_CODE_PATH: MxscPath =
    MxscPath::new("../price-aggregator/price-aggregator.mxsc.json");

const MULTISIG_ADDRESS: TestSCAddress = TestSCAddress::new("multisig");
const MULTI_TRANSFER_ADDRESS: TestSCAddress = TestSCAddress::new("multi-transfer");
const MOCK_MULTI_TRANSFER_ADDRESS: TestSCAddress = TestSCAddress::new("mock-multi-transfer");
const BRIDGE_PROXY_ADDRESS: TestSCAddress = TestSCAddress::new("bridge-proxy");
const ESDT_SAFE_ADDRESS: TestSCAddress = TestSCAddress::new("esdt-safe");
const MOCK_ESDT_SAFE_ADDRESS: TestSCAddress = TestSCAddress::new("mock-esdt-safe");
const BRIDGED_TOKENS_WRAPPER_ADDRESS: TestSCAddress = TestSCAddress::new("bridged-tokens-wrapper");
const PRICE_AGGREGATOR_ADDRESS: TestSCAddress = TestSCAddress::new("price-aggregator");

const ORACLE_ADDRESS: TestAddress = TestAddress::new("oracle");
const OWNER_ADDRESS: TestAddress = TestAddress::new("owner");
const USER1_ADDRESS: TestAddress = TestAddress::new("user1");
const USER2_ADDRESS: TestAddress = TestAddress::new("user2");
const NON_BOARD_MEMEBER_ADDRESS: TestAddress = TestAddress::new("non-board-member");
const RELAYER1_ADDRESS: TestAddress = TestAddress::new("relayer1");
const RELAYER2_ADDRESS: TestAddress = TestAddress::new("relayer2");

const RANDOM_SC_ADDRESS: TestSCAddress = TestSCAddress::new("random-sc");

const ESDT_SAFE_ETH_TX_GAS_LIMIT: u64 = 150_000;

const BALANCE: &str = "2,000,000";

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(MULTISIG_CODE_PATH, multisig::ContractBuilder);
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
    blockchain.register_contract(
        PRICE_AGGREGATOR_CODE_PATH,
        fee_estimator_module::ContractBuilder,
    );
    blockchain.register_contract(
        MOCK_MULTI_TRANSFER_PATH_EXPR,
        mock_multi_transfer_esdt::ContractBuilder,
    );
    blockchain.register_contract(MOCK_ESDT_SAFE_PATH_EXPR, mock_esdt_safe::ContractBuilder);

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
            .esdt_balance(WEGLD_TOKEN_ID, 1001u64)
            .esdt_balance(ETH_TOKEN_ID, 1001u64)
            .esdt_balance(NATIVE_TOKEN_ID, 100_000u64)
            .account(USER1_ADDRESS)
            .nonce(1)
            .account(RELAYER1_ADDRESS)
            .nonce(1)
            .balance(2_000u64)
            .account(RELAYER2_ADDRESS)
            .nonce(1)
            .balance(2_000u64)
            .account(NON_BOARD_MEMEBER_ADDRESS)
            .nonce(1);

        world
            .account(PRICE_AGGREGATOR_ADDRESS)
            .code(PRICE_AGGREGATOR_CODE_PATH);

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
            .typed(multisig_proxy::MultisigProxy)
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
            .code(MULTISIG_CODE_PATH)
            .new_address(MULTISIG_ADDRESS)
            .run();
        self
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

    fn mock_multi_transfer_deploy(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(MULTISIG_ADDRESS)
            .typed(multi_transfer_esdt_proxy::MultiTransferEsdtProxy)
            .init()
            .code(MOCK_MULTI_TRANSFER_PATH_EXPR)
            .new_address(MOCK_MULTI_TRANSFER_ADDRESS)
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

    fn safe_deploy(&mut self) {
        self.world
            .tx()
            .from(MULTISIG_ADDRESS)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .init(ETH_TX_GAS_LIMIT)
            .code(ESDT_SAFE_CODE_PATH)
            .new_address(ESDT_SAFE_ADDRESS)
            .run();
    }

    fn mock_safe_deploy(&mut self) {
        self.world
            .tx()
            .from(MULTISIG_ADDRESS)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .init(ETH_TX_GAS_LIMIT)
            .code(MOCK_ESDT_SAFE_PATH_EXPR)
            .new_address(MOCK_ESDT_SAFE_ADDRESS)
            .run();
    }

    fn config_multisig(&mut self) {
        self.world
            .tx()
            .from(MULTISIG_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .add_token_to_whitelist(
                TokenIdentifier::from_esdt_bytes("WEGLD-123456"),
                "WEGLD",
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
            .add_token_to_whitelist(
                TokenIdentifier::from_esdt_bytes("ETH-123456"),
                "ETH",
                true,
                false,
                BigUint::zero(),
                BigUint::zero(),
                BigUint::zero(),
                OptionalValue::Some(BigUint::from(ESDT_SAFE_ETH_TX_GAS_LIMIT)),
            )
            .run();

        self.world.set_esdt_balance(
            MULTISIG_ADDRESS,
            b"EGLD-123456",
            BigUint::from(100_000_000_000u64),
        );

        self.world
            .tx()
            .from(MULTISIG_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .add_token_to_whitelist(
                TokenIdentifier::from_esdt_bytes("EGLD-123456"),
                "EGLD",
                false,
                true,
                BigUint::zero(),
                BigUint::zero(),
                BigUint::zero(),
                OptionalValue::Some(BigUint::from(ESDT_SAFE_ETH_TX_GAS_LIMIT)),
            )
            .run();

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(MULTISIG_ADDRESS)
            .typed(multisig_proxy::MultisigProxy)
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
            .typed(multisig_proxy::MultisigProxy)
            .unpause_endpoint()
            .run();

        self.world
            .tx()
            .from(MULTISIG_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .unpause_endpoint()
            .run();

        self.world
            .tx()
            .from(RELAYER1_ADDRESS)
            .to(MULTISIG_ADDRESS)
            .typed(multisig_proxy::MultisigProxy)
            .stake()
            .egld(1_000)
            .run();

        self.world
            .tx()
            .from(RELAYER2_ADDRESS)
            .to(MULTISIG_ADDRESS)
            .typed(multisig_proxy::MultisigProxy)
            .stake()
            .egld(1_000)
            .run();

        let staked_relayers = self
            .world
            .query()
            .to(MULTISIG_ADDRESS)
            .typed(multisig_proxy::MultisigProxy)
            .get_all_staked_relayers()
            .returns(ReturnsResult)
            .run();

        self.world.set_esdt_balance(
            MULTISIG_ADDRESS,
            b"WEGLD-123456",
            BigUint::from(100_000_000_000u64),
        );

        self.world.set_esdt_balance(
            MULTISIG_ADDRESS,
            b"ETH-123456",
            BigUint::from(100_000_000_000u64),
        );

        self.world.set_esdt_local_roles(
            ESDT_SAFE_ADDRESS,
            b"WEGLD-123456",
            &[EsdtLocalRole::Mint, EsdtLocalRole::Burn],
        );

        self.world.set_esdt_local_roles(
            ESDT_SAFE_ADDRESS,
            b"ETH-123456",
            &[EsdtLocalRole::Mint, EsdtLocalRole::Burn],
        );

        assert!(staked_relayers
            .to_vec()
            .contains(&RELAYER1_ADDRESS.to_managed_address()));
        assert!(staked_relayers
            .to_vec()
            .contains(&RELAYER2_ADDRESS.to_managed_address()));
    }

    fn deploy_contracts_config(&mut self) {
        self.multisig_deploy();
        self.safe_deploy();
        self.multi_transfer_deploy();
        self.bridge_proxy_deploy();
        self.bridged_tokens_wrapper_deploy();
        self.config_multisig();
    }
}

#[test]
fn config_test() {
    let mut state = MultiTransferTestState::new();

    state.deploy_contracts_config();
}

#[test]
fn ethereum_to_multiversx_call_data_empty_test() {
    let mut state = MultiTransferTestState::new();
    let token_amount = BigUint::from(76_000_000u64);

    state.deploy_contracts_config();

    let eth_tx = EthTxAsMultiValue::<StaticApi>::from((
        EthAddress {
            raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
        },
        ManagedAddress::from(USER1_ADDRESS.eval_to_array()),
        TokenIdentifier::from(WEGLD_TOKEN_ID),
        token_amount.clone(),
        1u64,
        ManagedOption::none(),
    ));

    let mut transfers: MultiValueEncoded<StaticApi, EthTxAsMultiValue<StaticApi>> =
        MultiValueEncoded::new();
    transfers.push(eth_tx);

    state
        .world
        .tx()
        .from(RELAYER1_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .propose_multi_transfer_esdt_batch(1u32, transfers)
        .run();

    state
        .world
        .tx()
        .from(RELAYER2_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .sign(1usize)
        .run();

    state
        .world
        .tx()
        .from(RELAYER1_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .perform_action_endpoint(1usize)
        .run();

    state
        .world
        .check_account(USER1_ADDRESS)
        .esdt_balance(WEGLD_TOKEN_ID, token_amount.clone());
}

#[test]
fn ethereum_to_multiversx_relayer_call_data_several_tx_test() {
    let mut state = MultiTransferTestState::new();
    let token_amount = BigUint::from(5_000u64);

    state.world.start_trace();

    state.deploy_contracts_config();

    let addr =
        Address::from_slice(b"erd1dyw7aysn0nwmuahvxnh2e0pm0kgjvs2gmfdxjgz3x0pet2nkvt8s7tkyrj");
    let eth_tx = EthTxAsMultiValue::<StaticApi>::from((
        EthAddress {
            raw_addr: ManagedByteArray::new_from_bytes(b"5d959e98ea73c35778ff"),
        },
        ManagedAddress::from(addr.clone()),
        TokenIdentifier::from("ETHUSDC-afa689"),
        token_amount.clone(),
        1u64,
        ManagedOption::none(),
    ));

    let eth_tx2 = EthTxAsMultiValue::<StaticApi>::from((
        EthAddress {
            raw_addr: ManagedByteArray::new_from_bytes(b"5d959e98ea73c35778ff"),
        },
        ManagedAddress::from(addr.clone()),
        TokenIdentifier::from("ETHUSDC-afa689"),
        token_amount.clone(),
        2u64,
        ManagedOption::none(),
    ));

    let call_data: CallData<StaticApi> = CallData {
        endpoint: ManagedBuffer::from(b"fund"),
        gas_limit: GAS_LIMIT,
        args: ManagedOption::none(),
    };
    let call_data = ManagedSerializer::new().top_encode_to_managed_buffer(&call_data);

    let eth_tx3 = EthTxAsMultiValue::<StaticApi>::from((
        EthAddress {
            raw_addr: ManagedByteArray::new_from_bytes(b"5d959e98ea73c35778ff"),
        },
        ManagedAddress::from(addr.clone()),
        TokenIdentifier::from("ETHUSDC-afa689"),
        token_amount.clone(),
        3u64,
        ManagedOption::some(call_data),
    ));

    let args = ManagedVec::from_single_item(ManagedBuffer::from(b"5"));
    let call_data2: CallData<StaticApi> = CallData {
        endpoint: ManagedBuffer::from(b"fund"),
        gas_limit: GAS_LIMIT,
        args: ManagedOption::some(args),
    };
    let call_data2 = ManagedSerializer::new().top_encode_to_managed_buffer(&call_data2);

    let eth_tx4 = EthTxAsMultiValue::<StaticApi>::from((
        EthAddress {
            raw_addr: ManagedByteArray::new_from_bytes(b"5d959e98ea73c35778ff"),
        },
        ManagedAddress::from(addr.clone()),
        TokenIdentifier::from("ETHUSDC-afa689"),
        token_amount.clone(),
        4u64,
        ManagedOption::some(call_data2),
    ));
    let mut transfers: MultiValueEncoded<StaticApi, EthTxAsMultiValue<StaticApi>> =
        MultiValueEncoded::new();
    transfers.push(eth_tx);
    transfers.push(eth_tx2);
    transfers.push(eth_tx3);
    transfers.push(eth_tx4);

    state
        .world
        .tx()
        .from(RELAYER1_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .propose_multi_transfer_esdt_batch(1u32, transfers)
        .run();

    state
        .world
        .tx()
        .from(RELAYER2_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .sign(1usize)
        .run();

    state
        .world
        .tx()
        .from(RELAYER1_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .perform_action_endpoint(1usize)
        .returns(ExpectError(4, "Invalid token or amount"))
        .run();

    state.world.write_scenario_trace(
        "scenarios/ethereum_to_multiversx_relayer_call_data_several_tx_test.scen.json",
    );
}

#[test]
fn ethereum_to_multiversx_relayer_query_test() {
    let mut state = MultiTransferTestState::new();
    let token_amount = BigUint::from(76_000_000_000u64);
    state.world.start_trace();

    state.deploy_contracts_config();

    let eth_tx = EthTxAsMultiValue::<StaticApi>::from((
        EthAddress {
            raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
        },
        ManagedAddress::from(USER1_ADDRESS.eval_to_array()),
        TokenIdentifier::from(WEGLD_TOKEN_ID),
        token_amount.clone(),
        1u64,
        ManagedOption::none(),
    ));

    let mut transfers: MultiValueEncoded<StaticApi, EthTxAsMultiValue<StaticApi>> =
        MultiValueEncoded::new();
    transfers.push(eth_tx);

    state
        .world
        .tx()
        .from(RELAYER1_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .propose_multi_transfer_esdt_batch(1u32, transfers.clone())
        .run();

    let was_transfer = state
        .world
        .query()
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .was_transfer_action_proposed(1u64, transfers.clone())
        .returns(ReturnsResult)
        .run();

    assert!(was_transfer);

    let get_action_id = state
        .world
        .query()
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .get_action_id_for_transfer_batch(1u64, transfers)
        .returns(ReturnsResult)
        .run();

    assert!(get_action_id == 1usize);

    state
        .world
        .tx()
        .from(RELAYER2_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .sign(1usize)
        .run();

    state
        .world
        .tx()
        .from(RELAYER1_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .perform_action_endpoint(1usize)
        .run();

    state
        .world
        .check_account(USER1_ADDRESS)
        .esdt_balance(WEGLD_TOKEN_ID, token_amount.clone());

    state
        .world
        .write_scenario_trace("scenarios/ethereum_to_multiversx_relayer_query_test.scen.json");
}

#[test]
fn ethereum_to_multiversx_relayer_query2_test() {
    let mut state = MultiTransferTestState::new();
    let token_amount = BigUint::from(5_000u64);
    state.world.start_trace();

    state.deploy_contracts_config();

    let addr =
        Address::from_slice(b"erd1dyw7aysn0nwmuahvxnh2e0pm0kgjvs2gmfdxjgz3x0pet2nkvt8s7tkyrj");

    const ADDR: [u8; 32] = hex!("691dee92137cddbe76ec34eeacbc3b7d91264148da5a69205133c395aa7662cf");

    let eth_tx = EthTxAsMultiValue::<StaticApi>::from((
        EthAddress {
            raw_addr: ManagedByteArray::new_from_bytes(b"5d959e98ea73c35778ff"),
        },
        ManagedAddress::from(ADDR),
        TokenIdentifier::from("ETHUSDC-afa689"),
        token_amount.clone(),
        1u64,
        ManagedOption::none(),
    ));

    let mut transfers: MultiValueEncoded<StaticApi, EthTxAsMultiValue<StaticApi>> =
        MultiValueEncoded::new();
    transfers.push(eth_tx);

    state
        .world
        .tx()
        .from(RELAYER1_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .propose_multi_transfer_esdt_batch(1u32, transfers.clone())
        .run();

    let was_transfer = state
        .world
        .query()
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .was_transfer_action_proposed(1u64, transfers.clone())
        .returns(ReturnsResult)
        .run();

    assert!(was_transfer);

    let get_action_id = state
        .world
        .query()
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .get_action_id_for_transfer_batch(1u64, transfers)
        .returns(ReturnsResult)
        .run();

    assert!(get_action_id == 1usize);

    state
        .world
        .tx()
        .from(RELAYER2_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .sign(1usize)
        .run();

    state
        .world
        .tx()
        .from(RELAYER1_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .perform_action_endpoint(1usize)
        .returns(ExpectError(4, "Invalid token or amount"))
        .run();

    state
        .world
        .write_scenario_trace("scenarios/ethereum_to_multiversx_relayer_query2_test.scen.json");
}

#[test]
fn ethereum_to_multiversx_tx_batch_ok_test() {
    let mut state = MultiTransferTestState::new();
    let token_amount = BigUint::from(76_000_000_000u64);
    state.world.start_trace();

    state.deploy_contracts_config();

    let mut args = ManagedVec::new();
    args.push(ManagedBuffer::from(&[5u8, 6u8]));
    args.push(ManagedBuffer::from(&[7u8, 8u8, 9u8]));
    args.push(ManagedBuffer::from(&[7u8, 8u8, 9u8, 10u8, 11u8]));

    let call_data: CallData<StaticApi> = CallData {
        endpoint: ManagedBuffer::from("add"),
        gas_limit: GAS_LIMIT,
        args: ManagedOption::some(args),
    };

    let call_data: ManagedBuffer<StaticApi> =
        ManagedSerializer::new().top_encode_to_managed_buffer(&call_data);

    let eth_tx1 = EthTxAsMultiValue::<StaticApi>::from((
        EthAddress {
            raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
        },
        ManagedAddress::from(USER1_ADDRESS.eval_to_array()),
        TokenIdentifier::from(WEGLD_TOKEN_ID),
        token_amount.clone(),
        1u64,
        ManagedOption::some(call_data.clone()),
    ));

    let eth_tx2 = EthTxAsMultiValue::<StaticApi>::from((
        EthAddress {
            raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
        },
        ManagedAddress::from(USER1_ADDRESS.eval_to_array()),
        TokenIdentifier::from(ETH_TOKEN_ID),
        token_amount.clone(),
        2u64,
        ManagedOption::some(call_data.clone()),
    ));

    let mut transfers: MultiValueEncoded<StaticApi, EthTxAsMultiValue<StaticApi>> =
        MultiValueEncoded::new();
    transfers.push(eth_tx1);
    transfers.push(eth_tx2);

    state
        .world
        .tx()
        .from(RELAYER1_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .propose_multi_transfer_esdt_batch(1u32, transfers)
        .run();

    state
        .world
        .tx()
        .from(RELAYER2_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .sign(1usize)
        .run();

    state
        .world
        .tx()
        .from(RELAYER1_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .perform_action_endpoint(1usize)
        .run();

    state
        .world
        .check_account(USER1_ADDRESS)
        .esdt_balance(WEGLD_TOKEN_ID, token_amount.clone())
        .esdt_balance(ETH_TOKEN_ID, token_amount.clone());

    state.world.write_scenario_trace(
        "scenarios/ethereum_to_multiversx_tx_batch_ok_call_data_encoded.scen.json",
    );
}

#[test]
fn ethereum_to_multiversx_tx_batch_rejected_test() {
    let mut state = MultiTransferTestState::new();
    let over_the_limit_token_amount = BigUint::from(101_000_000_000u64);

    state.deploy_contracts_config();

    let mut args = ManagedVec::new();
    args.push(ManagedBuffer::from(&[5u8]));

    let call_data: CallData<StaticApi> = CallData {
        endpoint: ManagedBuffer::from("add"),
        gas_limit: GAS_LIMIT,
        args: ManagedOption::some(args),
    };

    let call_data: ManagedBuffer<StaticApi> =
        ManagedSerializer::new().top_encode_to_managed_buffer(&call_data);

    let eth_tx1 = EthTxAsMultiValue::<StaticApi>::from((
        EthAddress {
            raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
        },
        ManagedAddress::from(BRIDGE_PROXY_ADDRESS.eval_to_array()),
        TokenIdentifier::from(WEGLD_TOKEN_ID),
        over_the_limit_token_amount.clone(),
        1u64,
        ManagedOption::some(call_data.clone()),
    ));

    let eth_tx2 = EthTxAsMultiValue::<StaticApi>::from((
        EthAddress {
            raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
        },
        ManagedAddress::from(BRIDGE_PROXY_ADDRESS.eval_to_array()),
        TokenIdentifier::from(ETH_TOKEN_ID),
        over_the_limit_token_amount.clone(),
        2u64,
        ManagedOption::some(call_data.clone()),
    ));

    let mut transfers: MultiValueEncoded<StaticApi, EthTxAsMultiValue<StaticApi>> =
        MultiValueEncoded::new();
    transfers.push(eth_tx1);
    transfers.push(eth_tx2);

    state
        .world
        .tx()
        .from(RELAYER1_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .propose_multi_transfer_esdt_batch(1u32, transfers)
        .run();

    state
        .world
        .tx()
        .from(RELAYER2_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .sign(1usize)
        .run();

    state
        .world
        .tx()
        .from(RELAYER1_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .perform_action_endpoint(1usize)
        .run();

    let refund_tx = state
        .world
        .query()
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .get_current_refund_batch()
        .returns(ReturnsResult)
        .run();

    assert!(refund_tx.is_none());

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .move_refund_batch_to_safe_from_child_contract()
        .run();
}

#[test]
fn init_test() {
    let mut state = MultiTransferTestState::new();

    let mut board: MultiValueEncoded<StaticApi, ManagedAddress<StaticApi>> =
        MultiValueEncoded::new();
    board.push(ManagedAddress::from(RELAYER1_ADDRESS.eval_to_array()));
    board.push(ManagedAddress::from(RELAYER2_ADDRESS.eval_to_array()));
    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .init(
            ESDT_SAFE_ADDRESS,
            MULTI_TRANSFER_ADDRESS,
            BRIDGE_PROXY_ADDRESS,
            BRIDGED_TOKENS_WRAPPER_ADDRESS,
            PRICE_AGGREGATOR_ADDRESS,
            1_000u64,
            5000u64,
            2usize,
            board.clone(),
        )
        .code(MULTISIG_CODE_PATH)
        .new_address(MULTISIG_ADDRESS)
        .returns(ExpectError(
            4,
            "slash amount must be less than or equal to required stake",
        ))
        .run();

    let mut board2: MultiValueEncoded<StaticApi, ManagedAddress<StaticApi>> =
        MultiValueEncoded::new();
    board2.push(ManagedAddress::from(RELAYER1_ADDRESS.eval_to_array()));
    board2.push(ManagedAddress::from(RELAYER1_ADDRESS.eval_to_array()));
    let multisig2 = TestSCAddress::new("multisig2");
    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .init(
            ESDT_SAFE_ADDRESS,
            MULTI_TRANSFER_ADDRESS,
            BRIDGE_PROXY_ADDRESS,
            BRIDGED_TOKENS_WRAPPER_ADDRESS,
            PRICE_AGGREGATOR_ADDRESS,
            1_000u64,
            500u64,
            2usize,
            board2.clone(),
        )
        .code(MULTISIG_CODE_PATH)
        .new_address(multisig2)
        .returns(ExpectError(4, "duplicate board member"))
        .run();
}

#[test]
fn upgrade_test() {
    let mut state = MultiTransferTestState::new();

    state.deploy_contracts_config();

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .upgrade(
            ESDT_SAFE_ADDRESS,
            MULTI_TRANSFER_ADDRESS,
            BRIDGE_PROXY_ADDRESS,
            BRIDGED_TOKENS_WRAPPER_ADDRESS,
            PRICE_AGGREGATOR_ADDRESS,
        )
        .code(MULTISIG_CODE_PATH)
        .run();
}

#[test]
fn multisig_non_board_member_interaction_test() {
    let mut state = MultiTransferTestState::new();
    let token_amount = BigUint::from(76_000_000_000u64);

    state.deploy_contracts_config();

    let eth_tx = EthTxAsMultiValue::<StaticApi>::from((
        EthAddress {
            raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
        },
        ManagedAddress::from(USER1_ADDRESS.eval_to_array()),
        TokenIdentifier::from(WEGLD_TOKEN_ID),
        token_amount.clone(),
        1u64,
        ManagedOption::none(),
    ));

    let mut transfers: MultiValueEncoded<StaticApi, EthTxAsMultiValue<StaticApi>> =
        MultiValueEncoded::new();
    transfers.push(eth_tx);

    state
        .world
        .tx()
        .from(NON_BOARD_MEMEBER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .propose_multi_transfer_esdt_batch(1u32, transfers.clone())
        .returns(ExpectError(4, "only board members can propose"))
        .run();

    let mut tx_batch_status: MultiValueEncoded<StaticApi, TransactionStatus> =
        MultiValueEncoded::<StaticApi, TransactionStatus>::new();
    tx_batch_status.push(TransactionStatus::InProgress);

    state
        .world
        .tx()
        .from(RELAYER1_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .propose_multi_transfer_esdt_batch(1u32, transfers)
        .run();
}

#[test]
fn multisig_insuficient_signatures_test() {
    let mut state = MultiTransferTestState::new();
    let token_amount = BigUint::from(76_000_000_000u64);

    state.deploy_contracts_config();

    let eth_tx = EthTxAsMultiValue::<StaticApi>::from((
        EthAddress {
            raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
        },
        ManagedAddress::from(USER1_ADDRESS.eval_to_array()),
        TokenIdentifier::from(WEGLD_TOKEN_ID),
        token_amount.clone(),
        1u64,
        ManagedOption::none(),
    ));

    let mut transfers: MultiValueEncoded<StaticApi, EthTxAsMultiValue<StaticApi>> =
        MultiValueEncoded::new();
    transfers.push(eth_tx);

    state
        .world
        .tx()
        .from(RELAYER1_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .propose_multi_transfer_esdt_batch(1u32, transfers)
        .run();

    state
        .world
        .tx()
        .from(RELAYER1_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .perform_action_endpoint(1usize)
        .returns(ExpectError(4, "quorum has not been reached"))
        .run();
}

#[test]
fn multisig_non_board_member_sign_test() {
    let mut state = MultiTransferTestState::new();
    let token_amount = BigUint::from(76_000_000_000u64);

    state.deploy_contracts_config();

    let eth_tx = EthTxAsMultiValue::<StaticApi>::from((
        EthAddress {
            raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
        },
        ManagedAddress::from(USER1_ADDRESS.eval_to_array()),
        TokenIdentifier::from(WEGLD_TOKEN_ID),
        token_amount.clone(),
        1u64,
        ManagedOption::none(),
    ));

    let mut transfers: MultiValueEncoded<StaticApi, EthTxAsMultiValue<StaticApi>> =
        MultiValueEncoded::new();
    transfers.push(eth_tx);

    state
        .world
        .tx()
        .from(RELAYER1_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .propose_multi_transfer_esdt_batch(1u32, transfers)
        .run();

    state
        .world
        .tx()
        .from(NON_BOARD_MEMEBER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .sign(1usize)
        .returns(ExpectError(4, "only board members can sign"))
        .run();

    state
        .world
        .tx()
        .from(RELAYER1_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .perform_action_endpoint(1usize)
        .returns(ExpectError(4, "quorum has not been reached"))
        .run();
}

#[test]
fn test_distribute_fees_from_child_contracts_success() {
    let mut state = MultiTransferTestState::new();

    state.deploy_contracts_config();

    let dest_address1 = USER1_ADDRESS.to_managed_address();
    let dest_address2 = USER2_ADDRESS.to_managed_address();

    const PERCENTAGE_TOTAL: u32 = 10000;

    let percentage1 = 6000;
    let percentage2 = 4000;

    let mut dest_address_percentage_pairs: MultiValueEncoded<
        StaticApi,
        MultiValue2<ManagedAddress<StaticApi>, u32>,
    > = MultiValueEncoded::new();

    dest_address_percentage_pairs.push(MultiValue2::from((dest_address1.clone(), percentage1)));
    dest_address_percentage_pairs.push(MultiValue2::from((dest_address2.clone(), percentage2)));

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .distribute_fees_from_child_contracts(dest_address_percentage_pairs)
        .run();
}

#[test]
fn test_distribute_fees_from_child_contracts_invalid_percentage_sum() {
    let mut state = MultiTransferTestState::new();

    state.deploy_contracts_config();
    let dest_address1 = USER1_ADDRESS.to_managed_address();
    let dest_address2 = USER2_ADDRESS.to_managed_address();

    let percentage1 = 5000;
    let percentage2 = 4000;

    let mut dest_address_percentage_pairs: MultiValueEncoded<
        StaticApi,
        MultiValue2<ManagedAddress<StaticApi>, u32>,
    > = MultiValueEncoded::new();

    dest_address_percentage_pairs.push(MultiValue2::from((dest_address1.clone(), percentage1)));
    dest_address_percentage_pairs.push(MultiValue2::from((dest_address2.clone(), percentage2)));

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .distribute_fees_from_child_contracts(dest_address_percentage_pairs)
        .returns(ExpectError(4, "Percentages do not add up to 100%"))
        .run();
}

#[test]
fn test_distribute_fees_from_child_contracts_with_sc_address() {
    let mut state = MultiTransferTestState::new();

    state.deploy_contracts_config();

    let dest_address1 = USER1_ADDRESS.to_managed_address();
    let dest_address2 = ESDT_SAFE_ADDRESS.to_managed_address();
    let percentage1 = 6000;
    let percentage2 = 4000;

    let mut dest_address_percentage_pairs: MultiValueEncoded<
        StaticApi,
        MultiValue2<ManagedAddress<StaticApi>, u32>,
    > = MultiValueEncoded::new();

    dest_address_percentage_pairs.push(MultiValue2::from((dest_address1.clone(), percentage1)));
    dest_address_percentage_pairs.push(MultiValue2::from((dest_address2.clone(), percentage2)));

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .distribute_fees_from_child_contracts(dest_address_percentage_pairs)
        .returns(ExpectError(
            4,
            "Cannot transfer to smart contract dest_address",
        ))
        .run();
}

#[test]
fn test_unstake_successful_board_member() {
    let mut state = MultiTransferTestState::new();

    state.deploy_contracts_config();
    let stake_amount = BigUint::from(1_000u64);

    state
        .world
        .tx()
        .from(RELAYER1_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .stake()
        .egld(&stake_amount)
        .run();

    let remaining_stake = state
        .world
        .query()
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .amount_staked(RELAYER1_ADDRESS.to_managed_address())
        .returns(ReturnsResult)
        .run();

    let unstake_amount = BigUint::from(500u64);
    state
        .world
        .tx()
        .from(RELAYER1_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .unstake(unstake_amount.clone())
        .run();

    let remaining_stake = state
        .world
        .query()
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .amount_staked(RELAYER1_ADDRESS.to_managed_address())
        .returns(ReturnsResult)
        .run();

    let expected_remaining_stake = BigUint::from(2000u64) - &unstake_amount;
    assert_eq!(remaining_stake, expected_remaining_stake);

    state
        .world
        .check_account(RELAYER1_ADDRESS)
        .balance(unstake_amount.to_u64().unwrap());
}

#[test]
fn test_unstake_more_than_staked_amount() {
    let mut state = MultiTransferTestState::new();

    state.deploy_contracts_config();

    let unstake_amount = BigUint::from(1_500u64);
    state
        .world
        .tx()
        .from(RELAYER1_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .unstake(unstake_amount)
        .returns(ExpectError(4, "can't unstake more than amount staked"))
        .run();
}

#[test]
fn test_unstake_below_required_stake_board_member() {
    let mut state = MultiTransferTestState::new();

    state.deploy_contracts_config();

    let additional_unstake_amount = BigUint::from(600u64);
    state
        .world
        .tx()
        .from(RELAYER1_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .unstake(additional_unstake_amount)
        .returns(ExpectError(
            4,
            "can't unstake, must keep minimum amount as insurance",
        ))
        .run();
}

#[test]
fn test_unstake_updates_amount_staked_correctly() {
    let mut state = MultiTransferTestState::new();

    state.deploy_contracts_config();

    let stake_amount_relayer1 = BigUint::from(1_000u64);
    state
        .world
        .tx()
        .from(RELAYER1_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .stake()
        .egld(&stake_amount_relayer1)
        .run();

    let stake_amount_relayer2 = BigUint::from(1_000u64);
    state
        .world
        .tx()
        .from(RELAYER2_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .stake()
        .egld(&stake_amount_relayer2)
        .run();

    let unstake_amount_relayer1 = BigUint::from(200u64);
    state
        .world
        .tx()
        .from(RELAYER1_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .unstake(unstake_amount_relayer1.clone())
        .run();

    let remaining_stake_relayer1 = state
        .world
        .query()
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .amount_staked(RELAYER1_ADDRESS.to_managed_address())
        .returns(ReturnsResult)
        .run();

    let expected_remaining_stake_relayer1 = &BigUint::from(2_000u64) - &unstake_amount_relayer1;
    assert_eq!(remaining_stake_relayer1, expected_remaining_stake_relayer1);

    let remaining_stake_relayer2 = state
        .world
        .query()
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .amount_staked(RELAYER2_ADDRESS.to_managed_address())
        .returns(ReturnsResult)
        .run();

    assert_eq!(remaining_stake_relayer2, BigUint::from(2_000u64));
}

#[test]
fn test_propose_esdt_safe_set_current_transaction_batch_status_success() {
    let mut state = MultiTransferTestState::new();

    state.deploy_contracts_config();

    let esdt_safe_address = ESDT_SAFE_ADDRESS;
    let token_id = WEGLD_TOKEN_ID;
    let amount = BigUint::<StaticApi>::from(1_000u64);
    let destination = USER1_ADDRESS.to_managed_address::<StaticApi>();
    let nonce = 1u64;
}

#[test] //TODO: Implement this test
fn test_propose_esdt_safe_set_current_transaction_batch_status_already_proposed() {
    let mut state = MultiTransferTestState::new();

    state.deploy_contracts_config();

    let esdt_safe_address = ESDT_SAFE_ADDRESS;
    let token_id = WEGLD_TOKEN_ID;
    let amount = BigUint::<StaticApi>::from(1_000u64);
    let destination = USER1_ADDRESS.to_managed_address::<StaticApi>();
    let nonce = 1u64;

    let eth_tx = EthTransaction {
        from: EthAddress::<StaticApi>::zero(),
        to: ManagedAddress::from(USER1_ADDRESS.eval_to_array()),
        token_id: TokenIdentifier::from(WEGLD_TOKEN_ID),
        amount: BigUint::from(1000u64),
        tx_nonce: 1u64,
        call_data: ManagedOption::none(),
    };
}

#[test] //TODO: Fix this test
fn test_propose_esdt_safe_set_current_transaction_batch_status_wrong_batch_id() {
    let mut state = MultiTransferTestState::new();

    state.deploy_contracts_config();

    let esdt_safe_address = ESDT_SAFE_ADDRESS;
    let token_id = WEGLD_TOKEN_ID;
    let amount = BigUint::<StaticApi>::from(1_000u64);
    let destination = USER1_ADDRESS.to_managed_address::<StaticApi>();
    let nonce = 1u64;

    let statuses: MultiValueEncoded<StaticApi, TransactionStatus> =
        MultiValueEncoded::from(ManagedVec::from_single_item(TransactionStatus::Pending));

    // state
    //     .world
    //     .tx()
    //     .from(MULTISIG_ADDRESS)
    //     .to(ESDT_SAFE_ADDRESS)
    //     .typed(esdt_safe_proxy::EsdtSafeProxy)
    //     .create_transaction(
    //         EthAddress::zero(),
    //         OptionalValue::None::<sc_proxies::esdt_safe_proxy::RefundInfo<StaticApi>>,
    //     )
    //     .egld_or_single_esdt(
    //         &EgldOrEsdtTokenIdentifier::esdt(token_id),
    //         0,
    //         &BigUint::from(amount),
    //     )
    //     .returns(ReturnsResult)
    //     .run();

    // state
    //     .world
    //     .tx()
    //     .from(RELAYER1_ADDRESS)
    //     .to(MULTISIG_ADDRESS)
    //     .typed(multisig_proxy::MultisigProxy)
    //     .propose_esdt_safe_set_current_transaction_batch_status(5u64, statuses)
    //     .returns(ExpectError(4, "Can only propose for next batch ID"))
    //     .run();
}

#[test]
fn test_init_supply_from_child_contract_success() {
    let mut state = MultiTransferTestState::new();

    state.deploy_contracts_config();

    let token_id = NATIVE_TOKEN_ID;
    let amount = BigUint::from(1_000u64);

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .init_supply_from_child_contract(TokenIdentifier::from(token_id), amount.clone())
        .single_esdt(&TokenIdentifier::from(token_id), 0, &amount.clone())
        .run();

    state
        .world
        .check_account(ESDT_SAFE_ADDRESS)
        .esdt_balance(token_id, amount.clone());
}

#[test]
fn test_add_unprocessed_refund_tx_to_batch_success() {
    let mut state = MultiTransferTestState::new();

    let mut board: MultiValueEncoded<StaticApi, ManagedAddress<StaticApi>> =
        MultiValueEncoded::new();
    board.push(ManagedAddress::from(RELAYER1_ADDRESS.eval_to_array()));
    board.push(ManagedAddress::from(RELAYER2_ADDRESS.eval_to_array()));
    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .init(
            ESDT_SAFE_ADDRESS,
            MOCK_MULTI_TRANSFER_ADDRESS,
            BRIDGE_PROXY_ADDRESS,
            BRIDGED_TOKENS_WRAPPER_ADDRESS,
            PRICE_AGGREGATOR_ADDRESS,
            1_000u64,
            500u64,
            2usize,
            board,
        )
        .code(MULTISIG_CODE_PATH)
        .new_address(MULTISIG_ADDRESS)
        .run();

    state.mock_multi_transfer_deploy();

    let tx_id = 1u64;

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .add_unprocessed_refund_tx_to_batch(tx_id)
        .run();
}

#[test]
fn test_withdraw_slashed_amount_success() {
    let mut state = MultiTransferTestState::new();

    state.deploy_contracts_config();

    let slashed_amount = BigUint::from(500u64);
    state.world.set_esdt_balance(
        MULTISIG_ADDRESS,
        WEGLD_TOKEN_ID.as_bytes(),
        slashed_amount.clone(),
    );

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .slash_board_member(RELAYER1_ADDRESS.to_managed_address())
        .run();

    let remaining_stake = state
        .world
        .query()
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .amount_staked(RELAYER1_ADDRESS.to_managed_address())
        .returns(ReturnsResult)
        .run();

    assert_eq!(remaining_stake, BigUint::from(500u64));

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .withdraw_slashed_amount()
        .run();

    let remaining_slashed_amount = state
        .world
        .query()
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .slash_amount()
        .returns(ReturnsResult)
        .run();
    assert_eq!(remaining_slashed_amount, BigUint::from(500u64));
}

#[test] // TODO: Fix this test
fn test_perform_action_endpoint_set_current_transaction_batch_status_success() {
    let mut state = MultiTransferTestState::new();

    state.deploy_contracts_config();

    let esdt_safe_address = ESDT_SAFE_ADDRESS;
    let token_id = WEGLD_TOKEN_ID;
    let amount: BigUint<StaticApi> = BigUint::from(1_000u64);
    let destination: ManagedAddress<StaticApi> = USER1_ADDRESS.to_managed_address();
    let nonce = 1u64;

    // let eth_tx = EthTransaction {
    //     from: EthAddress::<StaticApi>::zero(),
    //     to: ManagedAddress::from(USER1_ADDRESS.eval_to_array()),
    //     token_id: TokenIdentifier::from(WEGLD_TOKEN_ID),
    //     amount: BigUint::from(1000u64),
    //     tx_nonce: 1u64,
    //     call_data: ManagedOption::none(),
    // };

    // let mut txs = ManagedVec::new();
    // txs.push(eth_tx.clone());

    // state
    //     .world
    //     .tx()
    //     .from(MULTISIG_ADDRESS)
    //     .to(esdt_safe_address)
    //     .typed(esdt_safe_proxy::EsdtSafeProxy)
    //     .set_transaction_batch_status(1, statuses.clone())
    //     .run();

    // let current_batch: OptionalValue<TxBatchSplitInFields<StaticApi>> = state
    //     .world
    //     .query()
    //     .to(esdt_safe_address)
    //     .typed(esdt_safe_proxy::EsdtSafeProxy)
    //     .get_current_tx_batch()
    //     .returns(ReturnsResult)
    //     .run();

    // let (current_batch_id, _current_batch_transactions) = match current_batch {
    //     OptionalValue::Some(batch) => batch.into_tuple(),
    //     OptionalValue::None => panic!("No current batch found in EsdtSafe"),
    // };

    // let statuses = MultiValueEncoded::from(vec![TransactionStatus::Success]);

    // let action_id: usize = state
    //     .world
    //     .tx()
    //     .from(RELAYER1_ADDRESS)
    //     .to(MULTISIG_ADDRESS)
    //     .typed(multisig_proxy::MultisigProxy)
    //     .propose_esdt_safe_set_current_transaction_batch_status(current_batch_id, statuses.clone())
    //     .returns(ReturnsResult)
    //     .run();

    // state
    //     .world
    //     .tx()
    //     .from(RELAYER2_ADDRESS) // Another board member
    //     .to(MULTISIG_ADDRESS)
    //     .typed(multisig_proxy::MultisigProxy)
    //     .sign(action_id)
    //     .run();

    state
        .world
        .tx()
        .from(RELAYER1_ADDRESS) // Board member
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .perform_action_endpoint(1usize)
        .run();

    // let action_exists = state
    //     .world
    //     .query()
    //     .to(MULTISIG_ADDRESS)
    //     .typed(multisig_proxy::MultisigProxy)
    //     .action_exists(action_id)
    //     .returns(ReturnsResult)
    //     .run();

    // assert!(!action_exists);

    // let batch_statuses: MultiValueEncoded<StaticApi, TransactionStatus> = state
    //     .world
    //     .query()
    //     .to(esdt_safe_address)
    //     .typed(esdt_safe_proxy::EsdtSafeProxy)
    //     .get_transaction_batch_status(current_batch_id)
    //     .returns(ReturnsResult)
    //     .run();

    // assert_eq!(batch_statuses.to_vec(), statuses.to_vec());
}

#[test] // TODO: Fix this test
fn test_withdraw_refund_fees_for_ethereum_success() {
    let mut state = MultiTransferTestState::new();

    let mut board: MultiValueEncoded<StaticApi, ManagedAddress<StaticApi>> =
        MultiValueEncoded::new();
    board.push(ManagedAddress::from(RELAYER1_ADDRESS.eval_to_array()));
    board.push(ManagedAddress::from(RELAYER2_ADDRESS.eval_to_array()));
    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .init(
            MOCK_ESDT_SAFE_ADDRESS,
            MULTI_TRANSFER_ADDRESS,
            BRIDGE_PROXY_ADDRESS,
            BRIDGED_TOKENS_WRAPPER_ADDRESS,
            PRICE_AGGREGATOR_ADDRESS,
            1_000u64,
            500u64,
            2usize,
            board,
        )
        .code(MULTISIG_CODE_PATH)
        .new_address(MULTISIG_ADDRESS)
        .run();
    state.mock_safe_deploy();

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .withdraw_refund_fees_for_ethereum(TokenIdentifier::from(WEGLD_TOKEN_ID))
        .run();
}

#[test]
fn test_withdraw_transaction_fees_success() {
    let mut state = MultiTransferTestState::new();

    let mut board: MultiValueEncoded<StaticApi, ManagedAddress<StaticApi>> =
        MultiValueEncoded::new();
    board.push(ManagedAddress::from(RELAYER1_ADDRESS.eval_to_array()));
    board.push(ManagedAddress::from(RELAYER2_ADDRESS.eval_to_array()));
    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .init(
            MOCK_ESDT_SAFE_ADDRESS,
            MULTI_TRANSFER_ADDRESS,
            BRIDGE_PROXY_ADDRESS,
            BRIDGED_TOKENS_WRAPPER_ADDRESS,
            PRICE_AGGREGATOR_ADDRESS,
            1_000u64,
            500u64,
            2usize,
            board,
        )
        .code(MULTISIG_CODE_PATH)
        .new_address(MULTISIG_ADDRESS)
        .run();
    state.mock_safe_deploy();

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .withdraw_transaction_fees(TokenIdentifier::from(WEGLD_TOKEN_ID))
        .run();
}

#[test]
fn test_upgrade_child_contract_from_source_success() {
    let mut state = MultiTransferTestState::new();

    state.deploy_contracts_config();

    let child_sc_address = MULTI_TRANSFER_ADDRESS.to_managed_address();

    state
        .world
        .tx()
        .from(MULTISIG_ADDRESS)
        .typed(multi_transfer_esdt_proxy::MultiTransferEsdtProxy)
        .init()
        .code(MOCK_MULTI_TRANSFER_PATH_EXPR)
        .new_address(MOCK_MULTI_TRANSFER_ADDRESS)
        .run();

    let init_args = MultiValueEncoded::new();

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .upgrade_child_contract_from_source(
            child_sc_address.clone(),
            MOCK_MULTI_TRANSFER_ADDRESS.clone(),
            false,
            init_args.clone(),
        )
        .run();

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .upgrade_child_contract_from_source(
            child_sc_address.clone(),
            MOCK_MULTI_TRANSFER_ADDRESS.clone(),
            true,
            init_args.clone(),
        )
        .run();

    // how to check ?
}

#[test]
fn test_add_board_member_success() {
    let mut state = MultiTransferTestState::new();

    state.deploy_contracts_config();

    let num_board_members = state
        .world
        .query()
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .num_board_members()
        .returns(ReturnsResult)
        .run();

    assert_eq!(num_board_members, 2);

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .add_board_member_endpoint(USER1_ADDRESS.to_managed_address())
        .run();

    let num_board_members_after = state
        .world
        .query()
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .num_board_members()
        .returns(ReturnsResult)
        .run();

    assert_eq!(num_board_members_after, 3);
}

#[test]
fn test_remove_user_success() {
    let mut state = MultiTransferTestState::new();

    state.deploy_contracts_config();

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .add_board_member_endpoint(USER1_ADDRESS.to_managed_address())
        .run();

    let num_board_members = state
        .world
        .query()
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .num_board_members()
        .returns(ReturnsResult)
        .run();

    assert_eq!(num_board_members, 3);

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .remove_user(USER1_ADDRESS.to_managed_address())
        .run();

    let num_board_members = state
        .world
        .query()
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .num_board_members()
        .returns(ReturnsResult)
        .run();

    assert_eq!(num_board_members, 2);
}

#[test]
fn test_remove_user_cannot_remove_all() {
    let mut state = MultiTransferTestState::new();

    let mut board: MultiValueEncoded<StaticApi, ManagedAddress<StaticApi>> =
        MultiValueEncoded::new();
    board.push(ManagedAddress::from(RELAYER1_ADDRESS.eval_to_array()));
    board.push(ManagedAddress::from(RELAYER2_ADDRESS.eval_to_array()));
    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .init(
            ESDT_SAFE_ADDRESS,
            MULTI_TRANSFER_ADDRESS,
            BRIDGE_PROXY_ADDRESS,
            BRIDGED_TOKENS_WRAPPER_ADDRESS,
            PRICE_AGGREGATOR_ADDRESS,
            1_000u64,
            500u64,
            1usize,
            board,
        )
        .code(MULTISIG_CODE_PATH)
        .new_address(MULTISIG_ADDRESS)
        .run();
    state.safe_deploy();
    state.multi_transfer_deploy();
    state.bridge_proxy_deploy();
    state.bridged_tokens_wrapper_deploy();
    state.config_multisig();

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .remove_user(RELAYER1_ADDRESS.to_managed_address())
        .run();

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .remove_user(RELAYER2_ADDRESS.to_managed_address())
        .returns(ExpectError(4u64, "cannot remove all board members"))
        .run();
}

#[test]
fn test_remove_user_quorum_exceed_board_size() {
    let mut state = MultiTransferTestState::new();

    state.deploy_contracts_config();

    let num_board_members = state
        .world
        .query()
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .num_board_members()
        .returns(ReturnsResult)
        .run();

    assert_eq!(num_board_members, 2);

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .remove_user(RELAYER1_ADDRESS.to_managed_address())
        .returns(ExpectError(4u64, "quorum cannot exceed board size"))
        .run();

    let num_board_members_after = state
        .world
        .query()
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .num_board_members()
        .returns(ReturnsResult)
        .run();

    assert_eq!(num_board_members_after, 2);
}

#[test]
fn test_change_quorum_success() {
    let mut state = MultiTransferTestState::new();

    state.deploy_contracts_config();

    let initial_num_board_members: usize = state
        .world
        .query()
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .num_board_members()
        .returns(ReturnsResult)
        .run();

    assert!(initial_num_board_members >= 1);

    let new_quorum = 1usize;
    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .change_quorum(new_quorum)
        .run();

    let updated_quorum: usize = state
        .world
        .query()
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .quorum()
        .returns(ReturnsResult)
        .run();

    assert_eq!(updated_quorum, new_quorum);
}

#[test]
fn test_add_mapping_success() {
    let mut state = MultiTransferTestState::new();

    state.deploy_contracts_config();

    let token_id = TokenIdentifier::from(WEGLD_TOKEN_ID);

    let erc20_address = EthAddress::zero();

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .add_mapping(erc20_address.clone(), token_id.clone())
        .run();

    state
        .world
        .query()
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .erc20_address_for_token_id(token_id.clone())
        .returns(ExpectValue(erc20_address.clone()))
        .run();

    state
        .world
        .query()
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .token_id_for_erc20_address(erc20_address.clone())
        .returns(ExpectValue(token_id.clone()))
        .run();
}

#[test]
fn test_add_mapping_token_id_already_mapped() {
    let mut state = MultiTransferTestState::new();

    state.deploy_contracts_config();

    let token_id = TokenIdentifier::from(WEGLD_TOKEN_ID);

    let erc20_address = EthAddress::zero();

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .add_mapping(erc20_address.clone(), token_id.clone())
        .run();

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .add_mapping(erc20_address.clone(), token_id.clone())
        .returns(ExpectError(4u64, "Mapping already exists for token ID"))
        .run();
}

#[test]
fn test_add_mapping_erc20_address_already_mapped() {
    let mut state = MultiTransferTestState::new();

    state.deploy_contracts_config();

    let token_id = TokenIdentifier::from(WEGLD_TOKEN_ID);

    let erc20_address = EthAddress::zero();

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .add_mapping(erc20_address.clone(), token_id.clone())
        .run();

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .add_mapping(erc20_address.clone(), TokenIdentifier::from(ETH_TOKEN_ID))
        .returns(ExpectError(4u64, "Mapping already exists for ERC20 token"))
        .run();
}

#[test]
fn test_clear_mapping_success() {
    let mut state = MultiTransferTestState::new();

    state.deploy_contracts_config();

    let token_id = TokenIdentifier::from(WEGLD_TOKEN_ID);

    let erc20_address = EthAddress::zero();

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .add_mapping(erc20_address.clone(), token_id.clone())
        .run();
    state
        .world
        .query()
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .erc20_address_for_token_id(token_id.clone())
        .returns(ExpectValue(erc20_address.clone()))
        .run();

    state
        .world
        .query()
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .token_id_for_erc20_address(erc20_address.clone())
        .returns(ExpectValue(token_id.clone()))
        .run();

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .clear_mapping(erc20_address.clone(), token_id.clone())
        .run();

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .clear_mapping(erc20_address.clone(), token_id.clone())
        .returns(ExpectError(4u64, "Mapping does not exist for ERC20 token"))
        .run();

    let erc20_address2 = EthAddress {
        raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080900"),
    };

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .add_mapping(erc20_address2, token_id.clone())
        .run();

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .clear_mapping(erc20_address.clone(), token_id.clone())
        .returns(ExpectError(4u64, "Mapping does not exist for token id"))
        .run();
}

#[test]
fn test_clear_mapping_invalid_mapping() {
    let mut state = MultiTransferTestState::new();

    state.deploy_contracts_config();

    let token_id1 = TokenIdentifier::from(WEGLD_TOKEN_ID);
    let token_id2 = TokenIdentifier::from("OTHER-123456");

    let erc20_address1 = EthAddress {
        raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
    };

    let erc20_address2 = EthAddress {
        raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080900"),
    };

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .add_mapping(erc20_address1.clone(), token_id1.clone())
        .run();

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .add_mapping(erc20_address2.clone(), token_id2.clone())
        .run();

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .clear_mapping(erc20_address1.clone(), token_id2.clone())
        .returns(ExpectError(4, "Invalid mapping"))
        .run();
}

#[test]
fn test_pause_unpause_esdt_safe() {
    let mut state = MultiTransferTestState::new();

    state.deploy_contracts_config();

    let is_paused_before: bool = state
        .world
        .query()
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .paused_status()
        .returns(ReturnsResult)
        .run();

    assert!(!is_paused_before);

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .pause_esdt_safe()
        .run();

    let is_paused_after: bool = state
        .world
        .query()
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .paused_status()
        .returns(ReturnsResult)
        .run();

    assert!(is_paused_after);

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .unpause_esdt_safe()
        .run();

    let is_paused_after_unpause: bool = state
        .world
        .query()
        .to(ESDT_SAFE_ADDRESS)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .paused_status()
        .returns(ReturnsResult)
        .run();

    assert!(!is_paused_after_unpause);
}

#[test]
fn test_init_supply_functions_success() {
    let mut state = MultiTransferTestState::new();

    state.deploy_contracts_config();

    let amount = BigUint::from(100u64);

    state
        .world
        .set_esdt_balance(OWNER_ADDRESS, NATIVE_TOKEN_ID.as_bytes(), amount.clone());

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .init_supply_esdt_safe(NATIVE_TOKEN_ID.clone(), amount.clone())
        .single_esdt(&TokenIdentifier::from(NATIVE_TOKEN_ID), 0, &amount)
        .run();

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .init_supply_mint_burn_esdt_safe(ETH_TOKEN_ID.clone(), amount.clone(), amount.clone())
        .run();
}

#[test]
fn test_pause_unpause_proxy() {
    let mut state = MultiTransferTestState::new();

    state.deploy_contracts_config();

    let is_paused_before: bool = state
        .world
        .query()
        .to(BRIDGE_PROXY_ADDRESS)
        .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
        .paused_status()
        .returns(ReturnsResult)
        .run();

    assert!(!is_paused_before);

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .pause_proxy()
        .run();

    let is_paused_after: bool = state
        .world
        .query()
        .to(BRIDGE_PROXY_ADDRESS)
        .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
        .paused_status()
        .returns(ReturnsResult)
        .run();

    assert!(is_paused_after);

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .unpause_proxy()
        .run();

    let is_paused_after_unpause: bool = state
        .world
        .query()
        .to(BRIDGE_PROXY_ADDRESS)
        .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
        .paused_status()
        .returns(ReturnsResult)
        .run();

    assert!(!is_paused_after_unpause);
}

#[test]
fn test_change_esdt_safe_settings() {
    let mut state = MultiTransferTestState::new();

    state.deploy_contracts_config();

    let esdt_safe_address = ESDT_SAFE_ADDRESS;

    let new_gas_limit = BigUint::from(5_000_000u64);
    let token_id = TokenIdentifier::from(WEGLD_TOKEN_ID);
    let new_price_per_gas_unit = BigUint::from(100u64);
    let new_ticker = ManagedBuffer::from("WEGLD");

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .change_multiversx_to_eth_gas_limit(new_gas_limit.clone())
        .run();

    let updated_gas_limit = state
        .world
        .query()
        .to(esdt_safe_address)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .eth_tx_gas_limit()
        .returns(ReturnsResult)
        .run();

    assert_eq!(updated_gas_limit, new_gas_limit);

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .change_default_price_per_gas_unit(token_id.clone(), new_price_per_gas_unit.clone())
        .run();

    let updated_price_per_gas_unit = state
        .world
        .query()
        .to(esdt_safe_address)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .default_price_per_gas_unit(token_id.clone())
        .returns(ReturnsResult)
        .run();

    assert_eq!(updated_price_per_gas_unit, new_price_per_gas_unit);

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .change_token_ticker(token_id.clone(), new_ticker.clone())
        .run();
}

#[test]
fn test_esdt_safe_whitelist_management() {
    let mut state = MultiTransferTestState::new();

    state.deploy_contracts_config();

    let esdt_safe_address = ESDT_SAFE_ADDRESS;

    let token_id = TokenIdentifier::from("TEST-123456");
    let ticker = ManagedBuffer::from("TEST");
    let mint_burn_allowed = true;
    let is_native_token = false;
    let total_balance = BigUint::from(0u64);
    let mint_balance = BigUint::from(500_000u64);
    let burn_balance = BigUint::from(200_000u64);
    let opt_default_price_per_gas_unit = OptionalValue::Some(BigUint::from(100u64));

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .esdt_safe_add_token_to_whitelist(
            &token_id,
            ticker.clone(),
            mint_burn_allowed,
            is_native_token,
            &total_balance,
            &mint_balance,
            &burn_balance,
            opt_default_price_per_gas_unit.clone(),
        )
        .run();

    let tokens_whitelisted = state
        .world
        .query()
        .to(esdt_safe_address)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .token_whitelist()
        .returns(ReturnsResult)
        .run();

    assert!(tokens_whitelisted.to_vec().contains(&token_id));

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .esdt_safe_remove_token_from_whitelist(token_id.clone())
        .run();

    let tokens_whitelisted_after = state
        .world
        .query()
        .to(esdt_safe_address)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .token_whitelist()
        .returns(ReturnsResult)
        .run();

    assert!(!tokens_whitelisted_after.to_vec().contains(&token_id));
}

#[test]
fn test_esdt_safe_settings_management() {
    let mut state = MultiTransferTestState::new();

    state.deploy_contracts_config();

    let esdt_safe_address = ESDT_SAFE_ADDRESS;

    let new_max_tx_batch_size = 100usize;
    let new_max_tx_batch_block_duration = 600u64; // e.g., 600 blocks
    let token_id = TokenIdentifier::from("TEST-123456");
    let max_bridged_amount = BigUint::from(1_000_000u64);

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .esdt_safe_set_max_tx_batch_size(new_max_tx_batch_size)
        .run();

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .esdt_safe_set_max_tx_batch_block_duration(new_max_tx_batch_block_duration)
        .run();

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .esdt_safe_set_max_bridged_amount_for_token(token_id.clone(), max_bridged_amount.clone())
        .run();

    let updated_max_bridged_amount = state
        .world
        .query()
        .to(esdt_safe_address)
        .typed(esdt_safe_proxy::EsdtSafeProxy)
        .max_bridged_amount(token_id.clone())
        .returns(ReturnsResult)
        .run();

    assert_eq!(updated_max_bridged_amount, max_bridged_amount);
}

#[test]
fn test_multi_transfer_esdt_settings_management() {
    let mut state = MultiTransferTestState::new();

    state.deploy_contracts_config();

    let token_id = TokenIdentifier::from("TEST-123456");
    let max_bridged_amount = BigUint::from(1_000_000u64);
    let new_max_refund_tx_batch_size = 100usize;
    let new_max_refund_tx_batch_block_duration = 600u64;

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .multi_transfer_esdt_set_max_bridged_amount_for_token(
            token_id.clone(),
            max_bridged_amount.clone(),
        )
        .run();

    let updated_max_bridged_amount = state
        .world
        .query()
        .to(MULTI_TRANSFER_ADDRESS)
        .typed(multi_transfer_esdt_proxy::MultiTransferEsdtProxy)
        .max_bridged_amount(token_id.clone())
        .returns(ReturnsResult)
        .run();

    assert_eq!(updated_max_bridged_amount, max_bridged_amount);

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .multi_transfer_esdt_set_max_refund_tx_batch_size(new_max_refund_tx_batch_size)
        .run();

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .multi_transfer_esdt_set_max_refund_tx_batch_block_duration(
            new_max_refund_tx_batch_block_duration,
        )
        .run();
}
