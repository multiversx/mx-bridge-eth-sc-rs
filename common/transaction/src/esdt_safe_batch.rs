elrond_wasm::derive_imports!();

use elrond_wasm::{
    api::ManagedTypeApi,
    types::{ManagedMultiResultVec, ManagedVec, MultiResult2},
};

use crate::{Transaction, TxAsMultiResult};

#[derive(TypeAbi, TopEncode, TopDecode)]
pub struct EsdtSafeTxBatch<M: ManagedTypeApi> {
    pub batch_id: u64,
    pub transactions: ManagedVec<M, Transaction<M>>,
}

pub type TxBatchSplitInFields<M> =
    MultiResult2<u64, ManagedMultiResultVec<M, TxAsMultiResult<M>>>;
