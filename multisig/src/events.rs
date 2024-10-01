use eth_address::EthAddress;

#[multiversx_sc::module]
pub trait EventsModule {
    #[event("addMapping")]
    fn add_mapping_event(
        &self,
        #[indexed] erc20_address: EthAddress<Self::Api>,
        #[indexed] token_id: TokenIdentifier,
    );

    #[event("clearMapping")]
    fn clear_mapping_event(
        &self,
        #[indexed] erc20_address: EthAddress<Self::Api>,
        #[indexed] token_id: TokenIdentifier,
    );

    #[event("moveRefundBatchToSafeEvent")]
    fn move_refund_batch_to_safe_event(&self);

    #[event("addUnprocessedRefundTxToBatchEvent")]
    fn add_unprocessed_refund_tx_to_batch_event(&self, #[indexed] tx_id: u64);

    #[event("pauseEsdtEvent")]
    fn pause_esdt_safe_event(&self);

    #[event("unpauseEsdtEvent")]
    fn unpause_esdt_safe_event(&self);
}
