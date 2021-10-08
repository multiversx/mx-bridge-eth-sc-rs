use elrond_wasm::api::BigUintApi;
use elrond_wasm::types::{Address, TokenIdentifier, Vec};
use transaction::TransactionStatus;

elrond_wasm::derive_imports!();

// Actions with _ in front are not used
// Keeping the actions even if they're not used, for backwards compatibility of action type ID
#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub enum Action<BigUint: BigUintApi> {
    Nothing,
    _AddBoardMember(Address),
    _AddProposer(Address),
    _RemoveUser(Address),
    _SlashUser(Address),
    _ChangeQuorum(usize),
    SetCurrentTransactionBatchStatus {
        esdt_safe_batch_id: u64,
        tx_batch_status: Vec<TransactionStatus>,
    },
    BatchTransferEsdtToken {
        batch_id: u64,
        transfers: Vec<(Address, TokenIdentifier, BigUint)>,
    },
}

impl<BigUint: BigUintApi> Action<BigUint> {
    /// Only pending actions are kept in storage,
    /// both executed and discarded actions are removed (converted to `Nothing`).
    /// So this is equivalent to `action != Action::Nothing`.
    pub fn is_pending(&self) -> bool {
        !matches!(*self, Action::Nothing)
    }
}

/// Not used internally, just to retrieve results via endpoint.
#[derive(TopEncode, TypeAbi)]
pub struct ActionFullInfo<BigUint: BigUintApi> {
    pub action_id: usize,
    pub action_data: Action<BigUint>,
    pub signers: Vec<Address>,
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
