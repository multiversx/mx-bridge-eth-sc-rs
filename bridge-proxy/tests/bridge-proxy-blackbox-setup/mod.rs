
use bridge_proxy::bridge_proxy::ConfigModule;

use multiversx_sc::{
    api::ManagedTypeApi,
    codec::multi_types::OptionalValue,
    types::{Address, BigUint, BoxedBytes, CodeMetadata, ManagedBuffer, ManagedVec},
};
use multiversx_sc_scenario::{managed_address, rust_biguint, testing_framework::*, DebugApi};


const BRIDGE_PROXY_WASM_PATH: &str = "bridge-proxy/output/bridge-proxy.wasm";
const BRIDGE_TOKEN_ID: &[u8] = b"BRIDGE-123456";


pub struct BridgeProxySetup<BridgeProxyObjBuilder>
where
    BridgeProxyObjBuilder: 'static + Copy + Fn() -> bridge_proxy::ContractObj<DebugApi>,
{
    pub b_mock: BlockchainStateWrapper,
    pub owner_address: Address,
    pub bp_wrapper: ContractObjWrapper<bridge_proxy::ContractObj<DebugApi>, BridgeProxyObjBuilder>,
}

impl<BridgeProxyObjBuilder> BridgeProxySetup<BridgeProxyObjBuilder>
where
    BridgeProxyObjBuilder: 'static + Copy + Fn() -> bridge_proxy::ContractObj<DebugApi>,
{
    pub fn new(bp_builder: BridgeProxyObjBuilder) -> Self {
        let rust_zero = rust_biguint!(0u64);
        let mut b_mock = BlockchainStateWrapper::new();
        let owner_address = b_mock.create_user_account(&rust_zero);

        let bp_wrapper = b_mock.create_sc_account(
            &rust_zero,
            Some(&owner_address),
            bp_builder,
            BRIDGE_PROXY_PATH,
        );
        b_mock
            .execute_tx(&owner_address, &ms_wrapper, &rust_zero, |sc| {
                let mut board_members = ManagedVec::new();
                board_members.push(managed_address!(&board_member_address));

                sc.init(QUORUM_SIZE, board_members.into());
                sc.change_user_role(0, managed_address!(&proposer_address), UserRole::Proposer);
            })
            .assert_ok();

        Self {
            b_mock,
            owner_address,
            bp_wrapper,
        }
    }
}