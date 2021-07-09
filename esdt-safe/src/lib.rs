#![no_std]
#![allow(non_snake_case)]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

mod fee_estimator;

pub mod aggregator_proxy;

use eth_address::*;
use transaction::*;

const DEFAULT_MAX_TX_BATCH_SIZE: usize = 10;
const DEFAULT_MIN_BLOCK_NONCE_DIFF: u64 = 5;

#[elrond_wasm_derive::contract]
pub trait EsdtSafe: fee_estimator::FeeEstimatorModule {
    #[init]
    fn init(
        &self,
        fee_estimator_contract_address: Address,
        gas_station_contract_address: Address,
        min_value_of_bridged_tokens_in_dollars: Self::BigUint,
        #[var_args] token_whitelist: VarArgs<TokenIdentifier>,
    ) -> SCResult<()> {
        self.fee_estimator_contract_address()
            .set(&fee_estimator_contract_address);
        self.gas_station_contract_address()
            .set(&gas_station_contract_address);

        for token in token_whitelist.into_vec() {
            require!(token.is_valid_esdt_identifier(), "Invalid token ID");
            self.token_whitelist().insert(token);
        }

        self.max_tx_batch_size().set(&DEFAULT_MAX_TX_BATCH_SIZE);
        self.min_block_nonce_diff()
            .set(&DEFAULT_MIN_BLOCK_NONCE_DIFF);
        self.min_value_of_bridged_tokens_in_dollars()
            .set(&min_value_of_bridged_tokens_in_dollars);

        Ok(())
    }

    // endpoints - owner-only

    /// Owner is a multisig SC, so we can't send directly to the owner or caller address here
    #[endpoint(claimAccumulatedFees)]
    fn claim_accumulated_fees(&self, dest_address: Address) -> SCResult<()> {
        self.require_caller_owner()?;

        for token_id in self.token_whitelist().iter() {
            let accumulated_fees = self.accumulated_transaction_fees(&token_id).get();
            if accumulated_fees > 0 {
                self.accumulated_transaction_fees(&token_id).clear();
                
                self.send()
                    .direct(&dest_address, &token_id, &accumulated_fees, &[]);
            }
        }

        Ok(())
    }

    #[endpoint(addTokenToWhitelist)]
    fn add_token_to_whitelist(
        &self,
        token_id: TokenIdentifier,
        default_value_in_dollars: Self::BigUint,
    ) -> SCResult<()> {
        self.require_caller_owner()?;
        self.require_local_burn_role_set(&token_id)?;

        self.default_value_in_dollars(&token_id)
            .set(&default_value_in_dollars);
        self.token_whitelist().insert(token_id);

        Ok(())
    }

    #[endpoint(removeTokenFromWhitelist)]
    fn remove_token_from_whitelist(&self, token_id: TokenIdentifier) -> SCResult<()> {
        self.require_caller_owner()?;

        self.token_whitelist().remove(&token_id);
        self.default_value_in_dollars(&token_id).clear();

        Ok(())
    }

    #[endpoint(setMaxTxBatchSize)]
    fn set_max_tx_batch_size(&self, new_max_tx_batch_size: usize) -> SCResult<()> {
        self.require_caller_owner()?;
        require!(
            new_max_tx_batch_size > 0,
            "Max tx batch size must be more than 0"
        );

        self.max_tx_batch_size().set(&new_max_tx_batch_size);

        Ok(())
    }

    #[endpoint(setMinBlockNonceDiff)]
    fn set_min_block_nonce_diff(&self, new_min_block_nonce_diff: u64) -> SCResult<()> {
        self.require_caller_owner()?;
        require!(
            new_min_block_nonce_diff > 0,
            "Min block nonce diff must be more than 0"
        );

        self.min_block_nonce_diff().set(&new_min_block_nonce_diff);

        Ok(())
    }

    #[endpoint(setDefaultValueInDollars)]
    fn set_default_value_in_dollars(
        &self,
        token_id: TokenIdentifier,
        default_value_in_dollars: Self::BigUint,
    ) -> SCResult<()> {
        self.require_caller_owner()?;
        require!(
            self.token_whitelist().contains(&token_id),
            "Token is not in whitelist"
        );

        self.default_value_in_dollars(&token_id)
            .set(&default_value_in_dollars);

        Ok(())
    }

    #[endpoint(getNextTransactionBatch)]
    fn get_next_transaction_batch(&self) -> SCResult<Vec<Transaction<Self::BigUint>>> {
        self.require_caller_owner()?;

        let current_block_nonce = self.blockchain().get_block_nonce();
        let min_block_nonce_diff = self.min_block_nonce_diff().get();

        let mut tx_batch = Vec::new();
        let max_tx_batch_size = self.max_tx_batch_size().get();

        while let Some(tx) = self.get_next_pending_transaction() {
            let block_nonce_diff = current_block_nonce - tx.block_nonce;
            if block_nonce_diff < min_block_nonce_diff {
                break;
            }

            self.transaction_status(&tx.from, tx.nonce)
                .set(&TransactionStatus::InProgress);
            self.clear_next_pending_transaction();

            tx_batch.push(tx);
            if tx_batch.len() == max_tx_batch_size {
                break;
            }
        }

        Ok(tx_batch)
    }

    #[endpoint(setTransactionBatchStatus)]
    fn set_transaction_batch_status(
        &self,
        #[var_args] tx_status_batch: VarArgs<(Address, TxNonce, TransactionStatus)>,
    ) -> SCResult<()> {
        only_owner!(self, "only owner may call this function");

        for (sender, nonce, tx_status) in tx_status_batch.into_vec() {
            require!(
                self.transaction_status(&sender, nonce).get() == TransactionStatus::InProgress,
                "Transaction has to be executed first"
            );

            match tx_status {
                TransactionStatus::Executed => {
                    let tx = self.transactions_by_nonce(&sender).get(nonce);

                    self.require_local_burn_role_set(&tx.token_identifier)?;
                    self.burn_esdt_token(&tx.token_identifier, &tx.amount);
                }
                TransactionStatus::Rejected => {
                    let tx = self.transactions_by_nonce(&sender).get(nonce);

                    self.refund_esdt_token(&tx.from, &tx.token_identifier, &tx.amount);
                }
                _ => {
                    return sc_error!("Transaction status may only be set to Executed or Rejected")
                }
            }

            // storage cleanup
            self.transaction_status(&sender, nonce).clear();
            self.transactions_by_nonce(&sender).clear_entry(nonce);
        }

        Ok(())
    }

    // endpoints

    #[payable("*")]
    #[endpoint(createTransaction)]
    fn create_transaction(
        &self,
        #[payment_token] payment_token: TokenIdentifier,
        #[payment_amount] payment_amount: Self::BigUint,
        to: EthAddress,
    ) -> SCResult<()> {
        require!(
            self.call_value().esdt_token_nonce() == 0,
            "Only fungible ESDT tokens accepted"
        );
        require!(
            self.token_whitelist().contains(&payment_token),
            "Payment token is not on whitelist"
        );
        require!(!to.is_zero(), "Can't transfer to address zero");

        let token_value_in_dollars = self.get_value_in_dollars(&payment_token, &payment_amount);
        let min_value_bridged_tokens_in_dollars =
            self.min_value_of_bridged_tokens_in_dollars().get();

        require!(
            token_value_in_dollars >= min_value_bridged_tokens_in_dollars,
            "Amount of tokens to bridge is too low"
        );

        let required_fee = self.calculate_required_fee(&payment_token);

        require!(
            required_fee < payment_amount,
            "Transaction fees cost more than the entire bridged amount"
        );

        self.accumulated_transaction_fees(&payment_token)
            .update(|fees| *fees += &required_fee);

        let actual_bridged_amount = payment_amount - required_fee;
        let caller = self.blockchain().get_caller();
        let sender_nonce = self.transactions_by_nonce(&caller).len() + 1;
        let tx = Transaction {
            block_nonce: self.blockchain().get_block_nonce(),
            nonce: sender_nonce,
            from: caller.clone(),
            to,
            token_identifier: payment_token,
            amount: actual_bridged_amount,
        };

        self.transactions_by_nonce(&caller).push(&tx);

        self.transaction_status(&caller, sender_nonce)
            .set(&TransactionStatus::Pending);
        self.pending_transaction_address_nonce_list()
            .push_back((caller, sender_nonce));

        Ok(())
    }

    // private

    fn burn_esdt_token(&self, token_id: &TokenIdentifier, amount: &Self::BigUint) {
        self.send().esdt_local_burn(token_id, amount);
    }

    fn refund_esdt_token(&self, to: &Address, token_id: &TokenIdentifier, amount: &Self::BigUint) {
        self.send()
            .direct(to, token_id, amount, self.data_or_empty(to, b"refund"));
    }

    fn data_or_empty(&self, to: &Address, data: &'static [u8]) -> &[u8] {
        if self.blockchain().is_smart_contract(to) {
            &[]
        } else {
            data
        }
    }

    fn require_caller_owner(&self) -> SCResult<()> {
        only_owner!(self, "only owner may call this function");
        Ok(())
    }

    fn require_local_burn_role_set(&self, token_id: &TokenIdentifier) -> SCResult<()> {
        let roles = self.blockchain().get_esdt_local_roles(token_id);
        require!(
            roles.contains(&EsdtLocalRole::Burn),
            "Must set local burn role first"
        );

        Ok(())
    }

    fn get_next_pending_transaction(&self) -> Option<Transaction<Self::BigUint>> {
        self.pending_transaction_address_nonce_list()
            .front()
            .map(|(sender, nonce)| self.transactions_by_nonce(&sender).get(nonce))
    }

    fn clear_next_pending_transaction(&self) {
        let _ = self.pending_transaction_address_nonce_list().pop_front();
    }

    // storage

    // token whitelist

    #[storage_mapper("tokenWhitelist")]
    fn token_whitelist(&self) -> SetMapper<Self::Storage, TokenIdentifier>;

    // transactions for each address, sorted by nonce
    // due to how VecMapper works internally, nonces will start at 1

    #[storage_mapper("transactionsByNonce")]
    fn transactions_by_nonce(
        &self,
        address: &Address,
    ) -> VecMapper<Self::Storage, Transaction<Self::BigUint>>;

    #[view(getTransactionStatus)]
    #[storage_mapper("transactionStatus")]
    fn transaction_status(
        &self,
        sender: &Address,
        nonce: TxNonce,
    ) -> SingleValueMapper<Self::Storage, TransactionStatus>;

    #[storage_mapper("pendingTransactionList")]
    fn pending_transaction_address_nonce_list(
        &self,
    ) -> LinkedListMapper<Self::Storage, (Address, TxNonce)>;

    #[storage_mapper("accumulatedTransactionFees")]
    fn accumulated_transaction_fees(
        &self,
        token_id: &TokenIdentifier,
    ) -> SingleValueMapper<Self::Storage, Self::BigUint>;

    // configurable

    #[storage_mapper("maxTxBatchSize")]
    fn max_tx_batch_size(&self) -> SingleValueMapper<Self::Storage, usize>;

    #[storage_mapper("minBlockNonceDiff")]
    fn min_block_nonce_diff(&self) -> SingleValueMapper<Self::Storage, u64>;

    #[storage_mapper("minValueOfBridgedTokensInDollars")]
    fn min_value_of_bridged_tokens_in_dollars(
        &self,
    ) -> SingleValueMapper<Self::Storage, Self::BigUint>;
}
