#![no_std]
use multiversx_sc::imports::*;
use multiversx_sc::storage::StorageKey;

pub const BRIDGE_PROXY_CONTRACT_ADDRESS_STORAGE_KEY: &[u8] = b"proxyAddress";
pub const BRIDGED_TOKENS_WRAPPER_ADDRESS_STORAGE_KEY: &[u8] = b"bridgedTokensWrapperAddress";
pub const ESDT_SAFE_CONTRACT_ADDRESS_STORAGE_KEY: &[u8] = b"esdtSafeAddress";
pub const MULTI_TRNASFER_CONTRACT_ADDRESS_STORAGE_KEY: &[u8] = b"multiTransferEsdtAddress";
pub const FEE_ESTIMATOR_CONTRACT_ADDRESS_STORAGE_KEY: &[u8] = b"feeEstimatorAddress";

#[multiversx_sc::module]
pub trait CommonStorageModule {
    fn get_bridged_tokens_wrapper_address(
        &self,
        owner_address: ManagedAddress,
    ) -> SingleValueMapper<ManagedAddress, ManagedAddress> {
        SingleValueMapper::<_, _, ManagedAddress>::new_from_address(
            owner_address,
            StorageKey::new(BRIDGED_TOKENS_WRAPPER_ADDRESS_STORAGE_KEY),
        )
    }

    fn get_bridge_proxy_address(
        &self,
        owner_address: ManagedAddress,
    ) -> SingleValueMapper<ManagedAddress, ManagedAddress> {
        SingleValueMapper::<_, _, ManagedAddress>::new_from_address(
            owner_address,
            StorageKey::new(BRIDGE_PROXY_CONTRACT_ADDRESS_STORAGE_KEY),
        )
    }

    fn get_esdt_safe_address(
        &self,
        owner_address: ManagedAddress,
    ) -> SingleValueMapper<ManagedAddress, ManagedAddress> {
        SingleValueMapper::<_, _, ManagedAddress>::new_from_address(
            owner_address,
            StorageKey::new(ESDT_SAFE_CONTRACT_ADDRESS_STORAGE_KEY),
        )
    }

    fn get_multi_transfer_address(
        &self,
        owner_address: ManagedAddress,
    ) -> SingleValueMapper<ManagedAddress, ManagedAddress> {
        SingleValueMapper::<_, _, ManagedAddress>::new_from_address(
            owner_address,
            StorageKey::new(MULTI_TRNASFER_CONTRACT_ADDRESS_STORAGE_KEY),
        )
    }

    fn get_fee_estimator_address(
        &self,
        owner_address: ManagedAddress,
    ) -> SingleValueMapper<ManagedAddress, ManagedAddress> {
        SingleValueMapper::<_, _, ManagedAddress>::new_from_address(
            owner_address,
            StorageKey::new(FEE_ESTIMATOR_CONTRACT_ADDRESS_STORAGE_KEY),
        )
    }
}
