#![no_std]

elrond_wasm::imports!();

use transaction::{esdt_safe_batch::TxBatchSplitInFields, EthTransaction, Transaction};

pub mod token_whitelist_module;

const DEFAULT_MAX_TX_BATCH_SIZE: usize = 10;
const DEFAULT_MAX_TX_BATCH_BLOCK_DURATION: u64 = u64::MAX;

#[elrond_wasm::contract]
pub trait MultiTransferEsdt:
    token_whitelist_module::TokenWhitelistModule + tx_batch_module::TxBatchModule
{
    #[init]
    fn init(&self) -> SCResult<()> {
        self.max_tx_batch_size()
            .set_if_empty(&DEFAULT_MAX_TX_BATCH_SIZE);
        self.max_tx_batch_block_duration()
            .set_if_empty(&DEFAULT_MAX_TX_BATCH_BLOCK_DURATION);

        // batch ID 0 is considered invalid
        self.first_batch_id().set_if_empty(&1);
        self.last_batch_id().set_if_empty(&1);

        Ok(())
    }

    #[only_owner]
    #[endpoint(batchTransferEsdtToken)]
    fn batch_transfer_esdt_token(
        &self,
        #[var_args] transfers: ManagedVarArgs<EthTransaction<Self::Api>>,
    ) {
        for eth_tx in transfers {
            if eth_tx.to.is_zero() || self.blockchain().is_smart_contract(&eth_tx.to) {
                self.add_refund_tx_to_batch(eth_tx);
                continue;
            }
            if !self.token_whitelist().contains(&eth_tx.token_id)
                || !self.is_local_role_set(&eth_tx.token_id, &EsdtLocalRole::Mint)
            {
                self.add_refund_tx_to_batch(eth_tx);
                continue;
            }

            self.send()
                .esdt_local_mint(&eth_tx.token_id, 0, &eth_tx.amount);
            self.send()
                .direct(&eth_tx.to, &eth_tx.token_id, 0, &eth_tx.amount, &[]);
        }
    }

    #[only_owner]
    #[endpoint(getAndClearFirstRefundBatch)]
    fn get_and_clear_first_refund_batch(&self) -> OptionalResult<TxBatchSplitInFields<Self::Api>> {
        let opt_current_batch = self.get_first_batch_any_status();
        if matches!(opt_current_batch, OptionalResult::Some(_)) {
            self.clear_first_batch();
        }

        opt_current_batch
    }

    // private

    fn add_refund_tx_to_batch(&self, eth_tx: EthTransaction<Self::Api>) {
        let tx = Transaction {
            block_nonce: self.blockchain().get_block_nonce(),
            nonce: eth_tx.tx_nonce,
            from: eth_tx.from.as_managed_buffer().clone(),
            to: eth_tx.to.as_managed_buffer().clone(),
            token_identifier: eth_tx.token_id,
            amount: eth_tx.amount,
            is_refund_tx: true,
        };

        self.add_to_batch(tx);
    }
}
