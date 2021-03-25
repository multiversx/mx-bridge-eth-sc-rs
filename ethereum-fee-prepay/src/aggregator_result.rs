elrond_wasm::imports!();

pub use crate::priority::PriorityGasCosts;
pub use crate::transaction_type::TransactionGasLimits;

extern crate aggregator;
pub use crate::aggregator::aggregator_interface::Round;

pub struct AggregatorResult<BigUint: BigUintApi> {
    pub egld_to_eth: BigUint,
    pub egld_to_eth_scaling: BigUint,
    pub transaction_gas_limits: TransactionGasLimits<BigUint>,
    pub priority_gas_costs: PriorityGasCosts<BigUint>,
}

pub fn parse_aggregator_result<BigUint: BigUintApi>(
    result: AsyncCallResult<OptionalArg<Round<BigUint>>>,
) -> Result<AggregatorResult<BigUint>, SCError> {
    match result {
        AsyncCallResult::Ok(optional_arg_round) => {
            let round = optional_arg_round
                .into_option()
                .ok_or("no aggregator round")?;
            parse_round(round)
        }
        AsyncCallResult::Err(error) => Result::Err(error.err_msg.into()),
    }
}

fn parse_round<BigUint: BigUintApi>(
    round: Round<BigUint>,
) -> Result<AggregatorResult<BigUint>, SCError> {
    let submission = round.answer.ok_or("no answer in round")?;
    match &submission.values[..] {
        [first, second, third, fourth, fifth, sixth, seventh, eighth] => {
            Result::Ok(AggregatorResult {
                egld_to_eth: first.clone(),
                egld_to_eth_scaling: BigUint::from(10u64.pow(round.decimals as u32)),
                transaction_gas_limits: TransactionGasLimits {
                    ethereum: second.clone(),
                    erc20: third.clone(),
                    erc721: fourth.clone(),
                    erc1155: fifth.clone(),
                },
                priority_gas_costs: PriorityGasCosts {
                    fast: sixth.clone(),
                    average: seventh.clone(),
                    low: eighth.clone(),
                },
            })
        }
        _ => Result::Err("incorrect length of answer values".into()),
    }
}
