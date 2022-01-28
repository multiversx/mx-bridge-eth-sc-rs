#![no_std]

elrond_wasm::imports!();

use transaction::{esdt_safe_batch::TxBatchSplitInFields, EthTransaction, Transaction};

const DEFAULT_MAX_TX_BATCH_SIZE: usize = 10;
const DEFAULT_MAX_TX_BATCH_BLOCK_DURATION: u64 = u64::MAX;

#[elrond_wasm::contract]
pub trait MultiTransferEsdt: tx_batch_module::TxBatchModule {
    #[init]
    fn init(&self) {
        self.max_tx_batch_size()
            .set_if_empty(&DEFAULT_MAX_TX_BATCH_SIZE);
        self.max_tx_batch_block_duration()
            .set_if_empty(&DEFAULT_MAX_TX_BATCH_BLOCK_DURATION);

        // batch ID 0 is considered invalid
        self.first_batch_id().set_if_empty(&1);
        self.last_batch_id().set_if_empty(&1);
    }

    #[only_owner]
    #[endpoint(batchTransferEsdtToken)]
    fn batch_transfer_esdt_token(
        &self,
        batch_id: u64,
        #[var_args] transfers: ManagedVarArgs<EthTransaction<Self::Api>>,
    ) {
        let mut refund_tx_list = ManagedVec::new();
        for eth_tx in transfers {
            if eth_tx.to.is_zero() || self.blockchain().is_smart_contract(&eth_tx.to) {
                self.transfer_failed_invalid_destination(batch_id, eth_tx.tx_nonce);

                let refund_tx = self.convert_to_refund_tx(eth_tx);
                refund_tx_list.push(refund_tx);

                continue;
            }
            if !self.is_local_role_set(&eth_tx.token_id, &EsdtLocalRole::Mint) {
                self.transfer_failed_invalid_token(batch_id, eth_tx.tx_nonce);

                let refund_tx = self.convert_to_refund_tx(eth_tx);
                refund_tx_list.push(refund_tx);

                continue;
            }

            self.send()
                .esdt_local_mint(&eth_tx.token_id, 0, &eth_tx.amount);
            self.send()
                .direct(&eth_tx.to, &eth_tx.token_id, 0, &eth_tx.amount, &[]);

            self.transfer_performed_event(batch_id, eth_tx.tx_nonce);
        }

        self.add_multiple_tx_to_batch(&refund_tx_list);
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

    fn convert_to_refund_tx(&self, eth_tx: EthTransaction<Self::Api>) -> Transaction<Self::Api> {
        Transaction {
            block_nonce: self.blockchain().get_block_nonce(),
            nonce: eth_tx.tx_nonce,
            from: eth_tx.from.as_managed_buffer().clone(),
            to: eth_tx.to.as_managed_buffer().clone(),
            token_identifier: eth_tx.token_id,
            amount: eth_tx.amount,
            is_refund_tx: true,
        }
    }

    fn is_local_role_set(&self, token_id: &TokenIdentifier, role: &EsdtLocalRole) -> bool {
        let roles = self.blockchain().get_esdt_local_roles(token_id);

        roles.has_role(role)
    }

    // events

    #[event("transferPerformedEvent")]
    fn transfer_performed_event(&self, #[indexed] batch_id: u64, #[indexed] tx_id: u64);

    #[event("transferFailedInvalidDestination")]
    fn transfer_failed_invalid_destination(&self, #[indexed] batch_id: u64, #[indexed] tx_id: u64);

    #[event("transferFailedInvalidToken")]
    fn transfer_failed_invalid_token(&self, #[indexed] batch_id: u64, #[indexed] tx_id: u64);
}
