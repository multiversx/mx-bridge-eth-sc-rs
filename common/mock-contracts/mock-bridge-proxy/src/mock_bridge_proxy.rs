#![no_std]

#[allow(unused_imports)]
use multiversx_sc::imports::*;

/// An empty contract. To be used as a template when starting a new contract from scratch.
#[multiversx_sc::contract]
pub trait MockBridgeProxy {
    #[init]
    fn init(&self, _opt_multi_transfer_address: OptionalValue<ManagedAddress>) {}

    #[upgrade]
    fn upgrade(&self) {}
}
