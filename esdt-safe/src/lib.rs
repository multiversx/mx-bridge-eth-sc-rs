#![no_std]
#![allow(non_snake_case)]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use eth_address::*;
use transaction::esdt_safe_batch::EsdtSafeTxBatchSplitInFields;
use transaction::*;

const DEFAULT_MAX_TX_BATCH_SIZE: usize = 10;
const DEFAULT_MAX_TX_BATCH_BLOCK_DURATION: u64 = 100;

#[elrond_wasm_derive::contract]
pub trait EsdtSafe: fee_estimator_module::FeeEstimatorModule + token_module::TokenModule {
    #[init]
    fn init(
        &self,
        fee_estimator_contract_address: Address,
        eth_tx_gas_limit: Self::BigUint,
        #[var_args] token_whitelist: VarArgs<TokenIdentifier>,
    ) -> SCResult<()> {
        self.fee_estimator_contract_address()
            .set(&fee_estimator_contract_address);
        self.eth_tx_gas_limit().set(&eth_tx_gas_limit);

        for token in token_whitelist.into_vec() {
            require!(token.is_valid_esdt_identifier(), "Invalid token ID");
            let _ = self.token_whitelist().insert(token);
        }

        self.max_tx_batch_size()
            .set_if_empty(&DEFAULT_MAX_TX_BATCH_SIZE);
        self.max_tx_batch_block_duration()
            .set_if_empty(&DEFAULT_MAX_TX_BATCH_BLOCK_DURATION);

        Ok(())
    }

    // endpoints - owner-only

    #[only_owner]
    #[endpoint(setMaxTxBatchSize)]
    fn set_max_tx_batch_size(&self, new_max_tx_batch_size: usize) -> SCResult<()> {
        require!(
            new_max_tx_batch_size > 0,
            "Max tx batch size must be more than 0"
        );

        self.max_tx_batch_size().set(&new_max_tx_batch_size);

        Ok(())
    }

    #[only_owner]
    #[endpoint(setMaxTxBatchBlockDuration)]
    fn set_max_tx_batch_block_duration(
        &self,
        new_max_tx_batch_block_duration: u64,
    ) -> SCResult<()> {
        require!(
            new_max_tx_batch_block_duration > 0,
            "Max tx batch block duration must be more than 0"
        );

        self.max_tx_batch_block_duration()
            .set(&new_max_tx_batch_block_duration);

        Ok(())
    }

    #[only_owner]
    #[endpoint(setTransactionBatchStatus)]
    fn set_transaction_batch_status(
        &self,
        batch_id: u64,
        #[var_args] tx_statuses: VarArgs<TransactionStatus>,
    ) -> SCResult<()> {
        let first_batch_id = self.first_batch_id().get();
        require!(
            batch_id == first_batch_id,
            "Batches must be processed in order"
        );

        let tx_batch = self.pending_batches(batch_id).get();
        require!(
            tx_batch.len() == tx_statuses.len(),
            "Invalid number of statuses provided"
        );

        for (tx, tx_status) in tx_batch.iter().zip(tx_statuses.into_vec().iter()) {
            match *tx_status {
                TransactionStatus::Executed => {
                    // local burn role might be removed while tx is executed
                    // tokens will remain locked forever in that case
                    // otherwise, the whole batch would fail
                    if self.is_local_role_set(&tx.token_identifier, &EsdtLocalRole::Burn) {
                        self.burn_esdt_token(&tx.token_identifier, &tx.amount);
                    }
                }
                TransactionStatus::Rejected => {
                    self.refund_esdt_token(&tx.from, &tx.token_identifier, &tx.amount);
                }
                _ => {
                    return sc_error!("Transaction status may only be set to Executed or Rejected")
                }
            }
        }

        let new_first_batch_id = first_batch_id + 1;

        // for the case when the last existing batch was processed
        // otherwise, we'd create a batch with the same ID again
        self.last_batch_id().update(|last_batch_id| {
            if *last_batch_id == first_batch_id {
                *last_batch_id = new_first_batch_id;
            }
        });
        self.first_batch_id().set(&new_first_batch_id);
        self.pending_batches(batch_id).clear();

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

        let required_fee = self.calculate_required_fee(&payment_token);
        require!(
            required_fee < payment_amount,
            "Transaction fees cost more than the entire bridged amount"
        );

        self.accumulated_transaction_fees(&payment_token)
            .update(|fees| *fees += &required_fee);

        let actual_bridged_amount = payment_amount - required_fee;
        let caller = self.blockchain().get_caller();
        let tx_nonce = self.last_tx_nonce().update(|last_tx_nonce| {
            *last_tx_nonce += 1;
            *last_tx_nonce
        });
        let tx = Transaction {
            block_nonce: self.blockchain().get_block_nonce(),
            nonce: tx_nonce,
            from: caller,
            to,
            token_identifier: payment_token,
            amount: actual_bridged_amount,
        };

        self.add_to_batch(tx);

        Ok(())
    }

    // views

    #[view(getCurrentTxBatch)]
    fn get_current_tx_batch(&self) -> OptionalResult<EsdtSafeTxBatchSplitInFields<Self::BigUint>> {
        let first_batch_id = self.first_batch_id().get();
        let first_batch = self.pending_batches(first_batch_id).get();

        if self.is_batch_full(&first_batch) {
            let batch_len = first_batch.len();
            let mut result_vec = Vec::with_capacity(batch_len);
            for tx in first_batch {
                result_vec.push(tx.into_multiresult());
            }

            return OptionalResult::Some((first_batch_id, result_vec.into()).into());
        }

        OptionalResult::None
    }

    // private

    fn add_to_batch(&self, transaction: Transaction<Self::BigUint>) {
        let last_batch_id = self.last_batch_id().get();
        let mut last_batch = self.pending_batches(last_batch_id).get();

        if self.is_batch_full(&last_batch) {
            self.create_new_batch(transaction);
        } else {
            last_batch.push(transaction);
            self.pending_batches(last_batch_id).set(&last_batch);
        }
    }

    #[allow(clippy::vec_init_then_push)]
    fn create_new_batch(&self, transaction: Transaction<Self::BigUint>) {
        let last_batch_id = self.last_batch_id().get();
        let new_batch_id = last_batch_id + 1;

        let mut new_batch = Vec::with_capacity(1);
        new_batch.push(transaction);

        self.pending_batches(new_batch_id).set(&new_batch);
        self.last_batch_id().set(&new_batch_id);
    }

    fn is_batch_full(&self, tx_batch: &[Transaction<Self::BigUint>]) -> bool {
        if tx_batch.is_empty() {
            return false;
        }

        let max_batch_size = self.max_tx_batch_size().get();
        if tx_batch.len() == max_batch_size {
            return true;
        }

        let current_block_nonce = self.blockchain().get_block_nonce();
        let first_tx_in_batch_block_nonce = tx_batch[0].block_nonce;
        let block_diff = current_block_nonce - first_tx_in_batch_block_nonce;
        let max_tx_batch_block_duration = self.max_tx_batch_block_duration().get();

        block_diff > max_tx_batch_block_duration
    }

    fn burn_esdt_token(&self, token_id: &TokenIdentifier, amount: &Self::BigUint) {
        self.send().esdt_local_burn(token_id, 0, amount);
    }

    fn refund_esdt_token(&self, to: &Address, token_id: &TokenIdentifier, amount: &Self::BigUint) {
        self.send()
            .direct(to, token_id, 0, amount, self.data_or_empty(to, b"refund"));
    }

    fn data_or_empty(&self, to: &Address, data: &'static [u8]) -> &[u8] {
        if self.blockchain().is_smart_contract(to) {
            &[]
        } else {
            data
        }
    }

    // storage

    #[storage_mapper("firstBatchId")]
    fn first_batch_id(&self) -> SingleValueMapper<Self::Storage, u64>;

    #[storage_mapper("lastBatchId")]
    fn last_batch_id(&self) -> SingleValueMapper<Self::Storage, u64>;

    #[storage_mapper("pendingBatches")]
    fn pending_batches(
        &self,
        batch_id: u64,
    ) -> SingleValueMapper<Self::Storage, Vec<Transaction<Self::BigUint>>>;

    #[storage_mapper("lastTxNonce")]
    fn last_tx_nonce(&self) -> SingleValueMapper<Self::Storage, u64>;

    // configurable

    #[storage_mapper("maxTxBatchSize")]
    fn max_tx_batch_size(&self) -> SingleValueMapper<Self::Storage, usize>;

    #[storage_mapper("maxTxBatchBlockDuration")]
    fn max_tx_batch_block_duration(&self) -> SingleValueMapper<Self::Storage, u64>;
}
