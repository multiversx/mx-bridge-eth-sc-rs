#![no_std]
use multiversx_sc::imports::*;
use multiversx_sc::storage::StorageKey;

pub const BRIDGE_PROXY_CONTRACT_ADDRESS_STORAGE_KEY: &[u8] = b"proxyAddress";
pub const BRIDGED_TOKENS_WRAPPER_ADDRESS_STORAGE_KEY: &[u8] = b"bridgedTokensWrapperAddress";
pub const ESDT_SAFE_CONTRACT_ADDRESS_STORAGE_KEY: &[u8] = b"esdtSafeAddress";
pub const MULTI_TRANSFER_CONTRACT_ADDRESS_STORAGE_KEY: &[u8] = b"multiTransferEsdtAddress";
pub const FEE_ESTIMATOR_CONTRACT_ADDRESS_STORAGE_KEY: &[u8] = b"feeEstimatorAddress";

#[multiversx_sc::module]
pub trait CommonStorageModule {
    fn single_value_mapper(&self, key: &[u8]) -> SingleValueMapper<ManagedAddress, ManagedAddress> {
        SingleValueMapper::<_, _, ManagedAddress>::new_from_address(
            self.blockchain().get_owner_address(),
            StorageKey::new(key),
        )
    }

    fn get_bridged_tokens_wrapper_address(&self) -> ManagedAddress {
        self.single_value_mapper(BRIDGED_TOKENS_WRAPPER_ADDRESS_STORAGE_KEY)
            .get()
    }

    fn get_bridge_proxy_address(&self) -> ManagedAddress {
        self.single_value_mapper(BRIDGE_PROXY_CONTRACT_ADDRESS_STORAGE_KEY)
            .get()
    }

    fn get_esdt_safe_address(&self) -> ManagedAddress {
        self.single_value_mapper(ESDT_SAFE_CONTRACT_ADDRESS_STORAGE_KEY)
            .get()
    }

    fn get_multi_transfer_address(&self) -> ManagedAddress {
        self.single_value_mapper(MULTI_TRANSFER_CONTRACT_ADDRESS_STORAGE_KEY)
            .get()
    }

    fn get_fee_estimator_address(&self) -> ManagedAddress {
        self.single_value_mapper(FEE_ESTIMATOR_CONTRACT_ADDRESS_STORAGE_KEY)
            .get()
    }
}
