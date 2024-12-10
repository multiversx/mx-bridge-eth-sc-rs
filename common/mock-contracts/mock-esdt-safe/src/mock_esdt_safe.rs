#![no_std]
multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use eth_address::EthAddress;

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, Clone, ManagedVecItem, PartialEq)]
pub struct RefundInfo<M: ManagedTypeApi> {
    pub address: ManagedAddress<M>,
    pub initial_batch_id: u64,
    pub initial_nonce: u64,
}

/// An empty contract. To be used as a template when starting a new contract from scratch.
#[multiversx_sc::contract]
pub trait MockEsdtSafe {
    #[init]
    fn init(&self, _eth_tx_gas_limit: BigUint) {}

    #[upgrade]
    fn upgrade(&self) {}

    #[payable("*")]
    #[endpoint(createRefundTransaction)]
    fn create_refund_transaction(
        &self,
        _to: EthAddress<Self::Api>,
        _opt_refund_info: OptionalValue<RefundInfo<Self::Api>>,
    ) {
    }

    #[only_owner]
    #[endpoint(withdrawTransactionFees)]
    fn withdraw_transaction_fees(
        &self,
        _token_id: TokenIdentifier,
        _multisig_owner: ManagedAddress,
    ) {
    }

    #[only_owner]
    #[endpoint(withdrawRefundFeesForEthereum)]
    fn withdraw_refund_fees_for_ethereum(
        &self,
        _token_id: TokenIdentifier,
        _multisig_owner: ManagedAddress,
    ) {
    }
}
