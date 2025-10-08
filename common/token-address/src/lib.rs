#![no_std]

use multiversx_sc::derive_imports::*;
use multiversx_sc::{api::ManagedTypeApi, types::ManagedBuffer};

/// Wrapper over a buffer
#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, Clone, ManagedVecItem, PartialEq)]
pub struct TokenAddress<M: ManagedTypeApi> {
    pub raw_addr: ManagedBuffer<M>,
}

impl<M: ManagedTypeApi> TokenAddress<M> {
    pub fn zero() -> Self {
        Self {
            raw_addr: ManagedBuffer::new_from_bytes(&[0u8]),
        }
    }

    pub fn as_managed_buffer(&self) -> &ManagedBuffer<M> {
        &self.raw_addr
    }
}
