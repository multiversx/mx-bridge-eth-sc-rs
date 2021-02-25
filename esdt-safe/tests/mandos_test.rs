extern crate esdt_safe;
use esdt_safe::*;
use elrond_wasm::*;
use elrond_wasm_debug::*;

fn contract_map() -> ContractMap<TxContext> {
	let mut contract_map = ContractMap::new();
	contract_map.register_contract(
		"file:../output/esdt-safe.wasm",
		Box::new(|context| Box::new(EsdtSafeImpl::new(context))),
	);
	contract_map
}

#[test]
fn deposit_test() {
	parse_execute_mandos("mandos/deposit_egld.scen.json", &contract_map());
}

#[test]
fn whithdraw_test() {
	parse_execute_mandos("mandos/withdraw.scen.json", &contract_map());
}

#[test]
fn create_transaction_ok_test() {
	parse_execute_mandos("mandos/create_transaction_ok.scen.json", &contract_map());
}

#[test]
fn create_transaction_not_enough_deposit_test() {
	parse_execute_mandos("mandos/create_transaction_not_enough_deposit.scen.json", &contract_map());
}
