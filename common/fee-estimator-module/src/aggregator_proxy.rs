multiversx_sc::imports!();

pub const GWEI_STRING: &[u8] = b"GWEI";

pub type AggregatorResultAsMultiValue<M> =
    MultiValue5<u32, ManagedBuffer<M>, ManagedBuffer<M>, BigUint<M>, u8>;

#[multiversx_sc::proxy]
pub trait Aggregator {
    #[view(latestPriceFeedOptional)]
    fn latest_price_feed_optional(
        &self,
        from: ManagedBuffer,
        to: ManagedBuffer,
    ) -> OptionalValue<AggregatorResultAsMultiValue<Self::Api>>;
}

pub struct AggregatorResult<M: ManagedTypeApi> {
    pub round_id: u32,
    pub from_token_name: ManagedBuffer<M>,
    pub to_token_name: ManagedBuffer<M>,
    pub price: BigUint<M>,
    pub decimals: u8,
}

impl<M: ManagedTypeApi> From<AggregatorResultAsMultiValue<M>> for AggregatorResult<M> {
    fn from(multi_result: AggregatorResultAsMultiValue<M>) -> Self {
        let (round_id, from_token_name, to_token_name, price, decimals) = multi_result.into_tuple();

        AggregatorResult {
            round_id,
            from_token_name,
            to_token_name,
            price,
            decimals,
        }
    }
}
