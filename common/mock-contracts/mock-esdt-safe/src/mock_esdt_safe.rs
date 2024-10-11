#![no_std]

#[allow(unused_imports)]
use multiversx_sc::imports::*;

/// An empty contract. To be used as a template when starting a new contract from scratch.
#[multiversx_sc::contract]
pub trait MockEsdtSafe {
    #[init]
    fn init(
        &self,
        fee_estimator_contract_address: ManagedAddress,
        multi_transfer_contract_address: ManagedAddress,
        eth_tx_gas_limit: BigUint,
    ) {
    }

    #[upgrade]
    fn upgrade(&self) {}
}
