#![no_std]

#[allow(unused_imports)]
use multiversx_sc::imports::*;

/// An empty contract. To be used as a template when starting a new contract from scratch.
#[multiversx_sc::contract]
pub trait MockPriceAggregator {
    #[init]
    fn init(
        &self,
        _staking_token: EgldOrEsdtTokenIdentifier,
        _staking_amount: BigUint,
        _slash_amount: BigUint,
        _slash_quorum: usize,
        _submission_count: usize,
        _oracles: MultiValueEncoded<ManagedAddress>,
    ) {
    }

    #[upgrade]
    fn upgrade(&self) {}

    #[view(latestPriceFeedOptional)]
    fn latest_price_feed_optional(&self, _from: ManagedBuffer, _to: ManagedBuffer) {}
}
