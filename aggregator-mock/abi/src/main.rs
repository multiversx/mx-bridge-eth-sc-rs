use aggregator_mock::*;
use elrond_wasm_debug::*;

fn main() {
	let contract = AggregatorMockImpl::new(TxContext::dummy());
	print!("{}", abi_json::contract_abi(&contract));
}
