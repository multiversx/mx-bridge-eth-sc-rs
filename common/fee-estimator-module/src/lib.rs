#![no_std]

elrond_wasm::imports!();

mod aggregator_proxy;
pub use aggregator_proxy::*;

#[elrond_wasm_derive::module]
pub trait FeeEstimatorModule {
    #[only_owner]
    #[endpoint(setFeeEstimatorContractAddress)]
    fn set_fee_estimator_contract_address(&self, new_address: Address) {
        self.fee_estimator_contract_address().set(&new_address);
    }

    #[only_owner]
    #[endpoint(setEthTxGasLimit)]
    fn set_eth_tx_gas_limit(&self, new_limit: Self::BigUint) {
        self.eth_tx_gas_limit().set(&new_limit);
    }

    #[only_owner]
    #[endpoint(setDefaultPricePerGwei)]
    fn set_default_price_per_gas_unit(
        &self,
        token_id: TokenIdentifier,
        default_gwei_price: Self::BigUint,
    ) {
        self.default_price_per_gas_unit(&token_id)
            .set(&default_gwei_price);
    }

    #[only_owner]
    #[endpoint(setTokenTicker)]
    fn set_token_ticker(&self, token_id: TokenIdentifier, ticker: BoxedBytes) {
        self.token_ticker(&token_id).set(&ticker);
    }

    #[view(calculateRequiredFee)]
    fn calculate_required_fee(&self, token_id: &TokenIdentifier) -> Self::BigUint {
        let price_per_gas_unit = self.get_price_per_gas_unit(token_id);
        let gas_limit = self.eth_tx_gas_limit().get();

        price_per_gas_unit * gas_limit
    }

    fn get_price_per_gas_unit(&self, token_id: &TokenIdentifier) -> Self::BigUint {
        let opt_price = self.get_aggregator_mapping(&GWEI_STRING.into(), token_id);

        opt_price.unwrap_or_else(|| self.default_price_per_gas_unit(token_id).get())
    }

    fn get_aggregator_mapping(
        &self,
        from: &TokenIdentifier,
        to: &TokenIdentifier,
    ) -> Option<Self::BigUint> {
        let fee_estimator_sc_address = self.fee_estimator_contract_address().get();
        if fee_estimator_sc_address.is_zero() {
            return None;
        }

        let from_ticker = self.token_ticker(from).get();
        let to_ticker = self.token_ticker(to).get();

        let result: OptionalResult<AggregatorResultAsMultiResult<Self::BigUint>> = self
            .aggregator_proxy(fee_estimator_sc_address)
            .latest_price_feed_optional(from_ticker, to_ticker)
            .execute_on_dest_context();

        result
            .into_option()
            .map(|multi_result| AggregatorResult::from(multi_result).price)
    }

    // proxies

    #[proxy]
    fn aggregator_proxy(&self, sc_address: Address) -> aggregator_proxy::Proxy<Self::SendApi>;

    // storage

    #[view(getFeeEstimatorContractAddress)]
    #[storage_mapper("feeEstimatorContractAddress")]
    fn fee_estimator_contract_address(&self) -> SingleValueMapper<Self::Storage, Address>;

    #[view(getDefaultPricePerGwei)]
    #[storage_mapper("defaultPricePerGwei")]
    fn default_price_per_gas_unit(
        &self,
        token_id: &TokenIdentifier,
    ) -> SingleValueMapper<Self::Storage, Self::BigUint>;

    #[storage_mapper("tokenTicker")]
    fn token_ticker(
        &self,
        token_id: &TokenIdentifier,
    ) -> SingleValueMapper<Self::Storage, BoxedBytes>;

    #[view(getEthTxGasLimit)]
    #[storage_mapper("ethTxGasLimit")]
    fn eth_tx_gas_limit(&self) -> SingleValueMapper<Self::Storage, Self::BigUint>;
}
