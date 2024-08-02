#![allow(unused)]

use bridge_proxy::{bridge_proxy_contract_proxy, config::ProxyTrait as _, ProxyTrait as _};
use esdt_safe::{EsdtSafe, ProxyTrait as _};
use multi_transfer_esdt::{bridged_tokens_wrapper_proxy, multi_transfer_proxy, ProxyTrait as _};

use multisig::{
    __endpoints_5__::multi_transfer_esdt_address, esdt_safe_proxy, multi_transfer_esdt_proxy,
    multisig_proxy,
};
use multiversx_sc::{
    api::{HandleConstraints, ManagedTypeApi},
    codec::{
        multi_types::{MultiValueVec, OptionalValue},
        Empty,
    },
    contract_base::ManagedSerializer,
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
use token_module::ProxyTrait as _;
use transaction::{CallData, EthTransaction, TxBatchSplitInFields};

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
            .balance(1_000u64);

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
            .typed(multi_transfer_proxy::MultiTransferEsdtProxy)
            .set_wrapping_contract_address(OptionalValue::Some(
                BRIDGED_TOKENS_WRAPPER_ADDRESS.to_address(),
            ))
            .run();

        self.world
            .tx()
            .from(MULTISIG_ADDRESS)
            .to(MULTI_TRANSFER_ADDRESS)
            .typed(multi_transfer_proxy::MultiTransferEsdtProxy)
            .set_bridge_proxy_contract_address(OptionalValue::Some(
                BRIDGE_PROXY_ADDRESS.to_address(),
            ))
            .run();

        self.world
            .tx()
            .from(MULTISIG_ADDRESS)
            .to(MULTI_TRANSFER_ADDRESS)
            .typed(multi_transfer_proxy::MultiTransferEsdtProxy)
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
fn ethereum_to_multiversx_tx_batch_ok_test() {
    let mut state = MultiTransferTestState::new();
    let token_amount = BigUint::from(76_000_000_000u64);

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

    let eth_tx1 = EthTransaction {
        from: EthAddress {
            raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
        },
        to: ManagedAddress::from(USER1_ADDRESS.eval_to_array()),
        token_id: TokenIdentifier::from(WEGLD_TOKEN_ID),
        amount: token_amount.clone(),
        tx_nonce: 1u64,
        call_data: call_data.clone(),
    };

    let eth_tx2 = EthTransaction {
        from: EthAddress {
            raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
        },
        to: ManagedAddress::from(USER1_ADDRESS.eval_to_array()),
        token_id: TokenIdentifier::from(ETH_TOKEN_ID),
        amount: token_amount.clone(),
        tx_nonce: 2u64,
        call_data,
    };

    let mut transfers: ManagedVec<StaticApi, EthTransaction<StaticApi>> = ManagedVec::new();
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
}

#[test]
fn ethereum_to_multiversx_tx_batch_rejected_test() {
    let mut state = MultiTransferTestState::new();
    // let token_amount = BigUint::from(76_000_000_000u64);
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

    let eth_tx1 = EthTransaction {
        from: EthAddress {
            raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
        },
        to: ManagedAddress::from(BRIDGE_PROXY_ADDRESS.eval_to_array()),
        token_id: TokenIdentifier::from(WEGLD_TOKEN_ID),
        amount: over_the_limit_token_amount.clone(),
        tx_nonce: 1u64,
        call_data: call_data.clone(),
    };

    let eth_tx2 = EthTransaction {
        from: EthAddress {
            raw_addr: ManagedByteArray::new_from_bytes(b"01020304050607080910"),
        },
        to: ManagedAddress::from(BRIDGE_PROXY_ADDRESS.eval_to_array()),
        token_id: TokenIdentifier::from(ETH_TOKEN_ID),
        amount: over_the_limit_token_amount.clone(),
        tx_nonce: 2u64,
        call_data,
    };

    let mut transfers: ManagedVec<StaticApi, EthTransaction<StaticApi>> = ManagedVec::new();
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

    // TODO: check this
    assert!(refund_tx.is_none());

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(MULTISIG_ADDRESS)
        .typed(multisig_proxy::MultisigProxy)
        .move_refund_batch_to_safe_from_child_contract()
        .run();

    // state
    //     .world
    //     .check_account(ESDT_SAFE_ADDRESS)
    //     .esdt_balance(WEGLD_TOKEN_ID, over_the_limit_token_amount.clone())
    //     .esdt_balance(ETH_TOKEN_ID, over_the_limit_token_amount);
}
