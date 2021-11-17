#![no_std]

elrond_wasm::derive_imports!();
use elrond_wasm::{api::ManagedTypeApi, types::ManagedByteArray};

pub const ETH_ADDRESS_LEN: usize = 20;

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem, Clone)]
pub struct EthAddress<M: ManagedTypeApi> {
    pub raw_addr: ManagedByteArray<M, ETH_ADDRESS_LEN>,
}

impl<M: ManagedTypeApi> EthAddress<M> {
    pub fn zero(api: M) -> Self {
        Self {
            raw_addr: ManagedByteArray::new_from_bytes(api, &[0u8; ETH_ADDRESS_LEN]),
        }
    }
}
