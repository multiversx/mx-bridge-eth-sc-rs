#![no_std]

use elrond_wasm::api::BigUintApi;
use elrond_wasm::types::{Address, MultiResult6, TokenIdentifier};
use eth_address::EthAddress;

pub mod esdt_safe_batch;

elrond_wasm::derive_imports!();

pub type TxNonce = u64;
pub type BlockNonce = u64;
pub type TxAsMultiResult<BigUint> =
    MultiResult6<BlockNonce, TxNonce, Address, EthAddress, TokenIdentifier, BigUint>;

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi)]
pub struct Transaction<BigUint: BigUintApi> {
    pub block_nonce: BlockNonce,
    pub nonce: TxNonce,
    pub from: Address,
    pub to: EthAddress,
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

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, PartialEq, Clone, Copy)]
pub enum TransactionStatus {
    None,
    Pending,
    InProgress,
    Executed,
    Rejected,
}
