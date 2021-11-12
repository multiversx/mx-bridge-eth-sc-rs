#![no_std]

use elrond_wasm::{
    api::ManagedTypeApi,
    types::{BigUint, ManagedAddress, ManagedVecItem, MultiResult6, TokenIdentifier},
    EndpointResult,
};
use eth_address::EthAddress;

pub mod esdt_safe_batch;

elrond_wasm::derive_imports!();

// revert protection
pub const MIN_BLOCKS_FOR_FINALITY: u64 = 2;

pub type TxNonce = u64;
pub type BlockNonce = u64;
pub type TxAsMultiResult<M> = MultiResult6<
    BlockNonce,
    TxNonce,
    ManagedAddress<M>,
    EthAddress<M>,
    TokenIdentifier<M>,
    BigUint<M>,
>;

#[derive(TopEncode, TopDecode, TypeAbi, ManagedVecItem)]
pub struct SingleTransferTuple<M: ManagedTypeApi> {
    pub address: ManagedAddress<M>,
    pub token_id: TokenIdentifier<M>,
    pub amount: BigUint<M>,
}

#[derive(NestedEncode, NestedDecode, TypeAbi, ManagedVecItem)]
pub struct Transaction<M: ManagedTypeApi> {
    pub block_nonce: BlockNonce,
    pub nonce: TxNonce,
    pub from: ManagedAddress<M>,
    pub to: EthAddress<M>,
    pub token_identifier: TokenIdentifier<M>,
    pub amount: BigUint<M>,
}

impl<M: ManagedTypeApi> EndpointResult for Transaction<M> {
    type DecodeAs = Transaction<M>;

    fn finish<FA>(&self, api: FA)
    where
        FA: ManagedTypeApi + elrond_wasm::api::EndpointFinishApi + Clone + 'static,
    {
        self.block_nonce.finish(api.clone());
        self.nonce.finish(api.clone());
        self.from.finish(api.clone());
        self.to.finish(api.clone());
        self.token_identifier.finish(api.clone());
        self.amount.finish(api);
    }
}

impl<M: ManagedTypeApi> From<TxAsMultiResult<M>> for Transaction<M> {
    fn from(tx_as_multiresult: TxAsMultiResult<M>) -> Self {
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

impl<M: ManagedTypeApi> Transaction<M> {
    pub fn into_multiresult(self) -> TxAsMultiResult<M> {
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

impl From<u8> for TransactionStatus {
    fn from(raw_value: u8) -> Self {
        match raw_value {
            1u8 => Self::Pending,
            2u8 => Self::InProgress,
            3u8 => Self::Executed,
            4u8 => Self::Rejected,
            _ => Self::None,
        }
    }
}

impl TransactionStatus {
    fn as_u8(&self) -> u8 {
        match *self {
            Self::None => 0u8,
            Self::Pending => 1u8,
            Self::InProgress => 2u8,
            Self::Executed => 3u8,
            Self::Rejected => 4u8,
        }
    }
}

impl<M: ManagedTypeApi> ManagedVecItem<M> for TransactionStatus {
    const PAYLOAD_SIZE: usize = 1;
    const SKIPS_RESERIALIZATION: bool = true;

    fn from_byte_reader<Reader: FnMut(&mut [u8])>(api: M, reader: Reader) -> Self {
        u8::from_byte_reader(api, reader).into()
    }

    fn to_byte_writer<R, Writer: FnMut(&[u8]) -> R>(&self, writer: Writer) -> R {
        <u8 as ManagedVecItem<M>>::to_byte_writer(&self.as_u8(), writer)
    }
}
