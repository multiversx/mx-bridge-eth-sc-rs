#![no_std]

elrond_wasm::imports!();

mod aggregator_proxy;
pub use aggregator_proxy::*;

const TICKER_SEPARATOR: u8 = b'-';

#[elrond_wasm_derive::module]
pub trait FeeEstimatorModule {
    #[endpoint(setDefaultPricePerGwei)]
    fn set_default_price_per_gwei(
        &self,
        token_id: TokenIdentifier,
        default_gwei_price: Self::BigUint,
    ) -> SCResult<()> {
        only_owner!(self, "Only owner may call this function");

        self.default_price_per_gwei(&token_id).set(&default_gwei_price);

        Ok(())
    }

    #[endpoint(setFeeEstimatorContractAddress)]
    fn set_fee_estimator_contract_address(&self, new_address: Address) -> SCResult<()> {
        only_owner!(self, "Only owner may call this function");

        self.fee_estimator_contract_address().set(&new_address);

        Ok(())
    }

    #[view(calculateRequiredFee)]
    fn calculate_required_fee(&self, token_id: &TokenIdentifier) -> Self::BigUint {
        let price_per_gwei = self.get_price_per_gwei(token_id);
        let gas_limit = self.eth_tx_gas_limit().get();

        price_per_gwei * gas_limit
    }

    fn get_price_per_gwei(&self, token_id: &TokenIdentifier) -> Self::BigUint {
        let opt_price = self.get_aggregator_mapping(GWEI_STRING.into(), token_id.clone());

        opt_price.unwrap_or_else(|| self.default_price_per_gwei(token_id).get())
    }

    fn get_aggregator_mapping(
        &self,
        from: TokenIdentifier,
        to: TokenIdentifier,
    ) -> Option<Self::BigUint> {
        let fee_estimator_sc_address = self.fee_estimator_contract_address().get();
        if fee_estimator_sc_address.is_zero() {
            return None;
        }

        let from_ticker = self.get_token_ticker(from);
        let to_ticker = self.get_token_ticker(to);

        let result: OptionalResult<AggregatorResultAsMultiResult<Self::BigUint>> = self
            .aggregator_proxy(fee_estimator_sc_address)
            .latest_price_feed_optional(from_ticker, to_ticker)
            .execute_on_dest_context();

        result
            .into_option()
            .map(|multi_result| AggregatorResult::from(multi_result).price)
    }

    fn get_token_ticker(&self, token_id: TokenIdentifier) -> BoxedBytes {
        for (i, char) in token_id.as_esdt_identifier().iter().enumerate() {
            if *char == TICKER_SEPARATOR {
                return token_id.as_esdt_identifier()[..i].into();
            }
        }

        token_id.into_boxed_bytes()
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
    fn default_price_per_gwei(
        &self,
        token_id: &TokenIdentifier,
    ) -> SingleValueMapper<Self::Storage, Self::BigUint>;

    #[view(getEthTxGasLimit)]
    #[storage_mapper("ethTxGasLimit")]
    fn eth_tx_gas_limit(&self) -> SingleValueMapper<Self::Storage, Self::BigUint>;
}
