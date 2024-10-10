#![no_std]

#[allow(unused_imports)]
use multiversx_sc::imports::*;

/// An empty contract. To be used as a template when starting a new contract from scratch.
#[multiversx_sc::contract]
pub trait MockBridgedTokensWrapper {
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}

    #[payable("*")]
    #[endpoint(unwrapTokenCreateTransaction)]
    fn unwrap_token_create_transaction(
        &self,
        requested_token: TokenIdentifier,
        to: EthAddress<Self::Api>,
        opt_refunding_address: OptionalValue<ManagedAddress>,
    ) {
    }
}
