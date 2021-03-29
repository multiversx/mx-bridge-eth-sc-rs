#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

mod priority;
use priority::{Priority, PriorityGasCosts};

mod transaction_type;
use transaction_type::{TransactionGasLimits, TransactionType};

mod aggregator_result;

extern crate aggregator;
use crate::aggregator::aggregator_interface::{AggregatorInterface, AggregatorInterfaceProxy};

#[elrond_wasm_derive::contract(EthereumFeePrepayImpl)]
pub trait EthereumFeePrepay {
    #[init]
    fn init(&self, aggregator: Address) {
        self.aggregator().set(&aggregator);
        self.whitelist().insert(self.get_caller());
    }

    // balance management endpoints

    #[payable("EGLD")]
    #[endpoint(deposit)]
    fn deposit(&self, #[payment] payment: BigUint) {
        let caller = &self.get_caller();
        self.increase_balance(caller, &payment);
    }

    #[endpoint(withdraw)]
    fn withdraw(&self, amount: BigUint) -> SCResult<()> {
        let caller = &self.get_caller();
        sc_try!(self.try_decrease_balance(caller, &amount));
        self.send().direct_egld(caller, &amount, &[]);

        Ok(())
    }

    #[view(getDepositBalance)]
    fn get_deposit_balance(&self) -> BigUint {
        let caller = &self.get_caller();
        self.deposits()
            .get(caller)
            .unwrap_or_else(|| BigUint::zero())
    }

    // estimate endpoints

    #[endpoint(payFee)]
    fn pay_fee(
        &self,
        address: Address,
        relayer: Address,
        action: TransactionType,
        priority: Priority,
    ) -> SCResult<()> {
        sc_try!(self.require_whitelisted());

        let optional_arg_round =
            contract_call!(self, self.aggregator().get(), AggregatorInterfaceProxy)
                .latestRoundData()
                .execute_on_dest_context(self.get_gas_left(), self.send());
        let aggregator_result = sc_try!(aggregator_result::try_parse_round(optional_arg_round));
        let estimate = self.compute_estimate(
            aggregator_result.egld_to_eth,
            aggregator_result.egld_to_eth_scaling,
            action,
            aggregator_result.transaction_gas_limits,
            priority,
            aggregator_result.priority_gas_costs,
        );

        sc_try!(self.try_transfer(&address, &relayer, &estimate));

        Ok(())
    }

    fn compute_estimate(
        &self,
        egld_to_eth: BigUint,
        egld_to_eth_scaling: BigUint,
        transaction_type: TransactionType,
        transaction_gas_limits: TransactionGasLimits<BigUint>,
        priority: Priority,
        priority_gas_costs: PriorityGasCosts<BigUint>,
    ) -> BigUint {
        let gas_limit = match transaction_type {
            TransactionType::Ethereum => transaction_gas_limits.ethereum,
            TransactionType::Erc20 => transaction_gas_limits.erc20,
            TransactionType::Erc721 => transaction_gas_limits.erc721,
            TransactionType::Erc1155 => transaction_gas_limits.erc1155,
        };
        let gas_price = match priority {
            Priority::Fast => priority_gas_costs.fast,
            Priority::Average => priority_gas_costs.average,
            Priority::Low => priority_gas_costs.low,
        };
        egld_to_eth * gas_limit * gas_price / egld_to_eth_scaling
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
            self.is_whitelisted(&self.get_caller()),
            "only whitelisted callers allowed"
        );
        Ok(())
    }

    fn increase_balance(&self, address: &Address, amount: &BigUint) {
        let mut deposit = self
            .deposits()
            .get(address)
            .unwrap_or_else(|| BigUint::zero());
        deposit += amount;
        self.deposits().insert(address.clone(), deposit);
    }

    fn try_decrease_balance(&self, address: &Address, amount: &BigUint) -> SCResult<()> {
        let mut deposit = self
            .deposits()
            .get(address)
            .unwrap_or_else(|| BigUint::zero());

        require!(&deposit >= amount, "insufficient balance");

        deposit -= amount;
        self.deposits().insert(address.clone(), deposit);

        Ok(())
    }

    fn try_transfer(&self, from: &Address, to: &Address, amount: &BigUint) -> SCResult<()> {
        sc_try!(self.try_decrease_balance(from, amount));
        self.increase_balance(to, amount);
        
        Ok(())
    }

    #[storage_mapper("whitelist")]
    fn whitelist(&self) -> SetMapper<Self::Storage, Address>;

    #[storage_mapper("deposits")]
    fn deposits(&self) -> MapMapper<Self::Storage, Address, BigUint>;

    #[storage_mapper("aggregator")]
    fn aggregator(&self) -> SingleValueMapper<Self::Storage, Address>;
}
