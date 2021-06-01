use elrond_wasm::api::BigUintApi;
use elrond_wasm::types::{Address, TokenIdentifier, Vec};
use transaction::TransactionStatus;

elrond_wasm::derive_imports!();

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi)]
pub enum Action<BigUint: BigUintApi> {
    Nothing,
    AddBoardMember(Address),
    AddProposer(Address),
    RemoveUser(Address),
    SlashUser(Address),
    ChangeQuorum(usize),
    SetCurrentTransactionBatchStatus {
        relayer_reward_address: Address,
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

    pub fn is_slash_user(&self) -> bool {
        matches!(*self, Action::SlashUser(_))
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
    use elrond_wasm::types::Address;
    use elrond_wasm_debug::api::RustBigUint;

    #[test]
    fn test_is_pending() {
        assert!(!Action::<RustBigUint>::Nothing.is_pending());
        assert!(Action::<RustBigUint>::ChangeQuorum(5).is_pending());
    }

    #[test]
    fn test_is_slash_user() {
        assert!(!Action::<RustBigUint>::Nothing.is_slash_user());
        assert!(Action::<RustBigUint>::SlashUser(Address::zero()).is_slash_user());
    }
}
