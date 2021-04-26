#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use transaction::*;

pub mod aggregator_result;

extern crate aggregator;
use crate::aggregator::aggregator_interface::{AggregatorInterface, AggregatorInterfaceProxy};

#[elrond_wasm_derive::contract(EthereumFeePrepayImpl)]
pub trait EthereumFeePrepay {
    #[init]
    fn init(&self, aggregator: Address) {
        self.aggregator().set(&aggregator);
        self.whitelist().insert(self.blockchain().get_caller());
    }

    // balance management endpoints

    #[payable("EGLD")]
    #[endpoint(depositTransactionFee)]
    fn deposit_transaction_fee(&self, #[payment] payment: BigUint) {
        let caller = &self.blockchain().get_caller();
        self.increase_balance(caller, &payment);
    }

    /// defaults to max amount
    #[endpoint]
    fn withdraw(&self, #[var_args] opt_amount: OptionalArg<BigUint>) -> SCResult<()> {
        let caller = &self.blockchain().get_caller();
        let amount = match opt_amount {
            OptionalArg::Some(amt) => amt,
            OptionalArg::None => self.deposit(&caller).get(),
        };

        sc_try!(self.try_decrease_balance(caller, &amount));
        self.send().direct_egld(caller, &amount, &[]);

        Ok(())
    }

    // estimate endpoints

    #[endpoint(payFee)]
    fn pay_fee(
        &self,
        address: Address,
        relayer: Address,
        transaction_type: TransactionType,
        priority: Priority,
    ) -> SCResult<()> {
        sc_try!(self.require_whitelisted());

        let estimate = sc_try!(self.compute_estimate(transaction_type, priority));
        self.transfer(&address, &relayer, &estimate);

        Ok(())
    }

    #[endpoint(reserveFee)]
    fn reserve_fee(
        &self,
        address: Address,
        transaction_type: TransactionType,
        priority: Priority,
    ) -> SCResult<()> {
        sc_try!(self.require_whitelisted());

        let estimate = sc_try!(self.compute_estimate(transaction_type, priority));
        sc_try!(self.try_reserve_from_balance(&address, &estimate));

        Ok(())
    }

    #[endpoint(computeEstimate)]
    fn compute_estimate(
        &self,
        transaction_type: TransactionType,
        priority: Priority,
    ) -> SCResult<BigUint> {
        let optional_arg_round =
            contract_call!(self, self.aggregator().get(), AggregatorInterfaceProxy)
                .latestRoundData()
                .execute_on_dest_context(self.blockchain().get_gas_left(), self.send());
        let aggregator_result = sc_try!(aggregator_result::try_parse_round(optional_arg_round));

        let gas_limit = match transaction_type {
            TransactionType::Ethereum => aggregator_result.transaction_gas_limits.ethereum,
            TransactionType::Erc20 => aggregator_result.transaction_gas_limits.erc20,
            TransactionType::Erc721 => aggregator_result.transaction_gas_limits.erc721,
            TransactionType::Erc1155 => aggregator_result.transaction_gas_limits.erc1155,
        };
        let gas_price = match priority {
            Priority::Fast => aggregator_result.priority_gas_costs.fast,
            Priority::Average => aggregator_result.priority_gas_costs.average,
            Priority::Low => aggregator_result.priority_gas_costs.low,
        };

        Ok(aggregator_result.egld_to_eth * gas_limit * gas_price
            / aggregator_result.egld_to_eth_scaling)
    }

    // whitelist endpoints

    #[endpoint(addToWhitelist)]
    fn add_to_whitelist(&self, address: Address) -> SCResult<()> {
        only_owner!(self, "only owner may call this function");

        self.whitelist().insert(address);

        Ok(())
    }

    #[endpoint(removeFromWhitelist)]
    fn remove_from_whitelist(&self, address: Address) -> SCResult<()> {
        only_owner!(self, "only owner may call this function");

        self.whitelist().remove(&address);

        Ok(())
    }

    #[view(isWhitelisted)]
    fn is_whitelisted(&self, address: &Address) -> bool {
        self.whitelist().contains(address)
    }

    #[view(getWhitelist)]
    fn get_whitelist(&self) -> MultiResultVec<Address> {
        self.whitelist().iter().collect()
    }

    fn require_whitelisted(&self) -> SCResult<()> {
        require!(
            self.is_whitelisted(&self.blockchain().get_caller()),
            "only whitelisted callers allowed"
        );
        Ok(())
    }

    fn increase_balance(&self, address: &Address, amount: &BigUint) {
        let mut deposit = self.deposit(address).get();
        deposit += amount;
        self.deposit(address).set(&deposit);
    }

    fn try_decrease_balance(&self, address: &Address, amount: &BigUint) -> SCResult<()> {
        let mut deposit = self.deposit(address).get();

        require!(&deposit >= amount, "insufficient balance");

        deposit -= amount;
        self.deposit(address).set(&deposit);

        Ok(())
    }

    fn transfer(&self, from: &Address, to: &Address, amount: &BigUint) {
        let reserve = self.reserved_amount(from).get();

        // This is done to prevent a potential bug
        // If the fee increases even slightly between the "reserve" and "pay"
        // The call would fail with not enough funds
        let amount_to_send = if &reserve < amount {
            reserve
        } else {
            amount.clone()
        };

        self.decrease_reserve(from, &amount_to_send);
        self.increase_balance(to, &amount_to_send);
    }

    fn increase_reserve(&self, address: &Address, amount: &BigUint) {
        let mut reserve = self.reserved_amount(address).get();
        reserve += amount;
        self.reserved_amount(address).set(&reserve);
    }

    fn decrease_reserve(&self, address: &Address, amount: &BigUint) {
        let mut reserve = self.reserved_amount(address).get();
        reserve -= amount;
        self.reserved_amount(address).set(&reserve);
    }

    fn try_reserve_from_balance(&self, from: &Address, amount: &BigUint) -> SCResult<()> {
        sc_try!(self.try_decrease_balance(from, amount));
        self.increase_reserve(from, amount);

        Ok(())
    }

    // storage

    #[storage_mapper("whitelist")]
    fn whitelist(&self) -> SetMapper<Self::Storage, Address>;

    #[view(getDeposit)]
    #[storage_mapper("deposit")]
    fn deposit(&self, address: &Address) -> SingleValueMapper<Self::Storage, BigUint>;

    #[view(getReservedAmount)]
    #[storage_mapper("reservedAmount")]
    fn reserved_amount(&self, address: &Address) -> SingleValueMapper<Self::Storage, BigUint>;

    #[storage_mapper("aggregator")]
    fn aggregator(&self) -> SingleValueMapper<Self::Storage, Address>;
}
