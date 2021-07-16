elrond_wasm::imports!();

pub const GWEI_STRING: &[u8] = b"GWEI";

pub type AggregatorResultAsMultiResult<BigUint> =
    MultiResult5<u32, BoxedBytes, BoxedBytes, BigUint, u8>;

#[elrond_wasm_derive::proxy]
pub trait Aggregator {
    #[view(latestPriceFeedOptional)]
    fn latest_price_feed_optional(
        &self,
        from: BoxedBytes,
        to: BoxedBytes,
    ) -> OptionalResult<AggregatorResultAsMultiResult<Self::BigUint>>;
}

pub struct AggregatorResult<BigUint: BigUintApi> {
    pub round_id: u32,
    pub from_token_name: BoxedBytes,
    pub to_token_name: BoxedBytes,
    pub price: BigUint,
    pub decimals: u8,
}

impl<BigUint: BigUintApi> From<AggregatorResultAsMultiResult<BigUint>>
    for AggregatorResult<BigUint>
{
    fn from(multi_result: AggregatorResultAsMultiResult<BigUint>) -> Self {
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
