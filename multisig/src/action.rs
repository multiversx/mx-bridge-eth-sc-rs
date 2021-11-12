use elrond_wasm::api::BigUintApi;
use elrond_wasm::types::{ManagedAddress, TokenIdentifier, Vec};
use transaction::TransactionStatus;

elrond_wasm::derive_imports!();

// Actions with _ in front are not used
// Keeping the actions even if they're not used, for backwards compatibility of action type ID
#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub enum Action<M: ManagedTypeApi> {
    Nothing,
    _AddBoardMember(ManagedAddress),
    _AddProposer(ManagedAddress),
    _RemoveUser(ManagedAddress),
    _SlashUser(ManagedAddress),
    _ChangeQuorum(usize),
    SetCurrentTransactionBatchStatus {
        esdt_safe_batch_id: u64,
        tx_batch_status: Vec<TransactionStatus>,
    },
    BatchTransferEsdtToken {
        batch_id: u64,
        transfers: Vec<(ManagedAddress, TokenIdentifier, BigUint)>,
    },
}

impl<M: ManagedTypeApi> Action<BigUint> {
    /// Only pending actions are kept in storage,
    /// both executed and discarded actions are removed (converted to `Nothing`).
    /// So this is equivalent to `action != Action::Nothing`.
    pub fn is_pending(&self) -> bool {
        !matches!(*self, Action::Nothing)
    }
}

#[cfg(test)]
mod test {
    use super::Action;
    use elrond_wasm_debug::api::RustBigUint;

    #[test]
    fn test_is_pending() {
        assert!(!Action::<RustBigUint>::Nothing.is_pending());
    }
}
