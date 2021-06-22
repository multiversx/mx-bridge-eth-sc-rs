#![no_std]

elrond_wasm::derive_imports!();
use elrond_wasm::types::Box;

pub const ETH_ADDRESS_LEN: usize = 20;

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, Clone)]
pub struct EthAddress(Box<[u8; ETH_ADDRESS_LEN]>);

impl EthAddress {
    pub fn as_slice(&self) -> &[u8] {
        &(*self.0)[..]
    }

    pub fn is_zero(&self) -> bool {
        self.0.eq(&Self::zero().0)
    }
}

impl EthAddress {
    pub fn zero() -> Self {
        EthAddress(Box::from([0u8; ETH_ADDRESS_LEN]))
    }
}

impl<'a> From<&'a [u8]> for EthAddress {
    fn from(slice: &'a [u8]) -> Self {
        let mut zero = Self::zero();
        if slice.len() == ETH_ADDRESS_LEN {
            (*zero.0).copy_from_slice(slice)
        }

        zero
    }
}

impl From<[u8; ETH_ADDRESS_LEN]> for EthAddress {
    fn from(array: [u8; ETH_ADDRESS_LEN]) -> Self {
        Self::from(&array[..])
    }
}
