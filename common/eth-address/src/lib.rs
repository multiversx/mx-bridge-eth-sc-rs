#![no_std]
#![feature(generic_associated_types)]

elrond_wasm::derive_imports!();
use elrond_wasm::{
    api::ManagedTypeApi,
    types::{ManagedBuffer, ManagedByteArray},
};

pub const ETH_ADDRESS_LEN: usize = 20;

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, Clone, ManagedVecItem)]
pub struct EthAddress<M: ManagedTypeApi> {
    pub raw_addr: ManagedByteArray<M, ETH_ADDRESS_LEN>,
}

impl<M: ManagedTypeApi> EthAddress<M> {
    pub fn zero() -> Self {
        Self {
            raw_addr: ManagedByteArray::new_from_bytes(&[0u8; ETH_ADDRESS_LEN]),
        }
    }

    pub fn as_managed_buffer(&self) -> &ManagedBuffer<M> {
        self.raw_addr.as_managed_buffer()
    }
}
