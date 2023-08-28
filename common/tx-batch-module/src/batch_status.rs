multiversx_sc::derive_imports!();

use multiversx_sc::{api::ManagedTypeApi, types::ManagedVec};
use transaction::{BlockNonce, TxNonce};

#[derive(TopEncode, TopDecode, TypeAbi)]
pub enum BatchStatus<M: ManagedTypeApi> {
    AlreadyProcessed,
    Empty,
    PartiallyFull {
        end_block_nonce: BlockNonce,
        tx_ids: ManagedVec<M, TxNonce>,
    },
    Full,
    WaitingForSignatures,
}
