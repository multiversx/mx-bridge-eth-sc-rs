use multi_transfer_esdt::*;
use elrond_wasm_debug::*;

fn main() {
	let contract = MultiTransferEsdtImpl::new(TxContext::dummy());
	print!("{}", abi_json::contract_abi(&contract));
}
