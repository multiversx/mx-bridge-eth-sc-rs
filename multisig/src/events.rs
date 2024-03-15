use eth_address::EthAddress;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait EventsModule {
    #[event("add_mapping")]
    fn add_mapping_event(
        &self,
        #[indexed] erc20_address: EthAddress<Self::Api>,
        #[indexed] token_id: TokenIdentifier,
    );

    #[event("clear_mapping")]
    fn clear_mapping_event(
        &self,
        #[indexed] erc20_address: EthAddress<Self::Api>,
        #[indexed] token_id: TokenIdentifier,
    );

    #[event("move_refund_batch_to_safe")]
    fn move_refund_batch_to_safe_event(&self);

    #[event("pause_esdt_safe")]
    fn pause_esdt_safe_event(&self);

    #[event("unpause_esdt_safe")]
    fn unpause_esdt_safe_event(&self);
}
