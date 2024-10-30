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
    storage::mappers::SingleValue,
    types::{
        Address, BigUint, CodeMetadata, ManagedAddress, ManagedBuffer, ManagedByteArray,
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
    TxBatchSplitInFields,
};

const WEGLD_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("WEGLD-123456");
const ETH_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("ETH-123456");

const USER_ETHEREUM_ADDRESS: &[u8] = b"0x0102030405060708091011121314151617181920";

const GAS_LIMIT: u64 = 100_000_000;

const MULTISIG_CODE_PATH: MxscPath = MxscPath::new("output/multisig.mxsc.json");
const MULTI_TRANSFER_CODE_PATH: MxscPath =
    MxscPath::new("../multi-transfer-esdt/output/multi-transfer-esdt.mxsc.json");
const BRIDGE_PROXY_CODE_PATH: MxscPath =
    MxscPath::new("../bridge-proxy/output/bridge-proxy.mxsc.json");
const ESDT_SAFE_CODE_PATH: MxscPath = MxscPath::new("../esdt-safe/output/esdt-safe.mxsc.json");
const BRIDGED_TOKENS_WRAPPER_CODE_PATH: MxscPath =
    MxscPath::new("../bridged-tokens-wrapper/output/bridged-tokens-wrapper.mxsc.json");
const PRICE_AGGREGATOR_CODE_PATH: MxscPath =
    MxscPath::new("../price-aggregator/price-aggregator.mxsc.json");

const MULTISIG_ADDRESS: TestSCAddress = TestSCAddress::new("multisig");
const MULTI_TRANSFER_ADDRESS: TestSCAddress = TestSCAddress::new("multi-transfer");
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
            .account(USER1_ADDRESS)
            .nonce(1)
            .account(RELAYER1_ADDRESS)
            .nonce(1)
            .balance(1_000u64)
            .account(RELAYER2_ADDRESS)
            .nonce(1)
            .balance(1_000u64)
            .account(NON_BOARD_MEMEBER_ADDRESS)
            .nonce(1);

        let roles = vec![
            "ESDTRoleLocalMint".to_string(),
            "ESDTRoleLocalBurn".to_string(),
        ];
        world
            .account(ESDT_SAFE_ADDRESS)
            .esdt_roles(WEGLD_TOKEN_ID, roles.clone())
            .esdt_roles(ETH_TOKEN_ID, roles)
            .code(ESDT_SAFE_CODE_PATH)
            .owner(OWNER_ADDRESS);

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
            .from(OWNER_ADDRESS)
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
            .from(OWNER_ADDRESS)
            .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
            .init(OptionalValue::Some(MULTI_TRANSFER_ADDRESS.to_address()))
            .code(BRIDGE_PROXY_CODE_PATH)
            .new_address(BRIDGE_PROXY_ADDRESS)
            .run();

        self
    }

    fn safe_deploy(&mut self, price_aggregator_contract_address: Address) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .upgrade(
                ManagedAddress::zero(),
                MULTI_TRANSFER_ADDRESS.to_address(),
                BRIDGE_PROXY_ADDRESS.to_address(),
                ESDT_SAFE_ETH_TX_GAS_LIMIT,
            )
            .code(ESDT_SAFE_CODE_PATH)
            .run();

        self
    }

    fn config_multisig(&mut self) {
        self.world
            .tx()
            .from(MULTISIG_ADDRESS)
            .to(MULTI_TRANSFER_ADDRESS)
            .typed(multi_transfer_esdt_proxy::MultiTransferEsdtProxy)
            .set_wrapping_contract_address(OptionalValue::Some(
                BRIDGED_TOKENS_WRAPPER_ADDRESS.to_address(),
            ))
            .run();

        self.world
            .tx()
            .from(MULTISIG_ADDRESS)
            .to(MULTI_TRANSFER_ADDRESS)
            .typed(multi_transfer_esdt_proxy::MultiTransferEsdtProxy)
            .set_bridge_proxy_contract_address(OptionalValue::Some(
                BRIDGE_PROXY_ADDRESS.to_address(),
            ))
            .run();

        self.world
            .tx()
            .from(MULTISIG_ADDRESS)
            .to(MULTI_TRANSFER_ADDRESS)
            .typed(multi_transfer_esdt_proxy::MultiTransferEsdtProxy)
            .set_esdt_safe_contract_address(OptionalValue::Some(ESDT_SAFE_ADDRESS.to_address()))
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

        self.world
            .tx()
            .from(OWNER_ADDRESS)
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
            .from(OWNER_ADDRESS)
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

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(MULTISIG_ADDRESS)
            .typed(multisig_proxy::MultisigProxy)
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
            .typed(multisig_proxy::MultisigProxy)
            .unpause_endpoint()
            .run();

        self.world
            .tx()
            .from(OWNER_ADDRESS)
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

        assert!(staked_relayers
            .to_vec()
            .contains(&RELAYER1_ADDRESS.to_managed_address()));
        assert!(staked_relayers
            .to_vec()
            .contains(&RELAYER2_ADDRESS.to_managed_address()));
    }
}

#[test]
fn config_test() {
    let mut state = MultiTransferTestState::new();

    state.multisig_deploy();
    state.safe_deploy(Address::zero());
    state.multi_transfer_deploy();
    state.bridge_proxy_deploy();
    state.bridged_tokens_wrapper_deploy();
    state.config_multisig();
}

#[test]
fn ethereum_to_multiversx_call_data_empty_test() {
    let mut state = MultiTransferTestState::new();
    let token_amount = BigUint::from(76_000_000_000u64);

    state.multisig_deploy();
    state.safe_deploy(Address::zero());
    state.multi_transfer_deploy();
    state.bridge_proxy_deploy();
    state.bridged_tokens_wrapper_deploy();
    state.config_multisig();

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

    state.multisig_deploy();
    state.safe_deploy(Address::zero());
    state.multi_transfer_deploy();
    state.bridge_proxy_deploy();
    state.bridged_tokens_wrapper_deploy();
    state.config_multisig();

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

    state.multisig_deploy();
    state.safe_deploy(Address::zero());
    state.multi_transfer_deploy();
    state.bridge_proxy_deploy();
    state.bridged_tokens_wrapper_deploy();
    state.config_multisig();

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

    state.multisig_deploy();
    state.safe_deploy(Address::zero());
    state.multi_transfer_deploy();
    state.bridge_proxy_deploy();
    state.bridged_tokens_wrapper_deploy();
    state.config_multisig();

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

    state.multisig_deploy();
    state.safe_deploy(Address::zero());
    state.multi_transfer_deploy();
    state.bridge_proxy_deploy();
    state.bridged_tokens_wrapper_deploy();
    state.config_multisig();

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

    state.multisig_deploy();
    state.safe_deploy(Address::zero());
    state.multi_transfer_deploy();
    state.bridge_proxy_deploy();
    state.bridged_tokens_wrapper_deploy();
    state.config_multisig();

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
fn multisig_non_board_member_interaction_test() {
    let mut state = MultiTransferTestState::new();
    let token_amount = BigUint::from(76_000_000_000u64);

    state.multisig_deploy();
    state.safe_deploy(Address::zero());
    state.multi_transfer_deploy();
    state.bridge_proxy_deploy();
    state.bridged_tokens_wrapper_deploy();
    state.config_multisig();

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

    state.multisig_deploy();
    state.safe_deploy(Address::zero());
    state.multi_transfer_deploy();
    state.bridge_proxy_deploy();
    state.bridged_tokens_wrapper_deploy();
    state.config_multisig();

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

    state.multisig_deploy();
    state.safe_deploy(Address::zero());
    state.multi_transfer_deploy();
    state.bridge_proxy_deploy();
    state.bridged_tokens_wrapper_deploy();
    state.config_multisig();

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
