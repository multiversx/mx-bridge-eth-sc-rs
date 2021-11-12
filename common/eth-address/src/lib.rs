#![no_std]

elrond_wasm::derive_imports!();
use elrond_wasm::{
    api::{Handle, ManagedTypeApi},
    types::{ManagedByteArray, ManagedType, ManagedVecItem},
};

pub const ETH_ADDRESS_LEN: usize = 20;

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, Clone)]
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

impl<M: ManagedTypeApi> ManagedVecItem<M> for EthAddress<M> {
    const PAYLOAD_SIZE: usize = ETH_ADDRESS_LEN;
    const SKIPS_RESERIALIZATION: bool = false;

    fn from_byte_reader<Reader: FnMut(&mut [u8])>(api: M, reader: Reader) -> Self {
        let handle = Handle::from_byte_reader(api.clone(), reader);
        Self {
            raw_addr: ManagedByteArray::from_raw_handle(api, handle),
        }
    }

    fn to_byte_writer<R, Writer: FnMut(&[u8]) -> R>(&self, writer: Writer) -> R {
        <Handle as ManagedVecItem<M>>::to_byte_writer(&self.raw_addr.get_raw_handle(), writer)
    }
}
