elrond_wasm::imports!();
use elrond_wasm::sc_error;

use transaction::*;

extern crate aggregator;
pub use crate::aggregator::aggregator_interface::Round;

pub struct AggregatorResult<BigUint: BigUintApi> {
    pub egld_to_eth: BigUint,
    pub egld_to_eth_scaling: BigUint,
    pub transaction_gas_limits: TransactionGasLimits<BigUint>,
    pub priority_gas_costs: PriorityGasCosts<BigUint>,
}

pub fn try_parse_round<BigUint: BigUintApi>(
    optional_arg_round: OptionalArg<Round<BigUint>>,
) -> SCResult<AggregatorResult<BigUint>> {
    let round = match optional_arg_round {
        OptionalArg::Some(rnd) => rnd,
        OptionalArg::None => return sc_error!("no aggregator round"),
    };
    let submission = match round.answer {
        Some(sub) => sub,
        None => return sc_error!("no answer in round"),
    };

    match &submission.values[..] {
        [first, second, third, fourth, fifth, sixth, seventh, eighth] => Ok(AggregatorResult {
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
        }),
        _ => sc_error!("incorrect length of answer values"),
    }
}
