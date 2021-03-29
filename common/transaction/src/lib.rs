#![no_std]

use elrond_wasm::api::BigUintApi;
use elrond_wasm::types::{Address, TokenIdentifier};

elrond_wasm::derive_imports!();

pub type Nonce = usize;

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct Transaction<BigUint: BigUintApi> {
    pub from: Address,
    pub to: Address,
    pub token_identifier: TokenIdentifier,
    pub amount: BigUint,
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, PartialEq)]
pub enum TransactionStatus {
    None,
    Pending,
    InProgress,
    Executed,
    Rejected,
}
