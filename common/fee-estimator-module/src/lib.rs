#![no_std]

elrond_wasm::imports!();

mod aggregator_proxy;
use aggregator_proxy::*;

pub const ETH_ERC20_TX_GAS_LIMIT: u64 = 150_000;
pub const DENOMINATION: u64 = 1_000_000_000_000_000_000;

#[elrond_wasm_derive::module]
pub trait FeeEstimatorModule {
    #[endpoint(setDefaultValueInDollars)]
    fn set_default_value_in_dollars(
        &self,
        token_id: TokenIdentifier,
        default_value_in_dollars: Self::BigUint,
    ) -> SCResult<()> {
        only_owner!(self, "Only owner may call this function");

        self.default_value_in_dollars(&token_id)
            .set(&default_value_in_dollars);

        Ok(())
    }

    fn get_value_in_dollars(&self, token_id: &TokenIdentifier) -> Self::BigUint {
        let fee_estimator_sc_address = self.fee_estimator_contract_address().get();
        if fee_estimator_sc_address.is_zero() {
            return self.default_value_in_dollars(token_id).get();
        }

        let result: OptionalResult<AggregatorResultAsMultiResult<Self::BigUint>> = self
            .aggregator_proxy(fee_estimator_sc_address)
            .latest_price_feed_optional(token_id.clone().into_boxed_bytes(), DOLLAR_STRING.into())
            .execute_on_dest_context();

        let opt_price = result
            .into_option()
            .map(|multi_result| AggregatorResult::from(multi_result).price);

        opt_price.unwrap_or_else(|| self.default_value_in_dollars(token_id).get())
    }

    fn calculate_required_fee(&self, token_id: &TokenIdentifier) -> Self::BigUint {
        let eth_gas_unit_cost = self.get_eth_rapid_gas_price_per_unit(token_id);

        eth_gas_unit_cost * ETH_ERC20_TX_GAS_LIMIT.into()
    }

    // TODO: Call the required endpoint from the gas station SC
    fn get_eth_rapid_gas_price_per_unit(&self, _token_id: &TokenIdentifier) -> Self::BigUint {
        Self::BigUint::zero()
    }

    // proxies

    #[proxy]
    fn aggregator_proxy(&self, sc_address: Address) -> aggregator_proxy::Proxy<Self::SendApi>;

    // storage

    #[view(getFeeEstimatorContractAddress)]
    #[storage_mapper("feeEstimatorContractAddress")]
    fn fee_estimator_contract_address(&self) -> SingleValueMapper<Self::Storage, Address>;

    #[view(getGasStationContractAddress)]
    #[storage_mapper("gasStationContractAddress")]
    fn gas_station_contract_address(&self) -> SingleValueMapper<Self::Storage, Address>;

    #[storage_mapper("defaultValueInDollars")]
    fn default_value_in_dollars(
        &self,
        token_id: &TokenIdentifier,
    ) -> SingleValueMapper<Self::Storage, Self::BigUint>;
}
