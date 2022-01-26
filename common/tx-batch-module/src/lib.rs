#![no_std]

use transaction::{esdt_safe_batch::TxBatchSplitInFields, Transaction, MIN_BLOCKS_FOR_FINALITY};

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[elrond_wasm::module]
pub trait TxBatchModule {
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

    // views

    #[view(getCurrentTxBatch)]
    fn get_current_tx_batch(&self) -> OptionalResult<TxBatchSplitInFields<Self::Api>> {
        let first_batch_id = self.first_batch_id().get();
        let first_batch = self.pending_batches(first_batch_id).get();

        if self.is_batch_full(&first_batch) && self.is_batch_final(&first_batch) {
            let mut result_vec = ManagedMultiResultVec::new();
            for tx in first_batch.iter() {
                result_vec.push(tx.into_multiresult());
            }

            return OptionalResult::Some((first_batch_id, result_vec).into());
        }

        OptionalResult::None
    }

    #[view(getFirstBatchAnyStatus)]
    fn get_first_batch_any_status(&self) -> OptionalResult<TxBatchSplitInFields<Self::Api>> {
        let first_batch_id = self.first_batch_id().get();
        self.get_batch(first_batch_id)
    }

    #[view(getBatch)]
    fn get_batch(&self, batch_id: u64) -> OptionalResult<TxBatchSplitInFields<Self::Api>> {
        let tx_batch = self.pending_batches(batch_id).get();
        if tx_batch.is_empty() {
            return OptionalResult::None;
        }

        let mut result_vec = ManagedMultiResultVec::new();
        for tx in tx_batch.iter() {
            result_vec.push(tx.into_multiresult());
        }

        return OptionalResult::Some((batch_id, result_vec).into());
    }

    // private

    fn add_to_batch(&self, transaction: Transaction<Self::Api>) {
        let last_batch_id = self.last_batch_id().get();
        let mut last_batch = self.pending_batches(last_batch_id).get();

        if self.is_batch_full(&last_batch) {
            self.create_new_batch(transaction);
        } else {
            last_batch.push(transaction);
            self.pending_batches(last_batch_id).set(&last_batch);
        }
    }

    // optimized to prevent reading/storing the batch over and over
    fn add_multiple_tx_to_batch(&self, transactions: ManagedVec<Transaction<Self::Api>>) {
        let mut last_batch_id = self.last_batch_id().get();
        let mut last_batch = self.pending_batches(last_batch_id).get();

        for tx in &transactions {
            if self.is_batch_full(&last_batch) {
                self.pending_batches(last_batch_id).set(&last_batch);

                last_batch.overwrite_with_single_item(tx.clone());

                self.create_new_batch(tx);
                last_batch_id += 1;
            } else {
                last_batch.push(tx);
            }
        }

        self.pending_batches(last_batch_id).set(&last_batch);
    }

    #[allow(clippy::vec_init_then_push)]
    fn create_new_batch(&self, transaction: Transaction<Self::Api>) {
        let last_batch_id = self.last_batch_id().get();
        let new_batch_id = last_batch_id + 1;

        let mut new_batch = ManagedVec::new();
        new_batch.push(transaction);

        self.pending_batches(new_batch_id).set(&new_batch);
        self.last_batch_id().set(&new_batch_id);
    }

    fn is_batch_full(&self, tx_batch: &ManagedVec<Transaction<Self::Api>>) -> bool {
        if tx_batch.is_empty() {
            return false;
        }

        let max_batch_size = self.max_tx_batch_size().get();
        if tx_batch.len() == max_batch_size {
            return true;
        }

        let current_block_nonce = self.blockchain().get_block_nonce();
        let first_tx_in_batch_block_nonce = match tx_batch.get(0) {
            Some(tx) => tx.block_nonce,
            None => return false,
        };

        // reorg protection
        if current_block_nonce < first_tx_in_batch_block_nonce {
            return false;
        }

        let block_diff = current_block_nonce - first_tx_in_batch_block_nonce;
        let max_tx_batch_block_duration = self.max_tx_batch_block_duration().get();

        block_diff > max_tx_batch_block_duration
    }

    fn is_batch_final(&self, tx_batch: &ManagedVec<Transaction<Self::Api>>) -> bool {
        let batch_len = tx_batch.len();
        let last_tx_in_batch = match tx_batch.get(batch_len - 1) {
            Some(tx) => tx,
            None => return false,
        };

        let current_block = self.blockchain().get_block_nonce();

        // reorg protection
        if current_block < last_tx_in_batch.block_nonce {
            return false;
        }

        let block_diff = current_block - last_tx_in_batch.block_nonce;

        block_diff > MIN_BLOCKS_FOR_FINALITY
    }

    fn clear_first_batch(&self) {
        let first_batch_id = self.first_batch_id().get();
        let new_first_batch_id = first_batch_id + 1;

        // for the case when the last existing batch was processed
        // otherwise, we'd create a batch with the same ID again
        self.last_batch_id().update(|last_batch_id| {
            if *last_batch_id == first_batch_id {
                *last_batch_id = new_first_batch_id;
            }
        });
        self.first_batch_id().set(&new_first_batch_id);
        self.pending_batches(first_batch_id).clear();
    }

    fn get_and_save_next_tx_id(&self) -> u64 {
        self.last_tx_nonce().update(|last_tx_nonce| {
            *last_tx_nonce += 1;
            *last_tx_nonce
        })
    }

    // storage

    #[view(getFirstBatchId)]
    #[storage_mapper("firstBatchId")]
    fn first_batch_id(&self) -> SingleValueMapper<u64>;

    #[view(getLastBatchId)]
    #[storage_mapper("lastBatchId")]
    fn last_batch_id(&self) -> SingleValueMapper<u64>;

    #[storage_mapper("pendingBatches")]
    fn pending_batches(
        &self,
        batch_id: u64,
    ) -> SingleValueMapper<ManagedVec<Transaction<Self::Api>>>;

    #[storage_mapper("lastTxNonce")]
    fn last_tx_nonce(&self) -> SingleValueMapper<u64>;

    // configurable

    #[storage_mapper("maxTxBatchSize")]
    fn max_tx_batch_size(&self) -> SingleValueMapper<usize>;

    #[storage_mapper("maxTxBatchBlockDuration")]
    fn max_tx_batch_block_duration(&self) -> SingleValueMapper<u64>;
}
