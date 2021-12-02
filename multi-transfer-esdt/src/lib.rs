#![no_std]

elrond_wasm::imports!();

use fee_estimator_module::GWEI_STRING;
use transaction::{
    esdt_safe_batch::TxBatchSplitInFields, managed_address_to_managed_buffer, EthTransaction,
    Transaction,
};

const DEFAULT_MAX_TX_BATCH_SIZE: usize = 10;
const DEFAULT_MAX_TX_BATCH_BLOCK_DURATION: u64 = 3_600; // ~6 hours

#[elrond_wasm::contract]
pub trait MultiTransferEsdt:
    fee_estimator_module::FeeEstimatorModule
    + token_module::TokenModule
    + tx_batch_module::TxBatchModule
{
    #[init]
    fn init(
        &self,
        fee_estimator_contract_address: ManagedAddress,
        eth_tx_gas_limit: BigUint,
    ) -> SCResult<()> {
        self.fee_estimator_contract_address()
            .set(&fee_estimator_contract_address);
        self.eth_tx_gas_limit().set(&eth_tx_gas_limit);

        self.max_tx_batch_size()
            .set_if_empty(&DEFAULT_MAX_TX_BATCH_SIZE);
        self.max_tx_batch_block_duration()
            .set_if_empty(&DEFAULT_MAX_TX_BATCH_BLOCK_DURATION);

        // batch ID 0 is considered invalid
        self.first_batch_id().set_if_empty(&1);
        self.last_batch_id().set_if_empty(&1);

        // set ticker for "GWEI"
        let gwei_token_id = TokenIdentifier::from(GWEI_STRING);
        self.token_ticker(&gwei_token_id)
            .set(&gwei_token_id.as_managed_buffer());

        Ok(())
    }

    #[only_owner]
    #[endpoint(batchTransferEsdtToken)]
    fn batch_transfer_esdt_token(
        &self,
        #[var_args] transfers: ManagedVarArgs<EthTransaction<Self::Api>>,
    ) {
        let mut cached_token_ids = ManagedVec::new();
        let mut cached_prices = ManagedVec::new();

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

            let required_fee = match cached_token_ids.iter().position(|id| id == eth_tx.token_id) {
                Some(index) => cached_prices.get(index).unwrap_or_else(|| BigUint::zero()),
                None => {
                    let queried_fee = self.calculate_required_fee(&eth_tx.token_id);
                    cached_token_ids.push(eth_tx.token_id.clone());
                    cached_prices.push(queried_fee.clone());

                    queried_fee
                }
            };

            if eth_tx.amount <= required_fee {
                self.add_refund_tx_to_batch(eth_tx);
                continue;
            }

            let amount_to_send = &eth_tx.amount - &required_fee;

            self.accumulated_transaction_fees(&eth_tx.token_id)
                .update(|fees| *fees += required_fee);

            self.send()
                .esdt_local_mint(&eth_tx.token_id, 0, &eth_tx.amount);
            self.send()
                .direct(&eth_tx.to, &eth_tx.token_id, 0, &amount_to_send, &[]);
        }
    }

    #[only_owner]
    #[endpoint(getAndClearFirstRefundBatch)]
    fn get_and_clear_first_refund_batch(&self) -> OptionalResult<TxBatchSplitInFields<Self::Api>> {
        let opt_current_batch = self.get_current_tx_batch();

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
            to: managed_address_to_managed_buffer(&eth_tx.to),
            token_identifier: eth_tx.token_id,
            amount: eth_tx.amount,
            is_refund_tx: true,
        };

        self.add_to_batch(tx);
    }
}
