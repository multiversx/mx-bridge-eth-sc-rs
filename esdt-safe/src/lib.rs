#![no_std]
#![allow(non_snake_case)]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use core::convert::TryFrom;

use core::ops::Deref;
use eth_address::*;
use fee_estimator_module::GWEI_STRING;
use transaction::{transaction_status::TransactionStatus, Transaction};

const DEFAULT_MAX_TX_BATCH_SIZE: usize = 10;
const DEFAULT_MAX_TX_BATCH_BLOCK_DURATION: u64 = 100; // ~10 minutes

pub type PaymentsVec<M> = ManagedVec<M, EsdtTokenPayment<M>>;

pub struct TransactionDetails<Api: ManagedTypeApi> {
    pub batch_id: u64,
    pub tx_nonce: u64,
    pub payment_token: TokenIdentifier<Api>,
    pub actual_bridged_amount: BigUint<Api>,
    pub required_fee: BigUint<Api>,
    pub to_address: ManagedBuffer<Api>,
    pub is_refund_tx: bool,
}

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, Clone, ManagedVecItem, PartialEq)]
pub struct RefundInfo<M: ManagedTypeApi> {
    pub address: ManagedAddress<M>,
    pub initial_batch_id: u64,
    pub initial_nonce: u64,
}

#[multiversx_sc::contract]
pub trait EsdtSafe:
    fee_estimator_module::FeeEstimatorModule
    + token_module::TokenModule
    + tx_batch_module::TxBatchModule
    + max_bridged_amount_module::MaxBridgedAmountModule
    + multiversx_sc_modules::pause::PauseModule
    + storage_module::CommonStorageModule
{
    /// fee_estimator_contract_address - The address of a Price Aggregator contract,
    /// which will get the price of token A in token B
    ///
    /// eth_tx_gas_limit - The gas limit that will be used for transactions on the ETH side.
    /// Will be used to compute the fees for the transfer
    #[init]
    fn init(&self, eth_tx_gas_limit: BigUint) {
        self.eth_tx_gas_limit().set(&eth_tx_gas_limit);

        self.max_tx_batch_size()
            .set_if_empty(DEFAULT_MAX_TX_BATCH_SIZE);
        self.max_tx_batch_block_duration()
            .set_if_empty(DEFAULT_MAX_TX_BATCH_BLOCK_DURATION);

        // batch ID 0 is considered invalid
        self.first_batch_id().set_if_empty(1);
        self.last_batch_id().set_if_empty(1);

        // set ticker for "GWEI"
        let gwei_token_id = TokenIdentifier::from(GWEI_STRING);
        self.token_ticker(&gwei_token_id)
            .set(gwei_token_id.as_managed_buffer());

        self.set_paused(true);
    }

    #[upgrade]
    fn upgrade(&self, eth_tx_gas_limit: BigUint) {
        self.eth_tx_gas_limit().set(&eth_tx_gas_limit);

        self.max_tx_batch_size()
            .set_if_empty(DEFAULT_MAX_TX_BATCH_SIZE);
        self.max_tx_batch_block_duration()
            .set_if_empty(DEFAULT_MAX_TX_BATCH_BLOCK_DURATION);

        // batch ID 0 is considered invalid
        self.first_batch_id().set_if_empty(1);
        self.last_batch_id().set_if_empty(1);

        // set ticker for "GWEI"
        let gwei_token_id = TokenIdentifier::from(GWEI_STRING);
        self.token_ticker(&gwei_token_id)
            .set_if_empty(gwei_token_id.as_managed_buffer());

        self.set_paused(true);
    }

    /// Sets the statuses for the transactions, after they were executed on the Ethereum side.
    ///
    /// Only TransactionStatus::Executed (3) and TransactionStatus::Rejected (4) values are allowed.
    /// Number of provided statuses must be equal to number of transactions in the batch.
    #[only_owner]
    #[endpoint(setTransactionBatchStatus)]
    fn set_transaction_batch_status(
        &self,
        batch_id: u64,
        tx_statuses: MultiValueEncoded<TransactionStatus>,
    ) {
        let first_batch_id = self.first_batch_id().get();
        require!(
            batch_id == first_batch_id,
            "Batches must be processed in order"
        );

        let mut tx_batch = self.pending_batches(batch_id);
        require!(
            tx_batch.len() == tx_statuses.len(),
            "Invalid number of statuses provided"
        );

        for (tx, tx_status) in tx_batch.iter().zip(tx_statuses.to_vec().iter()) {
            // Since tokens don't exist in the EsdtSafe in the case of a refund transaction
            // we have no tokens to burn, nor to refund
            if tx.is_refund_tx {
                continue;
            }

            match tx_status {
                TransactionStatus::Executed => {}
                TransactionStatus::Rejected => {
                    let addr = ManagedAddress::try_from(tx.from.clone()).unwrap();
                    self.mark_refund(&addr, &tx.token_identifier, &tx.amount);
                }
                _ => {
                    sc_panic!("Transaction status may only be set to Executed or Rejected");
                }
            }

            self.set_status_event(
                batch_id,
                tx.from,
                tx.to,
                tx.token_identifier,
                tx.amount,
                tx.nonce,
                tx_status,
            );
        }

        self.clear_first_batch(&mut tx_batch);
    }

    /// Converts failed Ethereum -> MultiversX transactions to MultiversX -> Ethereum transaction.
    /// This is done every now and then to refund the tokens.
    ///
    /// As with normal MultiversX -> Ethereum transactions, a part of the tokens will be
    /// subtracted to pay for the fees
    #[payable("*")]
    #[endpoint(addRefundBatch)]
    fn add_refund_batch(&self, refund_transactions: ManagedVec<Transaction<Self::Api>>) {
        let caller = self.blockchain().get_caller();
        let multi_transfer_address = self.get_multi_transfer_address();
        require!(caller == multi_transfer_address, "Invalid caller");

        let refund_payments = self.call_value().all_esdt_transfers().deref().clone();
        require!(
            !refund_payments.is_empty(),
            "Cannot refund with no payments"
        );

        let block_nonce = self.blockchain().get_block_nonce();
        let mut cached_token_ids = ManagedVec::<Self::Api, TokenIdentifier>::new();
        let mut cached_prices = ManagedVec::<Self::Api, BigUint>::new();
        let mut new_transactions = ManagedVec::new();
        let mut original_tx_nonces = ManagedVec::<Self::Api, u64>::new();

        for (refund_tx, refund_payment) in refund_transactions.iter().zip(refund_payments.iter()) {
            require!(
                refund_tx.token_identifier == refund_payment.token_identifier,
                "Token identifiers do not match"
            );
            require!(
                refund_tx.amount == refund_payment.amount,
                "Amounts do not match"
            );

            let required_fee = match cached_token_ids
                .iter()
                .position(|id| *id == refund_tx.token_identifier)
            {
                Some(index) => (*cached_prices.get(index)).clone(),
                None => {
                    let queried_fee = self.calculate_required_fee(&refund_tx.token_identifier);
                    cached_token_ids.push(refund_tx.token_identifier.clone());
                    cached_prices.push(queried_fee.clone());

                    queried_fee
                }
            };

            if refund_tx.amount <= required_fee {
                self.failed_refunds().push(&refund_tx);
                continue;
            }

            let actual_bridged_amount = refund_tx.amount.clone() - &required_fee;
            self.refund_fees_for_ethereum(&refund_tx.token_identifier)
                .update(|fees| *fees += required_fee);
            let tx_nonce = self.get_and_save_next_tx_id();

            // "from" and "to" are inverted, since this was initially an Ethereum -> MultiversX tx
            let new_tx = Transaction {
                block_nonce,
                nonce: tx_nonce,
                from: refund_tx.to.clone(),
                to: refund_tx.from.clone(),
                token_identifier: refund_tx.token_identifier.clone(),
                amount: actual_bridged_amount.clone(),
                is_refund_tx: true,
            };
            new_transactions.push(new_tx);
            original_tx_nonces.push(refund_tx.nonce);

            let refund_token_id = refund_tx.token_identifier.clone();

            if self.mint_burn_token(&refund_token_id).get() {
                let burn_balances_mapper = self.burn_balances(&refund_token_id);
                let mint_balances_mapper = self.mint_balances(&refund_token_id);
                if !self.native_token(&refund_token_id).get() {
                    require!(
                        mint_balances_mapper.get()
                            >= &burn_balances_mapper.get() + &actual_bridged_amount,
                        "Not enough minted tokens!"
                    );
                }
                let burn_executed = self.internal_burn(&refund_token_id, &actual_bridged_amount);
                require!(burn_executed, "Cannot do the burn action!");
                burn_balances_mapper.update(|burned| {
                    *burned += &actual_bridged_amount;
                });
            } else {
                self.total_balances(&refund_token_id).update(|total| {
                    *total += &actual_bridged_amount;
                });
            }
        }

        let batch_ids = self.add_multiple_tx_to_batch(&new_transactions);
        for (i, tx) in new_transactions.iter().enumerate() {
            let batch_id = batch_ids.get(i);
            let original_tx_nonce = original_tx_nonces.get(i);

            self.add_refund_transaction_event(batch_id, tx.nonce, original_tx_nonce);
        }
    }

    #[endpoint(addRefundBatchForFailedTx)]
    fn add_refund_batch_for_failed_tx(&self) {
        let mut refund_transactions: ManagedVec<Transaction<Self::Api>> = ManagedVec::new();
        for failed_refund in self.failed_refunds().iter() {
            refund_transactions.push(failed_refund);
        }

        self.add_refund_batch(refund_transactions);
    }

    fn create_transaction_common(
        &self,
        to: EthAddress<Self::Api>,
        opt_min_bridge_amount: OptionalValue<BigUint<Self::Api>>,
    ) -> TransactionDetails<Self::Api> {
        require!(self.not_paused(), "Cannot create transaction while paused");
        require!(to != EthAddress::zero(), "Cannot send to an empty address");

        let (payment_token, payment_amount) = self.call_value().single_fungible_esdt();
        self.require_token_in_whitelist(&payment_token);

        let required_fee = self.calculate_required_fee(&payment_token);
        require!(
            required_fee < payment_amount.clone(),
            "Transaction fees cost more than the entire bridged amount"
        );

        match opt_min_bridge_amount {
            OptionalValue::Some(min_bridge_amount) => {
                require!(
                    required_fee < min_bridge_amount,
                    "Minimum bridged amount after fee is not achieved"
                );
            }
            OptionalValue::None => {}
        };

        self.require_below_max_amount(&payment_token, &payment_amount);
        let caller = self.blockchain().get_caller();

        let mut is_refund_tx = false;
        if caller == self.get_bridge_proxy_address() {
            is_refund_tx = true;
        }

        self.accumulated_transaction_fees(&payment_token)
            .update(|fees| *fees += &required_fee);

        let actual_bridged_amount = payment_amount.clone() - required_fee.clone();
        let tx_nonce = self.get_and_save_next_tx_id();
        let tx = Transaction {
            block_nonce: self.blockchain().get_block_nonce(),
            nonce: tx_nonce,
            from: caller.as_managed_buffer().clone(),
            to: to.as_managed_buffer().clone(),
            token_identifier: payment_token.clone(),
            amount: actual_bridged_amount.clone(),
            is_refund_tx,
        };

        let batch_id = self.add_to_batch(tx.clone());
        if self.mint_burn_token(&payment_token).get() {
            let burn_balances_mapper = self.burn_balances(&payment_token);
            let mint_balances_mapper = self.mint_balances(&payment_token);
            if !self.native_token(&payment_token).get() {
                require!(
                    mint_balances_mapper.get()
                        >= &burn_balances_mapper.get() + &actual_bridged_amount,
                    "Not enough minted tokens!"
                );
            }
            let burn_executed = self.internal_burn(&payment_token, &actual_bridged_amount);
            require!(burn_executed, "Cannot do the burn action!");
            burn_balances_mapper.update(|burned| {
                *burned += &actual_bridged_amount;
            });
        } else {
            self.total_balances(&payment_token).update(|total| {
                *total += &actual_bridged_amount;
            });
        }
        TransactionDetails {
            batch_id,
            tx_nonce,
            payment_token: payment_token.clone(),
            actual_bridged_amount,
            required_fee,
            to_address: tx.to,
            is_refund_tx,
        }
    }

    // endpoints

    /// Create an MultiversX -> Ethereum transaction. Only fungible tokens are accepted.
    ///
    /// Every transfer will have a part of the tokens subtracted as fees.
    /// The fee amount depends on the global eth_tx_gas_limit
    /// and the current GWEI price, respective to the bridged token
    ///
    /// fee_amount = price_per_gas_unit * eth_tx_gas_limit
    #[payable("*")]
    #[endpoint(createTransaction)]
    fn create_transaction(
        &self,
        to: EthAddress<Self::Api>,
        opt_min_bridge_amount: OptionalValue<BigUint<Self::Api>>,
    ) {
        let transaction_details = self.create_transaction_common(to, opt_min_bridge_amount);
        let caller = self.blockchain().get_caller();

        self.create_transaction_event(
            transaction_details.batch_id,
            transaction_details.tx_nonce,
            transaction_details.payment_token,
            transaction_details.actual_bridged_amount,
            transaction_details.required_fee,
            caller.as_managed_buffer().clone(),
            transaction_details.to_address,
        );
    }

    /// Create an Ethereum -> MultiversX refund transaction. Only fungible tokens are accepted.
    ///
    /// Every transfer will have a part of the tokens subtracted as fees.
    /// The fee amount depends on the global eth_tx_gas_limit
    /// and the current GWEI price, respective to the bridged token
    ///
    /// fee_amount = price_per_gas_unit * eth_tx_gas_limit
    #[payable("*")]
    #[endpoint(createRefundTransaction)]
    fn create_refund_transaction(
        &self,
        to: EthAddress<Self::Api>,
        opt_refund_info: OptionalValue<RefundInfo<Self::Api>>,
    ) {
        let bridge_proxy_address = self.get_bridge_proxy_address();
        let caller = self.blockchain().get_caller();

        require!(
            bridge_proxy_address == caller,
            "Only BridgeProxy SC can call this endpoint"
        );

        let transaction_details = self.create_transaction_common(to, OptionalValue::None);
        let caller = self.blockchain().get_caller();

        let refund_info = match opt_refund_info {
            OptionalValue::Some(refund_info) => refund_info,
            OptionalValue::None => RefundInfo {
                address: caller,
                initial_batch_id: 0u64,
                initial_nonce: 0,
            },
        };

        self.create_refund_transaction_event(
            transaction_details.batch_id,
            transaction_details.tx_nonce,
            transaction_details.payment_token,
            transaction_details.actual_bridged_amount,
            transaction_details.required_fee,
            refund_info.initial_batch_id,
            refund_info.initial_nonce,
        );
    }

    /// Claim funds for failed MultiversX -> Ethereum transactions.
    /// These are not sent automatically to prevent the contract getting stuck.
    /// For example, if the receiver is a SC, a frozen account, etc.
    #[endpoint(claimRefund)]
    fn claim_refund(&self, token_id: TokenIdentifier) -> EsdtTokenPayment<Self::Api> {
        let caller = self.blockchain().get_caller();
        let refund_amount = self.refund_amount(&caller, &token_id).get();
        require!(refund_amount > 0, "Nothing to refund");

        self.refund_amount(&caller, &token_id).clear();
        self.total_refund_amount(&token_id)
            .update(|total| *total -= &refund_amount);
        self.rebalance_for_refund(&token_id, &refund_amount);

        self.tx()
            .to(ToCaller)
            .single_esdt(&token_id, 0, &refund_amount)
            .transfer();

        self.claim_refund_transaction_event(&token_id, caller);
        EsdtTokenPayment::new(token_id, 0, refund_amount)
    }

    #[only_owner]
    #[endpoint(withdrawRefundFeesForEthereum)]
    fn withdraw_refund_fees_for_ethereum(
        &self,
        token_id: TokenIdentifier,
        multisig_owner: ManagedAddress,
    ) {
        let refund_fees_for_ethereum_mapper = self.refund_fees_for_ethereum(&token_id);
        require!(
            !refund_fees_for_ethereum_mapper.is_empty(),
            "There are no fees to withdraw"
        );
        let amount_out = refund_fees_for_ethereum_mapper.get();
        self.tx()
            .to(multisig_owner)
            .single_esdt(&token_id, 0, &amount_out)
            .transfer();
        refund_fees_for_ethereum_mapper.set(BigUint::zero());
    }

    #[only_owner]
    #[endpoint(withdrawTransactionFees)]
    fn withdraw_transaction_fees(&self, token_id: TokenIdentifier, multisig_owner: ManagedAddress) {
        let accumulated_transaction_fees_mapper = self.accumulated_transaction_fees(&token_id);
        require!(
            !accumulated_transaction_fees_mapper.is_empty(),
            "There are no fees to withdraw"
        );
        let amount_out = accumulated_transaction_fees_mapper.get();
        self.tx()
            .to(multisig_owner)
            .single_esdt(&token_id, 0, &amount_out)
            .transfer();
        accumulated_transaction_fees_mapper.set(BigUint::zero());
    }

    #[view(computeTotalAmmountsFromIndex)]
    fn compute_total_amounts_from_index(
        &self,
        start_index: u64,
        end_index: u64,
    ) -> PaymentsVec<Self::Api> {
        let mut all_payments = PaymentsVec::new();
        for index in start_index..end_index {
            let last_batch = self.pending_batches(index);
            for tx in last_batch.iter() {
                let new_payment = EsdtTokenPayment::new(tx.token_identifier, 0, tx.amount);
                let len = all_payments.len();

                let mut updated = false;
                for i in 0..len {
                    let mut current_payment = all_payments.get(i).clone();
                    if current_payment.token_identifier != new_payment.token_identifier {
                        continue;
                    }

                    current_payment.amount += &new_payment.amount;
                    let _ = all_payments.set(i, current_payment);

                    updated = true;
                    break;
                }
                if !updated {
                    all_payments.push(new_payment);
                }
            }
        }

        all_payments
    }

    /// Query function that lists all refund amounts for a user.
    /// Useful for knowing which token IDs to pass to the claimRefund endpoint.
    #[view(getRefundAmounts)]
    fn get_refund_amounts(
        &self,
        address: ManagedAddress,
        opt_tokens: OptionalValue<MultiValueEncoded<TokenIdentifier<Self::Api>>>,
    ) -> MultiValueEncoded<MultiValue2<TokenIdentifier, BigUint>> {
        let mut refund_amounts = MultiValueEncoded::new();
        match opt_tokens {
            OptionalValue::Some(tokens) => {
                for token_id in tokens {
                    self.get_refund_amount_for_token(&address, token_id, &mut refund_amounts);
                }
            }
            OptionalValue::None => {
                for token_id in self.token_whitelist().iter() {
                    let amount = self.refund_amount(&address, &token_id).get();
                    if amount > 0u32 {
                        refund_amounts.push((token_id, amount).into());
                    }
                }
            }
        }

        refund_amounts
    }

    fn get_refund_amount_for_token(
        &self,
        address: &ManagedAddress,
        token_id: TokenIdentifier<Self::Api>,
        refund_amounts: &mut MultiValueEncoded<MultiValue2<TokenIdentifier, BigUint>>,
    ) {
        let amount = self.refund_amount(address, &token_id).get();
        if amount > 0u32 {
            refund_amounts.push((token_id, amount).into());
        }
    }

    // views

    #[view(getTotalRefundAmounts)]
    fn get_total_refund_amounts(&self) -> MultiValueEncoded<MultiValue2<TokenIdentifier, BigUint>> {
        let mut refund_amounts = MultiValueEncoded::new();
        for token_id in self.token_whitelist().iter() {
            let amount = self.total_refund_amount(&token_id).get();
            if amount > 0u32 {
                refund_amounts.push((token_id, amount).into());
            }
        }

        refund_amounts
    }

    #[view(getRefundFeesForEthereum)]
    fn get_refund_fees_for_ethereum(&self, token_id: TokenIdentifier) -> BigUint {
        let refund_fees_for_ethereum_mapper = self.refund_fees_for_ethereum(&token_id);
        if refund_fees_for_ethereum_mapper.is_empty() {
            BigUint::zero()
        } else {
            refund_fees_for_ethereum_mapper.get()
        }
    }

    #[view(getTransactionFees)]
    fn get_transaction_fees(&self, token_id: TokenIdentifier) -> BigUint {
        let accumulated_transaction_fees_mapper = self.accumulated_transaction_fees(&token_id);
        if accumulated_transaction_fees_mapper.is_empty() {
            BigUint::zero()
        } else {
            accumulated_transaction_fees_mapper.get()
        }
    }
    // private

    fn rebalance_for_refund(&self, token_id: &TokenIdentifier, amount: &BigUint) {
        let mintBurnToken = self.mint_burn_token(token_id).get();
        if !mintBurnToken {
            let total_balances_mapper = self.total_balances(token_id);
            total_balances_mapper.update(|total| {
                *total -= amount;
            });
        } else {
            let mint_balances_mapper = self.mint_balances(token_id);
            let mint_executed = self.internal_mint(token_id, amount);
            require!(mint_executed, "Cannot do the mint action!");

            mint_balances_mapper.update(|minted| {
                *minted += amount;
            });
        }
    }

    fn mark_refund(&self, to: &ManagedAddress, token_id: &TokenIdentifier, amount: &BigUint) {
        self.refund_amount(to, token_id)
            .update(|refund| *refund += amount);
        self.total_refund_amount(token_id)
            .update(|total| *total += amount);
    }

    // events

    #[event("createTransactionEvent")]
    fn create_transaction_event(
        &self,
        #[indexed] batch_id: u64,
        #[indexed] tx_id: u64,
        #[indexed] token_id: TokenIdentifier,
        #[indexed] amount: BigUint,
        #[indexed] fee: BigUint,
        #[indexed] sender: ManagedBuffer,
        #[indexed] recipient: ManagedBuffer,
    );

    #[event("createRefundTransactionEvent")]
    fn create_refund_transaction_event(
        &self,
        #[indexed] batch_id: u64,
        #[indexed] tx_id: u64,
        #[indexed] token_id: TokenIdentifier,
        #[indexed] amount: BigUint,
        #[indexed] fee: BigUint,
        #[indexed] initial_batch_id: u64,
        #[indexed] initial_tx_id: u64,
    );

    #[event("createRefundTransactionEvent")]
    fn create_refund_transaction_sc_call_event(
        &self,
        #[indexed] batch_id: u64,
        #[indexed] tx_id: u64,
        #[indexed] token_id: TokenIdentifier,
        #[indexed] amount: BigUint,
        #[indexed] fee: BigUint,
        #[indexed] initial_batch_id: u64,
        #[indexed] initial_tx_id: u64,
        #[indexed] data: ManagedBuffer,
    );

    #[event("createTransactionScCallEvent")]
    fn create_transaction_sc_call_event(
        &self,
        #[indexed] batch_id: u64,
        #[indexed] tx_nonce: u64,
        #[indexed] payment_token: TokenIdentifier,
        #[indexed] amount: BigUint,
        #[indexed] fee: BigUint,
        #[indexed] to: ManagedBuffer,
        #[indexed] from: ManagedBuffer,
        #[indexed] data: ManagedBuffer,
    );

    #[event("addRefundTransactionEvent")]
    fn add_refund_transaction_event(
        &self,
        #[indexed] batch_id: u64,
        #[indexed] tx_id: u64,
        #[indexed] original_tx_id: u64,
    );

    #[event("claimRefundTransactionEvent")]
    fn claim_refund_transaction_event(
        &self,
        #[indexed] token_id: &TokenIdentifier,
        #[indexed] caller: ManagedAddress,
    );

    #[event("setStatusEvent")]
    fn set_status_event(
        &self,
        #[indexed] batch_id: u64,
        #[indexed] from: ManagedBuffer,
        #[indexed] to: ManagedBuffer,
        #[indexed] token_id: TokenIdentifier,
        #[indexed] amount: BigUint,
        #[indexed] tx_id: u64,
        #[indexed] tx_status: TransactionStatus,
    );

    // storage

    #[storage_mapper("totalRefundAmount")]
    fn total_refund_amount(&self, token_id: &TokenIdentifier) -> SingleValueMapper<BigUint>;

    #[storage_mapper("refundFeesForEthereum")]
    fn refund_fees_for_ethereum(&self, token_id: &TokenIdentifier) -> SingleValueMapper<BigUint>;

    #[storage_mapper("refundAmount")]
    fn refund_amount(
        &self,
        address: &ManagedAddress,
        token_id: &TokenIdentifier,
    ) -> SingleValueMapper<BigUint>;

    #[storage_mapper("failedRefunds")]
    fn failed_refunds(&self) -> VecMapper<Transaction<Self::Api>>;
}
