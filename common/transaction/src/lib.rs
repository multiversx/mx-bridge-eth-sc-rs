#![no_std]

use elrond_wasm::api::BigUintApi;
use elrond_wasm::types::{Address, MultiResult5, TokenIdentifier};

elrond_wasm::derive_imports!();

pub type TxNonce = usize;
pub type BlockNonce = u64;
pub type TxAsMultiResult<BigUint> =
    MultiResult6<BlockNonce, TxNonce, Address, Address, TokenIdentifier, BigUint>;

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct Transaction<BigUint: BigUintApi> {
    pub block_nonce: BlockNonce,
    pub nonce: TxNonce,
    pub from: Address,
    pub to: Address,
    pub token_identifier: TokenIdentifier,
    pub amount: BigUint,
}

impl<BigUint: BigUintApi> From<TxAsMultiResult<BigUint>> for Transaction<BigUint> {
    fn from(tx_as_multiresult: TxAsMultiResult<BigUint>) -> Self {
        let (block_nonce, nonce, from, to, token_identifier, amount) =
            tx_as_multiresult.into_tuple();

        Transaction {
            block_nonce,
            nonce,
            from,
            to,
            token_identifier,
            amount,
        }
    }
}

impl<BigUint: BigUintApi> Transaction<BigUint> {
    pub fn into_multiresult(self) -> TxAsMultiResult<BigUint> {
        (
            self.block_nonce,
            self.nonce,
            self.from,
            self.to,
            self.token_identifier,
            self.amount,
        )
            .into()
    }
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
