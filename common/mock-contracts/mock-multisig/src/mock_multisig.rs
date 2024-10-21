#![no_std]

#[allow(unused_imports)]
use multiversx_sc::imports::*;

/// An empty contract. To be used as a template when starting a new contract from scratch.
#[multiversx_sc::contract]
pub trait MockMultisig {
    #[init]
    fn init(
        &self,
        _esdt_safe_sc_address: ManagedAddress,
        _multi_transfer_sc_address: ManagedAddress,
        _proxy_sc_address: ManagedAddress,
        _required_stake: BigUint,
        _slash_amount: BigUint,
        _quorum: usize,
        _board: MultiValueEncoded<ManagedAddress>,
    ) {
    }

    #[upgrade]
    fn upgrade(&self) {}
}
