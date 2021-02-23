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
fn test_mandos() {
	//parse_execute_mandos("mandos/adder.scen.json", &contract_map());
}
