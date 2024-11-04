#![no_std]

#[allow(unused_imports)]
use multiversx_sc::imports::*;

/// An empty contract. To be used as a template when starting a new contract from scratch.
#[multiversx_sc::contract]
pub trait MockMultisig {
    #[init]
    fn init(
        &self,
        esdt_safe_sc_address: ManagedAddress,
        multi_transfer_sc_address: ManagedAddress,
        proxy_sc_address: ManagedAddress,
        bridged_tokens_wrapper_sc_address: ManagedAddress,
        price_aggregator_sc_address: ManagedAddress,
        _required_stake: BigUint,
        _slash_amount: BigUint,
        _quorum: usize,
        _board: MultiValueEncoded<ManagedAddress>,
    ) {
        require!(
            self.blockchain().is_smart_contract(&esdt_safe_sc_address),
            "Esdt Safe address is not a Smart Contract address"
        );
        self.esdt_safe_address().set(&esdt_safe_sc_address);

        require!(
            self.blockchain()
                .is_smart_contract(&multi_transfer_sc_address),
            "Multi Transfer address is not a Smart Contract address"
        );
        self.multi_transfer_esdt_address()
            .set(&multi_transfer_sc_address);

        require!(
            self.blockchain().is_smart_contract(&proxy_sc_address),
            "Proxy address is not a Smart Contract address"
        );
        self.proxy_address().set(&proxy_sc_address);

        require!(
            self.blockchain()
                .is_smart_contract(&bridged_tokens_wrapper_sc_address),
            "Bridged Tokens Wrapper address is not a Smart Contract address"
        );
        self.bridged_tokens_wrapper_address()
            .set(&bridged_tokens_wrapper_sc_address);

        require!(
            self.blockchain()
                .is_smart_contract(&price_aggregator_sc_address),
            "Price Aggregator address is not a Smart Contract address"
        );
        self.fee_estimator_address()
            .set(&price_aggregator_sc_address);
    }

    #[upgrade]
    fn upgrade(&self) {}

    #[view(getEsdtSafeAddress)]
    #[storage_mapper("esdtSafeAddress")]
    fn esdt_safe_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getMultiTransferEsdtAddress)]
    #[storage_mapper("multiTransferEsdtAddress")]
    fn multi_transfer_esdt_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getProxyAddress)]
    #[storage_mapper("proxyAddress")]
    fn proxy_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getBridgedTokensWrapperAddress)]
    #[storage_mapper("bridgedTokensWrapperAddress")]
    fn bridged_tokens_wrapper_address(
        &self,
    ) -> SingleValueMapper<Self::Api, ManagedAddress<Self::Api>>;

    #[view(getFeeEstimatorAddress)]
    #[storage_mapper("feeEstimatorAddress")]
    fn fee_estimator_address(&self) -> SingleValueMapper<Self::Api, ManagedAddress<Self::Api>>;
}
