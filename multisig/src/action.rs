use elrond_wasm::api::ManagedTypeApi;
use elrond_wasm::types::{ManagedAddress, ManagedVec};
use transaction::{SingleTransferTuple, TransactionStatus};

elrond_wasm::derive_imports!();

// Actions with _ in front are not used
// Keeping the actions even if they're not used, for backwards compatibility of action type ID
#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub enum Action<M: ManagedTypeApi> {
    Nothing,
    _AddBoardMember(ManagedAddress<M>),
    _AddProposer(ManagedAddress<M>),
    _RemoveUser(ManagedAddress<M>),
    _SlashUser(ManagedAddress<M>),
    _ChangeQuorum(usize),
    SetCurrentTransactionBatchStatus {
        esdt_safe_batch_id: u64,
        tx_batch_status: ManagedVec<M, TransactionStatus>,
    },
    BatchTransferEsdtToken {
        batch_id: u64,
        transfers: ManagedVec<M, SingleTransferTuple<M>>,
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
