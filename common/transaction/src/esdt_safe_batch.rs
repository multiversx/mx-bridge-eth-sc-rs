elrond_wasm::derive_imports!();

use elrond_wasm::{
    api::ManagedTypeApi,
    types::{ManagedDefault, ManagedMultiResultVec, ManagedVec, MultiResult2},
};

use crate::{Transaction, TxAsMultiResult};

#[derive(TypeAbi, TopEncode, TopDecode)]
pub struct EsdtSafeTxBatch<M: ManagedTypeApi> {
    pub batch_id: u64,
    pub transactions: ManagedVec<M, Transaction<M>>,
}

impl<M: ManagedTypeApi> ManagedDefault<M> for EsdtSafeTxBatch<M> {
    fn managed_default(api: M) -> Self {
        Self {
            batch_id: 0,
            transactions: ManagedVec::new(api),
        }
    }
}

pub type EsdtSafeTxBatchSplitInFields<M> =
    MultiResult2<u64, ManagedMultiResultVec<M, TxAsMultiResult<M>>>;
