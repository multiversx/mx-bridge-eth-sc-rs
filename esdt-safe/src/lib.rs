#![no_std]

elrond_wasm::imports!();

#[elrond_wasm_derive::contract(EsdtSafeImpl)]
pub trait EsdtSafe {
	#[init]
	fn init(&self, transaction_fee: BigUint) {
		self.set_transaction_fee(&transaction_fee);
	}

	// endpoints - owner-only

	#[endpoint(setTransactionFee)]
	fn set_transaction_fee_endpoint(&self, transaction_fee: BigUint) -> SCResult<()> {
		only_owner!(self, "only owner may call this function");

		self.set_transaction_fee(&transaction_fee);

		Ok(())
	}

	// endpoints

	#[payable("EGLD")]
	#[endpoint(depositEgldForTransactionFee)]
	fn deposit_egld_for_transaction_fee(&self, #[payment] payment: BigUint) {
		let caller = self.get_caller();
		let mut caller_deposit = self.get_deposit(&caller);
		caller_deposit += payment;
		self.set_deposit(&caller, &caller_deposit);
	}

	// storage

	// transaction fee, can only be set by owner

	#[view(getTransactionFee)]
	#[storage_get("transactionFee")]
	fn get_transaction_fee(&self) -> BigUint;

	#[storage_set("transactionFee")]
	fn set_transaction_fee(&self, transaction_fee: &BigUint);

	// nonce for each address

	#[view(getAccountNonce)]
	#[storage_get("accountNonce")]
	fn get_account_nonce(&self, address: &Address) -> u64;

	#[storage_set("accountNonce")]
	fn set_account_nonce(&self, address: &Address, nonce: u64);

	// eGLD amounts deposited by each address, for the sole purpose of paying for transaction fees

	#[view(getDeposit)]
	#[storage_get("deposit")]
	fn get_deposit(&self, address: &Address) -> BigUint;

	#[storage_set("deposit")]
	fn set_deposit(&self, address: &Address, deposit: &BigUint);
}
