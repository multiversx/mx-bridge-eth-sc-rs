extern crate ethereum_fee_prepay;
use ethereum_fee_prepay::*;
extern crate oracle;
use oracle::*;
extern crate aggregator;
use aggregator::*;
use elrond_wasm::*;
use elrond_wasm_debug::*;

fn contract_map() -> ContractMap<TxContext> {
    let mut contract_map = ContractMap::new();
    contract_map.register_contract(
        "file:../../../sc-chainlink-rs/oracle/output/oracle.wasm",
        Box::new(|context| Box::new(OracleImpl::new(context))),
    );
    contract_map.register_contract(
        "file:../../../sc-chainlink-rs/aggregator/output/aggregator.wasm",
        Box::new(|context| Box::new(AggregatorImpl::new(context))),
    );
    contract_map.register_contract(
        "file:../output/ethereum-fee-prepay.wasm",
        Box::new(|context| Box::new(EthereumFeePrepayImpl::new(context))),
    );
    contract_map
}

#[test]
fn init_test() {
    parse_execute_mandos("mandos/init.scen.json", &contract_map());
}

#[test]
fn prepare_aggregator_test() {
    parse_execute_mandos("mandos/prepare-aggregator.scen.json", &contract_map());
}

#[test]
fn prepay_test() {
    parse_execute_mandos("mandos/prepay.scen.json", &contract_map());
}
