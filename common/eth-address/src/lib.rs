#![no_std]

use multiversx_sc::derive_imports::*;
use multiversx_sc::{
    api::ManagedTypeApi,
    types::{ManagedBuffer, ManagedByteArray},
};

pub const ETH_ADDRESS_LEN: usize = 20;

/// Wrapper over a 20-byte array
#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, Clone, ManagedVecItem, PartialEq)]
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
