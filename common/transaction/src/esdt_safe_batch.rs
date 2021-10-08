elrond_wasm::derive_imports!();

use elrond_wasm::{
    api::BigUintApi,
    types::{MultiResult2, MultiResultVec},
    Vec,
};

use crate::{Transaction, TxAsMultiResult};

#[derive(TypeAbi, TopEncode, TopDecode)]
pub struct EsdtSafeTxBatch<BigUint: BigUintApi> {
    pub batch_id: u64,
    pub transactions: Vec<Transaction<BigUint>>,
}

impl<BigUint: BigUintApi> Default for EsdtSafeTxBatch<BigUint> {
    fn default() -> Self {
        EsdtSafeTxBatch {
            batch_id: 0,
            transactions: Vec::new(),
        }
    }
}

pub type EsdtSafeTxBatchSplitInFields<BigUint> =
    MultiResult2<u64, MultiResultVec<TxAsMultiResult<BigUint>>>;
