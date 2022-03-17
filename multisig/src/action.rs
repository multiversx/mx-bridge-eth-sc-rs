use elrond_wasm::api::ManagedTypeApi;
use elrond_wasm::types::ManagedVec;
use transaction::transaction_status::TransactionStatus;
use transaction::EthTransaction;

elrond_wasm::derive_imports!();

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub enum Action<M: ManagedTypeApi> {
    Nothing,
    SetCurrentTransactionBatchStatus {
        esdt_safe_batch_id: u64,
        tx_batch_status: ManagedVec<M, TransactionStatus>,
    },
    BatchTransferEsdtToken {
        eth_batch_id: u64,
        transfers: ManagedVec<M, EthTransaction<M>>,
    },
}

impl<M: ManagedTypeApi> Action<M> {
    /// Only pending actions are kept in storage,
    /// both executed and discarded actions are removed (converted to `Nothing`).
    /// So this is equivalent to `action != Action::Nothing`.
    pub fn is_pending(&self) -> bool {
        !matches!(*self, Action::Nothing)
    }
}
