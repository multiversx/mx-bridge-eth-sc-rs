multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait EventsModule {
    #[event("wrap_tokens")]
    fn wrap_tokens_event(&self, #[indexed] token_id: TokenIdentifier, #[indexed] amount: BigUint);

    #[event("unwrap_tokens")]
    fn unwrap_tokens_event(&self, #[indexed] token_id: TokenIdentifier, #[indexed] amount: BigUint);
}
