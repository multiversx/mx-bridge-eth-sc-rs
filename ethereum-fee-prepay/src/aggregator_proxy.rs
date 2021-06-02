elrond_wasm::imports!();

type AggregatorResultAsMultiResult<BigUint> =
    MultiResult5<u32, BoxedBytes, BoxedBytes, BigUint, u8>;

#[elrond_wasm_derive::proxy]
pub trait Aggregator {
    #[endpoint(latestPriceFeed)]
    fn latest_price_feed(
        &self,
        from_token_name: BoxedBytes,
        to_token_name: BoxedBytes,
    ) -> SCResult<AggregatorResultAsMultiResult<Self::BigUint>>;
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
