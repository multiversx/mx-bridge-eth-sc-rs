#![no_std]

#[allow(unused_imports)]
use multiversx_sc::imports::*;

/// An empty contract. To be used as a template when starting a new contract from scratch.
#[multiversx_sc::contract]
pub trait MockMultiTransferEsdt {
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}

    #[only_owner]
    #[endpoint(addUnprocessedRefundTxToBatch)]
    fn add_unprocessed_refund_tx_to_batch(&self, _tx_id: u64) {}
}
