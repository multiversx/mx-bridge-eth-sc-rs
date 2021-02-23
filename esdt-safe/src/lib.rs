#![no_std]

elrond_wasm::imports!();

#[elrond_wasm_derive::contract(EsdtSafeImpl)]
pub trait EsdtSafe {
	#[init]
	fn init(&self) {}
}
