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

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi)]
pub enum TransactionType {
    Ethereum, // 21000
    Erc20,
    Erc721,
    Erc1155,
}

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct TransactionGasLimits<BigUint: BigUintApi> {
    pub ethereum: BigUint,
    pub erc20: BigUint,
    pub erc721: BigUint,
    pub erc1155: BigUint,
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi)]
pub enum Priority {
    Fast,
    Average,
    Low,
}

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct PriorityGasCosts<BigUint: BigUintApi> {
    pub fast: BigUint,
    pub average: BigUint,
    pub low: BigUint,
}
