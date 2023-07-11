use bridge_proxy::*;
use multiversx_sc::types::Address;
use multiversx_sc_scenario::{managed_address, rust_biguint, testing_framework::*, DebugApi};

const BRIDGE_PROXY_PATH: &str = "output/bridge-proxy.wasm";

struct ContractSetup<BridgeProxyObjBuilder>
where
    BridgeProxyObjBuilder: 'static + Copy + Fn() -> bridge_proxy::ContractObj<DebugApi>,
{
    pub blockchain_wrapper: BlockchainStateWrapper,
    pub owner_address: Address,
    pub bridge_proxy_wrapper:
        ContractObjWrapper<bridge_proxy::ContractObj<DebugApi>, BridgeProxyObjBuilder>,
}

fn setup_contract<BridgeProxyObjBuilder>(
    bridge_proxy_builder: BridgeProxyObjBuilder,
) -> ContractSetup<BridgeProxyObjBuilder>
where
    BridgeProxyObjBuilder: 'static + Copy + Fn() -> bridge_proxy::ContractObj<DebugApi>,
{
    let rust_zero = rust_biguint!(0u64);
    let mut blockchain_wrapper = BlockchainStateWrapper::new();
    let owner_address = blockchain_wrapper.create_user_account(&rust_zero);
    let bridge_proxy_wrapper = blockchain_wrapper.create_sc_account(
        &rust_zero,
        Some(&owner_address),
        bridge_proxy_builder,
        BRIDGE_PROXY_PATH,
    );

    blockchain_wrapper
        .execute_tx(&owner_address, &bridge_proxy_wrapper, &rust_zero, |sc| {
            sc.init(managed_address!(&Address::zero()));
        })
        .assert_ok();

    blockchain_wrapper.add_mandos_set_account(bridge_proxy_wrapper.address_ref());

    ContractSetup {
        blockchain_wrapper,
        owner_address,
        bridge_proxy_wrapper,
    }
}

#[test]
fn deploy_test() {
    let mut setup = setup_contract(bridge_proxy::contract_obj);

    // simulate deploy
    setup
        .blockchain_wrapper
        .execute_tx(
            &setup.owner_address,
            &setup.bridge_proxy_wrapper,
            &rust_biguint!(0u64),
            |sc| {
                sc.init(managed_address!(&Address::zero()));
            },
        )
        .assert_ok();
}
