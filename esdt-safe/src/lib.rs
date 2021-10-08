#![no_std]
#![allow(non_snake_case)]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use eth_address::*;
use transaction::*;

const DEFAULT_MAX_TX_BATCH_SIZE: usize = 10;
const DEFAULT_MIN_TX_BATCH_FETCH_BLOCK_DIFF: u64 = 100;
const DEFAULT_MIN_BLOCK_NONCE_DIFF: u64 = 5;

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

        for token in token_whitelist.into_vec() {
            require!(token.is_valid_esdt_identifier(), "Invalid token ID");
            self.token_whitelist().insert(token);
        }

        self.max_tx_batch_size().set(&DEFAULT_MAX_TX_BATCH_SIZE);
        self.min_block_nonce_diff()
            .set(&DEFAULT_MIN_BLOCK_NONCE_DIFF);
        self.min_tx_batch_fetch_block_diff()
            .set(&DEFAULT_MIN_TX_BATCH_FETCH_BLOCK_DIFF);
        self.eth_tx_gas_limit().set(&eth_tx_gas_limit);

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
    #[endpoint(setMinBlockNonceDiff)]
    fn set_min_block_nonce_diff(&self, new_min_block_nonce_diff: u64) -> SCResult<()> {
        require!(
            new_min_block_nonce_diff > 0,
            "Min block nonce diff must be more than 0"
        );

        self.min_block_nonce_diff().set(&new_min_block_nonce_diff);

        Ok(())
    }

    #[only_owner]
    #[endpoint(setMinTxBatchFetchBlockDiff)]
    fn set_min_tx_batch_fetch_block_diff(&self, min_tx_batch_fetch_block_diff: u64) {
        self.min_tx_batch_fetch_block_diff()
            .set(&min_tx_batch_fetch_block_diff);
    }

    #[only_owner]
    #[endpoint(fetchNextTransactionBatch)]
    fn fetch_next_transaction_batch(&self) -> SCResult<Vec<Transaction<Self::BigUint>>> {
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

        require!(!tx_batch.is_empty(), "Empty batch");

        let min_tx_batch_fetch_block_diff = self.min_tx_batch_fetch_block_diff().get();
        let last_tx_batch_fetch_block = self.last_tx_batch_fetch_block().get();
        let fetch_block_diff = current_block_nonce - last_tx_batch_fetch_block;

        require!(
            tx_batch.len() == max_tx_batch_size
                || fetch_block_diff >= min_tx_batch_fetch_block_diff,
            "Empty batch"
        );

        self.last_tx_batch_fetch_block().set(&current_block_nonce);

        Ok(tx_batch)
    }

    #[only_owner]
    #[endpoint(setTransactionBatchStatus)]
    fn set_transaction_batch_status(
        &self,
        #[var_args] tx_status_batch: VarArgs<(Address, TxNonce, TransactionStatus)>,
    ) -> SCResult<()> {
        for (sender, nonce, tx_status) in tx_status_batch.into_vec() {
            require!(
                self.transaction_status(&sender, nonce).get() == TransactionStatus::InProgress,
                "Transaction has to be executed first"
            );

            match tx_status {
                TransactionStatus::Executed => {
                    let tx = self.transactions_by_nonce(&sender).get(nonce);

                    // local burn role might be removed while tx is executed
                    // tokens will remain locked forever in that case
                    // otherwise, the whole batch would fail
                    if self.is_local_role_set(&tx.token_identifier, &EsdtLocalRole::Burn) {
                        self.burn_esdt_token(&tx.token_identifier, &tx.amount);
                    }
                }
                TransactionStatus::Rejected => {
                    let tx = self.transactions_by_nonce(&sender).get(nonce);

                    self.refund_esdt_token(&tx.from, &tx.token_identifier, &tx.amount);
                }
                _ => {
                    return sc_error!("Transaction status may only be set to Executed or Rejected")
                }
            }

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

    fn get_next_pending_transaction(&self) -> Option<Transaction<Self::BigUint>> {
        self.pending_transaction_address_nonce_list()
            .front()
            .map(|(sender, nonce)| self.transactions_by_nonce(&sender).get(nonce))
    }

    fn clear_next_pending_transaction(&self) {
        let _ = self.pending_transaction_address_nonce_list().pop_front();
    }

    // storage

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

    #[storage_mapper("lastTxBatchFetchBlock")]
    fn last_tx_batch_fetch_block(&self) -> SingleValueMapper<Self::Storage, u64>;

    // configurable

    #[storage_mapper("maxTxBatchSize")]
    fn max_tx_batch_size(&self) -> SingleValueMapper<Self::Storage, usize>;

    #[storage_mapper("minBlockNonceDiff")]
    fn min_block_nonce_diff(&self) -> SingleValueMapper<Self::Storage, u64>;

    #[storage_mapper("minTxBatchFetchBlockDiff")]
    fn min_tx_batch_fetch_block_diff(&self) -> SingleValueMapper<Self::Storage, u64>;
}
