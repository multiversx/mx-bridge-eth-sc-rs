#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

mod aggregator_proxy;
use aggregator_proxy::*;

const GWEI_STRING: &[u8] = b"GWEI";
const EGLD_STRING: &[u8] = b"EGLD";
const ETH_STRING: &[u8] = b"ETH";
const ETH_ERC20_TX_GAS_LIMIT: u64 = 150_000;

// Only used until we have a real aggregator setup
const DEFAULT_EGLD_TX_FEE: u64 = 100_000_000_000_000_000; // 0.1 EGLD
const DEFAULT_ETH_TX_FEE: u64 = 5_000_000_000_000_000; // 0.005 Wrapped ETH

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, Clone, Copy)]
pub enum TxFeePaymentToken {
    Egld,
    WrappedEth,
}

#[elrond_wasm_derive::contract]
pub trait EthereumFeePrepay {
    #[proxy]
    fn aggregator_proxy(&self, sc_address: Address) -> aggregator_proxy::Proxy<Self::SendApi>;

    /// Pass zero-address for aggregator to use the default values for tx fees
    #[init]
    fn init(&self, aggregator: Address, wrapped_eth_token_id: TokenIdentifier) -> SCResult<()> {
        self.aggregator().set(&aggregator);
        self.whitelist().insert(self.blockchain().get_caller());

        require!(
            wrapped_eth_token_id.is_valid_esdt_identifier(),
            "Invalid token ID"
        );
        self.wrapped_eth_token_id().set(&wrapped_eth_token_id);

        Ok(())
    }

    // balance management endpoints

    #[payable("*")]
    #[endpoint(depositTransactionFee)]
    fn deposit_transaction_fee(
        &self,
        #[payment_token] payment_token: TokenIdentifier,
        #[payment] payment: Self::BigUint,
    ) -> SCResult<()> {
        let caller = &self.blockchain().get_caller();
        let tx_fee_payment_token = self.try_convert_to_tx_fee_payment_token(&payment_token)?;

        self.increase_balance(caller, tx_fee_payment_token, &payment);

        Ok(())
    }

    /// defaults to max amount
    #[endpoint]
    fn withdraw(
        &self,
        tx_fee_payment_token: TxFeePaymentToken,
        #[var_args] opt_amount: OptionalArg<Self::BigUint>,
    ) -> SCResult<()> {
        let caller = self.blockchain().get_caller();
        let token_id = self.convert_to_token_id(tx_fee_payment_token);
        let amount = match opt_amount {
            OptionalArg::Some(amt) => amt,
            OptionalArg::None => self.deposit(&caller, tx_fee_payment_token).get(),
        };

        self.try_decrease_balance(&caller, tx_fee_payment_token, &amount)?;
        self.send().direct(&caller, &token_id, &amount, &[]);

        Ok(())
    }

    /// Owner is a multisig SC, so we can't send directly to the owner or caller address here
    #[endpoint(claimAccumulatedFees)]
    fn claim_accumulated_fees(&self, dest_address: Address) -> SCResult<()> {
        self.require_whitelisted()?;

        let accumulated_egld = self.accumulated_tx_fees(TxFeePaymentToken::Egld).get();
        let accumulated_wrapped_eth = self
            .accumulated_tx_fees(TxFeePaymentToken::WrappedEth)
            .get();

        if accumulated_egld > 0 {
            self.send().direct(
                &dest_address,
                &TokenIdentifier::egld(),
                &accumulated_egld,
                &[],
            );
        }

        if accumulated_wrapped_eth > 0 {
            self.send().direct(
                &dest_address,
                &self.wrapped_eth_token_id().get(),
                &accumulated_wrapped_eth,
                &[],
            );
        }

        self.accumulated_tx_fees(TxFeePaymentToken::Egld).clear();
        self.accumulated_tx_fees(TxFeePaymentToken::WrappedEth)
            .clear();

        Ok(())
    }

    // estimate endpoints

    #[endpoint(reserveFee)]
    fn reserve_fee(
        &self,
        sender_address: Address,
        token_used_for_fee_payment: TokenIdentifier,
    ) -> SCResult<()> {
        self.require_whitelisted()?;

        let tx_fee_payment_token =
            self.try_convert_to_tx_fee_payment_token(&token_used_for_fee_payment)?;
        let estimate = self.compute_estimate(tx_fee_payment_token);

        self.try_decrease_balance(&sender_address, tx_fee_payment_token, &estimate)?;
        self.accumulated_tx_fees(tx_fee_payment_token)
            .update(|tx_fees| *tx_fees += estimate);

        Ok(())
    }

    #[endpoint(computeEstimate)]
    fn compute_estimate(&self, tx_fee_payment_token: TxFeePaymentToken) -> Self::BigUint {
        let aggregator_address = self.aggregator().get();

        if aggregator_address.is_zero() {
            match tx_fee_payment_token {
                TxFeePaymentToken::Egld => return DEFAULT_EGLD_TX_FEE.into(),
                TxFeePaymentToken::WrappedEth => return DEFAULT_ETH_TX_FEE.into(),
            }
        }

        let (from_token_name, to_token_name) = match tx_fee_payment_token {
            TxFeePaymentToken::Egld => {
                (BoxedBytes::from(GWEI_STRING), BoxedBytes::from(EGLD_STRING))
            }
            TxFeePaymentToken::WrappedEth => {
                (BoxedBytes::from(GWEI_STRING), BoxedBytes::from(ETH_STRING))
            }
        };

        let aggregator_result: AggregatorResult<Self::BigUint> = self
            .aggregator_proxy(aggregator_address)
            .latest_price_feed(from_token_name, to_token_name)
            .execute_on_dest_context()
            .into();

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

    fn increase_balance(
        &self,
        address: &Address,
        tx_fee_payment_token: TxFeePaymentToken,
        amount: &Self::BigUint,
    ) {
        self.deposit(address, tx_fee_payment_token)
            .update(|deposit| *deposit += amount);
    }

    fn try_decrease_balance(
        &self,
        address: &Address,
        tx_fee_payment_token: TxFeePaymentToken,
        amount: &Self::BigUint,
    ) -> SCResult<()> {
        self.deposit(address, tx_fee_payment_token)
            .update(|deposit| {
                require!(&*deposit >= amount, "insufficient balance");
                *deposit -= amount;
                Ok(())
            })
    }

    fn try_convert_to_tx_fee_payment_token(
        &self,
        token_id: &TokenIdentifier,
    ) -> SCResult<TxFeePaymentToken> {
        if token_id.is_egld() {
            Ok(TxFeePaymentToken::Egld)
        } else if token_id == &self.wrapped_eth_token_id().get() {
            Ok(TxFeePaymentToken::WrappedEth)
        } else {
            sc_error!("Wrong payment token")
        }
    }

    fn convert_to_token_id(&self, tx_fee_payment_token: TxFeePaymentToken) -> TokenIdentifier {
        match tx_fee_payment_token {
            TxFeePaymentToken::Egld => TokenIdentifier::egld(),
            TxFeePaymentToken::WrappedEth => self.wrapped_eth_token_id().get(),
        }
    }

    // storage

    #[storage_mapper("whitelist")]
    fn whitelist(&self) -> SetMapper<Self::Storage, Address>;

    #[storage_mapper("wrappedEthTokenId")]
    fn wrapped_eth_token_id(&self) -> SingleValueMapper<Self::Storage, TokenIdentifier>;

    #[view(getDeposit)]
    #[storage_mapper("deposit")]
    fn deposit(
        &self,
        address: &Address,
        token: TxFeePaymentToken,
    ) -> SingleValueMapper<Self::Storage, Self::BigUint>;

    #[storage_mapper("accumulatedTxFees")]
    fn accumulated_tx_fees(
        &self,
        token: TxFeePaymentToken,
    ) -> SingleValueMapper<Self::Storage, Self::BigUint>;

    #[storage_mapper("aggregator")]
    fn aggregator(&self) -> SingleValueMapper<Self::Storage, Address>;
}
