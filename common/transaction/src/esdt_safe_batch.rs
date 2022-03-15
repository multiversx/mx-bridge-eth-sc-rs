elrond_wasm::derive_imports!();

use elrond_wasm::{
    api::ManagedTypeApi,
    elrond_codec::multi_types::MultiValue2,
    types::{ManagedVec, MultiValueEncoded},
};

use crate::{Transaction, TxAsMultiValue};

#[derive(TypeAbi, TopEncode, TopDecode)]
pub struct EsdtSafeTxBatch<M: ManagedTypeApi> {
    pub batch_id: u64,
    pub transactions: ManagedVec<M, Transaction<M>>,
}

pub type TxBatchSplitInFields<M> = MultiValue2<u64, MultiValueEncoded<M, TxAsMultiValue<M>>>;
