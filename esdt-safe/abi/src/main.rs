use esdt_safe::*;
use elrond_wasm_debug::*;

fn main() {
	let contract = EsdtSafeImpl::new(TxContext::dummy());
	print!("{}", abi_json::contract_abi(&contract));
}
