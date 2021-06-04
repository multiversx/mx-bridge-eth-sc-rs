#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

mod aggregator_proxy;
use aggregator_proxy::*;

const GWEI_STRING: &[u8] = b"GWEI";
const EGLD_STRING: &[u8] = b"EGLD";
const ETH_ERC20_TX_GAS_LIMIT: u64 = 150_000;

#[elrond_wasm_derive::contract]
pub trait EthereumFeePrepay {
    #[proxy]
    fn aggregator_proxy(&self, sc_address: Address) -> aggregator_proxy::Proxy<Self::SendApi>;

    #[init]
    fn init(&self, aggregator: Address) {
        self.aggregator().set(&aggregator);
        self.whitelist().insert(self.blockchain().get_caller());
    }

    // balance management endpoints

    #[payable("EGLD")]
    #[endpoint(depositTransactionFee)]
    fn deposit_transaction_fee(&self, #[payment] payment: Self::BigUint) {
        let caller = &self.blockchain().get_caller();
        self.increase_balance(caller, &payment);
    }

    /// defaults to max amount
    #[endpoint]
    fn withdraw(&self, #[var_args] opt_amount: OptionalArg<Self::BigUint>) -> SCResult<()> {
        let caller = &self.blockchain().get_caller();
        let amount = match opt_amount {
            OptionalArg::Some(amt) => amt,
            OptionalArg::None => self.deposit(&caller).get(),
        };

        self.try_decrease_balance(caller, &amount)?;
        self.send().direct_egld(caller, &amount, &[]);

        Ok(())
    }

    // estimate endpoints

    #[endpoint(payFee)]
    fn pay_fee(&self, tx_senders: Vec<Address>, relayer: Address) -> SCResult<()> {
        self.require_whitelisted()?;

        // To save gas for the relayers, we always use the latest queried value
        // No need to check for empty, since this is guaranteed to be set by a previous CreateTransaction in EsdtSafe
        let estimate = self.last_query_price().get() * ETH_ERC20_TX_GAS_LIMIT.into();
        for tx_sender in tx_senders {
            self.transfer(&tx_sender, &relayer, &estimate);
        }

        Ok(())
    }

    #[endpoint(reserveFee)]
    fn reserve_fee(&self, address: Address) -> SCResult<()> {
        self.require_whitelisted()?;

        let estimate = self.compute_estimate();
        self.try_reserve_from_balance(&address, &estimate)?;

        Ok(())
    }

    #[endpoint(computeEstimate)]
    fn compute_estimate(&self) -> Self::BigUint {
        let aggregator_result: AggregatorResult<Self::BigUint> = self
            .aggregator_proxy(self.aggregator().get())
            .latest_price_feed(BoxedBytes::from(GWEI_STRING), BoxedBytes::from(EGLD_STRING))
            .execute_on_dest_context()
            .into();

        self.last_query_price().set(&aggregator_result.price);

        aggregator_result.price * ETH_ERC20_TX_GAS_LIMIT.into()
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

    fn increase_balance(&self, address: &Address, amount: &Self::BigUint) {
        let mut deposit = self.deposit(address).get();
        deposit += amount;
        self.deposit(address).set(&deposit);
    }

    fn try_decrease_balance(&self, address: &Address, amount: &Self::BigUint) -> SCResult<()> {
        let mut deposit = self.deposit(address).get();

        require!(&deposit >= amount, "insufficient balance");

        deposit -= amount;
        self.deposit(address).set(&deposit);

        Ok(())
    }

    fn transfer(&self, from: &Address, to: &Address, amount: &Self::BigUint) {
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
        self.send()
            .direct_egld(to, &amount_to_send, b"Ethereum tx fees");
    }

    fn increase_reserve(&self, address: &Address, amount: &Self::BigUint) {
        let mut reserve = self.reserved_amount(address).get();
        reserve += amount;
        self.reserved_amount(address).set(&reserve);
    }

    fn decrease_reserve(&self, address: &Address, amount: &Self::BigUint) {
        let mut reserve = self.reserved_amount(address).get();
        reserve -= amount;
        self.reserved_amount(address).set(&reserve);
    }

    fn try_reserve_from_balance(&self, from: &Address, amount: &Self::BigUint) -> SCResult<()> {
        self.try_decrease_balance(from, amount)?;
        self.increase_reserve(from, amount);

        Ok(())
    }

    // storage

    #[storage_mapper("whitelist")]
    fn whitelist(&self) -> SetMapper<Self::Storage, Address>;

    #[view(getDeposit)]
    #[storage_mapper("deposit")]
    fn deposit(&self, address: &Address) -> SingleValueMapper<Self::Storage, Self::BigUint>;

    #[view(getReservedAmount)]
    #[storage_mapper("reservedAmount")]
    fn reserved_amount(&self, address: &Address)
        -> SingleValueMapper<Self::Storage, Self::BigUint>;

    #[storage_mapper("aggregator")]
    fn aggregator(&self) -> SingleValueMapper<Self::Storage, Address>;

    #[storage_mapper("lastQueryPrice")]
    fn last_query_price(&self) -> SingleValueMapper<Self::Storage, Self::BigUint>;
}
