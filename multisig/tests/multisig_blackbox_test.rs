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
        Address, BigUint, CodeMetadata, EsdtLocalRole, ManagedAddress, ManagedBuffer,
        ManagedByteArray, ManagedOption, ManagedType, ManagedVec, MultiValueEncoded,
        ReturnsNewManagedAddress, ReturnsResult, TestAddress, TestSCAddress, TestTokenIdentifier,
        TokenIdentifier,
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
    TxBatchSplitInFields,
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
            .account(MOCK_MULTI_TRANSFER_ADDRESS)
            .code(MOCK_MULTI_TRANSFER_PATH_EXPR);

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

    let percentage1 = 6000; // 60%
    let percentage2 = 4000; // 40%

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

    let percentage1 = 5000; // 50%
    let percentage2 = 4000; // 40%

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
    let percentage1 = 6000; // 60%
    let percentage2 = 4000; // 40%

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

// #[test]
// fn test_propose_esdt_safe_set_current_transaction_batch_status_success() {
//     let mut state = MultiTransferTestState::new();

//     // Deploy and configure contracts
//     state.deploy_contracts_config();

//     // Simulate a current transaction batch in EsdtSafe
//     let esdt_safe_address = ESDT_SAFE_ADDRESS;
//     let token_id = WEGLD_TOKEN_ID;
//     let amount = BigUint::from(1_000u64);
//     let destination = USER1_ADDRESS.to_managed_address();
//     let nonce = 1u64;

//     // Prepare a transaction to be added to the batch
//     let tx = transaction::Transaction {
//         token_identifier: TokenIdentifier::from(token_id),
//         amount: amount.clone(),
//         destination: destination.clone(),
//         tx_nonce: nonce,
//         call_data: ManagedOption::none(),
//         block_nonce: todo!(),
//         nonce,
//         from: todo!(),
//         to: todo!(),
//         is_refund_tx: todo!(),
//     };

//     let mut txs = ManagedVec::new();
//     txs.push(tx.clone());

//     // Add the transaction to EsdtSafe as a new batch
//     state
//         .world
//         .tx()
//         .from(MULTISIG_ADDRESS)
//         .to(esdt_safe_address)
//         .typed(esdt_safe_proxy::EsdtSafeProxy)
//         .add_transaction_batch(txs)
//         .run();

//     // Get the current batch ID from EsdtSafe
//     let current_batch: OptionalValue<TxBatchSplitInFields<StaticApi>> = state
//         .world
//         .query()
//         .to(esdt_safe_address)
//         .typed(esdt_safe_proxy::EsdtSafeProxy)
//         .get_current_tx_batch()
//         .returns(ReturnsResult)
//         .run();

//     let (current_batch_id, _current_batch_transactions) = match current_batch {
//         OptionalValue::Some(batch) => batch.into_tuple(),
//         OptionalValue::None => panic!("No current batch found in EsdtSafe"),
//     };

//     // Prepare the statuses vector matching the number of transactions
//     let statuses = MultiValueEncoded::from(vec![TransactionStatus::Success]);

//     // Propose setting the statuses
//     let action_id: usize = state
//         .world
//         .tx()
//         .from(RELAYER1_ADDRESS) // Board member
//         .to(MULTISIG_ADDRESS)
//         .typed(multisig_proxy::MultisigProxy)
//         .propose_esdt_safe_set_current_transaction_batch_status(current_batch_id, statuses.clone())
//         .returns(ReturnsResult)
//         .run();

//     // Verify that an action ID was returned
//     assert!(action_id > 0);

//     // Verify that the action was stored in the Multisig contract
//     let action: Action<StaticApi> = state
//         .world
//         .query()
//         .to(MULTISIG_ADDRESS)
//         .typed(multisig_proxy::MultisigProxy)
//         .get_action_data(action_id)
//         .returns(ReturnsResult)
//         .run();

//     assert!(matches!(
//         action,
//         Action::SetCurrentTransactionBatchStatus { .. }
//     ));
// }

// #[test]
// fn test_propose_esdt_safe_set_current_transaction_batch_status_already_proposed() {
//     let mut state = MultiTransferTestState::new();

//     // Deploy and configure contracts
//     state.deploy_contracts_config();

//     // Simulate a current transaction batch in EsdtSafe
//     let esdt_safe_address = ESDT_SAFE_ADDRESS;
//     let token_id = WEGLD_TOKEN_ID;
//     let amount = BigUint::from(1_000u64);
//     let destination = USER1_ADDRESS.to_managed_address();
//     let nonce = 1u64;

//     let tx = transaction::Transaction {
//         token_identifier: TokenIdentifier::from(token_id),
//         amount: amount.clone(),
//         destination: destination.clone(),
//         tx_nonce: nonce,
//         call_data: ManagedOption::none(),
//     };

//     let mut txs = ManagedVec::new();
//     txs.push(tx.clone());

//     state
//         .world
//         .tx()
//         .from(MULTISIG_ADDRESS)
//         .to(esdt_safe_address)
//         .typed(esdt_safe_proxy::EsdtSafeProxy)
//         .add_transaction_batch(txs)
//         .run();

//     let current_batch: OptionalValue<TxBatchSplitInFields<StaticApi>> = state
//         .world
//         .query()
//         .to(esdt_safe_address)
//         .typed(esdt_safe_proxy::EsdtSafeProxy)
//         .get_current_tx_batch()
//         .returns(ReturnsResult)
//         .run();

//     let (current_batch_id, _current_batch_transactions) = match current_batch {
//         OptionalValue::Some(batch) => batch.into_tuple(),
//         OptionalValue::None => panic!("No current batch found in EsdtSafe"),
//     };

//     // Prepare the statuses vector
//     let statuses = MultiValueEncoded::from(vec![TransactionStatus::Success]);

//     // First proposal
//     state
//         .world
//         .tx()
//         .from(RELAYER1_ADDRESS)
//         .to(MULTISIG_ADDRESS)
//         .typed(multisig_proxy::MultisigProxy)
//         .propose_esdt_safe_set_current_transaction_batch_status(current_batch_id, statuses.clone())
//         .run();

//     // Attempt to propose the same action again
//     state
//         .world
//         .tx()
//         .from(RELAYER2_ADDRESS)
//         .to(MULTISIG_ADDRESS)
//         .typed(multisig_proxy::MultisigProxy)
//         .propose_esdt_safe_set_current_transaction_batch_status(current_batch_id, statuses)
//         .returns(ExpectError(4, "Action already proposed"))
//         .run();
// }

// #[test]
// fn test_propose_esdt_safe_set_current_transaction_batch_status_wrong_batch_id() {
//     let mut state = MultiTransferTestState::new();

//     // Deploy and configure contracts
//     state.deploy_contracts_config();

//     // Simulate a current transaction batch in EsdtSafe
//     let esdt_safe_address = ESDT_SAFE_ADDRESS;
//     let token_id = WEGLD_TOKEN_ID;
//     let amount = BigUint::from(1_000u64);
//     let destination = USER1_ADDRESS.to_managed_address();
//     let nonce = 1u64;

//     let tx = transaction::Transaction {
//         token_identifier: TokenIdentifier::from(token_id),
//         amount: amount.clone(),
//         destination: destination.clone(),
//         tx_nonce: nonce,
//         call_data: ManagedOption::none(),
//     };

//     let mut txs = ManagedVec::new();
//     txs.push(tx.clone());

//     state
//         .world
//         .tx()
//         .from(MULTISIG_ADDRESS)
//         .to(esdt_safe_address)
//         .typed(esdt_safe_proxy::EsdtSafeProxy)
//         .add_transaction_batch(txs)
//         .run();

//     let current_batch: OptionalValue<TxBatchSplitInFields<StaticApi>> = state
//         .world
//         .query()
//         .to(esdt_safe_address)
//         .typed(esdt_safe_proxy::EsdtSafeProxy)
//         .get_current_tx_batch()
//         .returns(ReturnsResult)
//         .run();

//     let (current_batch_id, _current_batch_transactions) = match current_batch {
//         OptionalValue::Some(batch) => batch.into_tuple(),
//         OptionalValue::None => panic!("No current batch found in EsdtSafe"),
//     };

//     // Provide an incorrect batch ID
//     let incorrect_batch_id = current_batch_id + 1;

//     let statuses = MultiValueEncoded::from(vec![TransactionStatus::Success]);

//     // Attempt to propose setting the statuses with incorrect batch ID
//     state
//         .world
//         .tx()
//         .from(RELAYER1_ADDRESS)
//         .to(MULTISIG_ADDRESS)
//         .typed(multisig_proxy::MultisigProxy)
//         .propose_esdt_safe_set_current_transaction_batch_status(incorrect_batch_id, statuses)
//         .returns(ExpectError(
//             4,
//             "Current EsdtSafe tx batch does not have the provided ID",
//         ))
//         .run();
// }

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

    let tx_id = 1u64;

    // state
    //     .world
    //     .tx()
    //     .from(OWNER_ADDRESS)
    //     .to(MULTISIG_ADDRESS)
    //     .typed(multisig_proxy::MultisigProxy)
    //     .add_unprocessed_refund_tx_to_batch(tx_id)
    //     .run();
}
