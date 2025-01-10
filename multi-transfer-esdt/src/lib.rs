#![no_std]

use multiversx_sc::{imports::*, storage::StorageKey};

use eth_address::EthAddress;
use sc_proxies::{bridge_proxy_contract_proxy, bridged_tokens_wrapper_proxy, esdt_safe_proxy};
use transaction::{EthTransaction, PaymentsVec, Transaction, TxNonce};

const DEFAULT_MAX_TX_BATCH_SIZE: usize = 10;
const DEFAULT_MAX_TX_BATCH_BLOCK_DURATION: u64 = u64::MAX;
const CHAIN_SPECIFIC_TO_UNIVERSAL_TOKEN_MAPPING: &[u8] = b"chainSpecificToUniversalMapping";

#[multiversx_sc::contract]
pub trait MultiTransferEsdt:
    tx_batch_module::TxBatchModule
    + max_bridged_amount_module::MaxBridgedAmountModule
    + storage_module::CommonStorageModule
{
    #[init]
    fn init(&self) {
        self.max_tx_batch_size()
            .set_if_empty(DEFAULT_MAX_TX_BATCH_SIZE);
        self.max_tx_batch_block_duration()
            .set_if_empty(DEFAULT_MAX_TX_BATCH_BLOCK_DURATION);
        // batch ID 0 is considered invalid
        self.first_batch_id().set_if_empty(1);
        self.last_batch_id().set_if_empty(1);
    }

    #[upgrade]
    fn upgrade(&self) {
        self.max_tx_batch_size()
            .set_if_empty(DEFAULT_MAX_TX_BATCH_SIZE);
        self.max_tx_batch_block_duration()
            .set_if_empty(DEFAULT_MAX_TX_BATCH_BLOCK_DURATION);
        // batch ID 0 is considered invalid
        self.first_batch_id().set_if_empty(1);
        self.last_batch_id().set_if_empty(1);
    }

    #[only_owner]
    #[endpoint(batchTransferEsdtToken)]
    fn batch_transfer_esdt_token(
        &self,
        batch_id: u64,
        transfers: MultiValueEncoded<EthTransaction<Self::Api>>,
    ) {
        let mut valid_payments_list = ManagedVec::new();
        let mut valid_tx_list = ManagedVec::new();
        let mut refund_tx_list = ManagedVec::new();

        let own_sc_address = self.blockchain().get_sc_address();
        let sc_shard = self.blockchain().get_shard_of_address(&own_sc_address);

        let safe_address = self.get_esdt_safe_address();

        for eth_tx in transfers {
            let token_roles = self
                .blockchain()
                .get_esdt_local_roles(&eth_tx.token_id.clone());
            if token_roles.has_role(&EsdtLocalRole::Transfer) {
                self.add_eth_tx_to_refund_tx_list(eth_tx.clone(), &mut refund_tx_list);
                self.token_with_transfer_role_event(eth_tx.token_id);
                continue;
            }

            let is_success: bool = self
                .tx()
                .to(safe_address.clone())
                .typed(esdt_safe_proxy::EsdtSafeProxy)
                .get_tokens(&eth_tx.token_id, &eth_tx.amount)
                .returns(ReturnsResult)
                .sync_call();

            if !is_success {
                self.add_eth_tx_to_refund_tx_list(eth_tx, &mut refund_tx_list);
                continue;
            }

            let universal_token = self.get_universal_token(eth_tx.clone());

            if eth_tx.to.is_zero() {
                self.add_eth_tx_to_refund_tx_list(eth_tx.clone(), &mut refund_tx_list);
                self.transfer_failed_invalid_destination_event(batch_id, eth_tx.tx_nonce);
                continue;
            }
            if self.is_above_max_amount(&eth_tx.token_id, &eth_tx.amount) {
                self.add_eth_tx_to_refund_tx_list(eth_tx.clone(), &mut refund_tx_list);
                self.transfer_over_max_amount_event(batch_id, eth_tx.tx_nonce);
                continue;
            }
            if self.is_account_same_shard_frozen(sc_shard, &eth_tx.to, &universal_token) {
                self.add_eth_tx_to_refund_tx_list(eth_tx.clone(), &mut refund_tx_list);
                self.transfer_failed_frozen_destination_account_event(batch_id, eth_tx.tx_nonce);
                continue;
            }

            // emit event before the actual transfer so we don't have to save the tx_nonces as well
            // emit events only for non-SC destinations
            if self.blockchain().is_smart_contract(&eth_tx.to) {
                self.transfer_performed_sc_event(
                    batch_id,
                    eth_tx.from.clone(),
                    eth_tx.to.clone(),
                    eth_tx.token_id.clone(),
                    eth_tx.amount.clone(),
                    eth_tx.tx_nonce,
                );
            } else {
                self.transfer_performed_event(
                    batch_id,
                    eth_tx.from.clone(),
                    eth_tx.to.clone(),
                    eth_tx.token_id.clone(),
                    eth_tx.amount.clone(),
                    eth_tx.tx_nonce,
                );
            }

            valid_tx_list.push(eth_tx.clone());
            valid_payments_list.push(EsdtTokenPayment::new(eth_tx.token_id, 0, eth_tx.amount));
        }

        let payments_after_wrapping = self.wrap_tokens(valid_payments_list);
        self.distribute_payments(valid_tx_list, payments_after_wrapping, batch_id);

        self.add_multiple_tx_to_batch(&refund_tx_list);
    }

    #[only_owner]
    #[endpoint(moveRefundBatchToSafe)]
    fn move_refund_batch_to_safe(&self) {
        let opt_current_batch = self.get_first_batch_any_status();
        match opt_current_batch {
            OptionalValue::Some(current_batch) => {
                let first_batch_id = self.first_batch_id().get();
                let mut first_batch = self.pending_batches(first_batch_id);

                self.clear_first_batch(&mut first_batch);
                let (_batch_id, all_tx_fields) = current_batch.into_tuple();
                let mut refund_batch = ManagedVec::new();
                let mut refund_payments = ManagedVec::new();

                for tx_fields in all_tx_fields {
                    let (_, tx_nonce, _, _, token_identifier, amount) =
                        tx_fields.clone().into_tuple();

                    if self.is_refund_valid(&token_identifier) {
                        refund_batch.push(Transaction::from(tx_fields));
                        refund_payments.push(EsdtTokenPayment::new(token_identifier, 0, amount));
                    } else {
                        require!(
                            self.unprocessed_refund_txs(tx_nonce).is_empty(),
                            "This transcation is already marked as unprocessed"
                        );
                        self.unprocessed_refund_txs(tx_nonce)
                            .set(Transaction::from(tx_fields));

                        self.unprocessed_refund_txs_event(tx_nonce);
                    }
                }

                let esdt_safe_addr = self.get_esdt_safe_address();
                self.tx()
                    .to(esdt_safe_addr)
                    .typed(esdt_safe_proxy::EsdtSafeProxy)
                    .add_refund_batch(refund_batch)
                    .payment(refund_payments)
                    .sync_call();
            }
            OptionalValue::None => {}
        }
    }

    #[only_owner]
    #[endpoint(addUnprocessedRefundTxToBatch)]
    fn add_unprocessed_refund_tx_to_batch(&self, tx_id: u64) {
        let refund_tx = self.unprocessed_refund_txs(tx_id).get();
        let mut refund_tx_list = ManagedVec::new();
        refund_tx_list.push(refund_tx);
        self.add_multiple_tx_to_batch(&refund_tx_list);

        self.unprocessed_refund_txs(tx_id).clear();
    }

    // private

    fn add_eth_tx_to_refund_tx_list(
        &self,
        eth_tx: EthTransaction<Self::Api>,
        refund_tx_list: &mut ManagedVec<Transaction<Self::Api>>,
    ) {
        let refund_tx = self.convert_to_refund_tx(eth_tx);
        refund_tx_list.push(refund_tx);
    }

    fn is_refund_valid(&self, token_id: &TokenIdentifier) -> bool {
        let esdt_safe_addr = self.get_esdt_safe_address();
        let own_sc_address = self.blockchain().get_sc_address();
        let sc_shard = self.blockchain().get_shard_of_address(&own_sc_address);
        let token_roles = self.blockchain().get_esdt_local_roles(token_id);

        if self.is_account_same_shard_frozen(sc_shard, &esdt_safe_addr, token_id)
            || token_roles.has_role(&EsdtLocalRole::Transfer)
        {
            return false;
        }

        return true;
    }

    fn get_universal_token(&self, eth_tx: EthTransaction<Self::Api>) -> TokenIdentifier {
        let mut storage_key = StorageKey::new(CHAIN_SPECIFIC_TO_UNIVERSAL_TOKEN_MAPPING);
        storage_key.append_item(&eth_tx.token_id);

        let chain_specific_to_universal_token_mapper: SingleValueMapper<
            TokenIdentifier,
            ManagedAddress,
        > = SingleValueMapper::<_, _, ManagedAddress>::new_from_address(
            self.get_bridged_tokens_wrapper_address(),
            storage_key,
        );
        if chain_specific_to_universal_token_mapper.is_empty() {
            eth_tx.token_id
        } else {
            chain_specific_to_universal_token_mapper.get()
        }
    }

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

    fn is_account_same_shard_frozen(
        &self,
        sc_shard: u32,
        dest_address: &ManagedAddress,
        token_id: &TokenIdentifier,
    ) -> bool {
        let dest_shard = self.blockchain().get_shard_of_address(dest_address);
        if sc_shard != dest_shard {
            return false;
        }

        let token_data = self
            .blockchain()
            .get_esdt_token_data(dest_address, token_id, 0);
        token_data.frozen
    }

    fn wrap_tokens(&self, payments: PaymentsVec<Self::Api>) -> PaymentsVec<Self::Api> {
        if self.get_bridged_tokens_wrapper_address().is_zero() {
            return payments;
        }

        let bridged_tokens_wrapper_addr = self.get_bridged_tokens_wrapper_address();
        self.tx()
            .to(bridged_tokens_wrapper_addr)
            .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
            .wrap_tokens()
            .payment(payments)
            .returns(ReturnsResult)
            .sync_call()
    }

    fn distribute_payments(
        &self,
        transfers: ManagedVec<EthTransaction<Self::Api>>,
        payments: PaymentsVec<Self::Api>,
        batch_id: u64,
    ) {
        let bridge_proxy_addr = self.get_bridge_proxy_address();
        for (eth_tx, p) in transfers.iter().zip(payments.iter()) {
            if self.blockchain().is_smart_contract(&eth_tx.to) {
                self.tx()
                    .to(bridge_proxy_addr.clone())
                    .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
                    .deposit(&eth_tx.clone(), batch_id)
                    .single_esdt(&p.token_identifier, 0, &p.amount)
                    .sync_call();
            } else {
                self.tx()
                    .to(&eth_tx.to)
                    .single_esdt(&p.token_identifier, 0, &p.amount)
                    .callback(self.callbacks().transfer_callback(eth_tx.clone(), batch_id))
                    .gas(self.blockchain().get_gas_left())
                    // .gas_for_callback(CALLBACK_ESDT_TRANSFER_GAS_LIMIT)
                    .register_promise();
            }
        }
    }

    #[promises_callback]
    fn transfer_callback(
        &self,
        #[call_result] result: ManagedAsyncCallResult<()>,
        tx: EthTransaction<Self::Api>,
        batch_id: u64,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(()) => {
                self.transfer_performed_event(
                    batch_id,
                    tx.from,
                    tx.to,
                    tx.token_id,
                    tx.amount,
                    tx.tx_nonce,
                );
            }
            ManagedAsyncCallResult::Err(_) => {
                // TODO: Maybe fire a better event, but this is the most likely cause anyway
                self.transfer_failed_frozen_destination_account_event(batch_id, tx.tx_nonce);

                let refund_tx = self.convert_to_refund_tx(tx);
                self.add_to_batch(refund_tx);
            }
        }
    }
    // storage

    #[storage_mapper("unprocessedRefundTxs")]
    fn unprocessed_refund_txs(&self, tx_id: u64) -> SingleValueMapper<Transaction<Self::Api>>;

    // events

    #[event("transferPerformedEvent")]
    fn transfer_performed_event(
        &self,
        #[indexed] batch_id: u64,
        #[indexed] from: EthAddress<Self::Api>,
        #[indexed] to: ManagedAddress,
        #[indexed] token_id: TokenIdentifier,
        #[indexed] amount: BigUint,
        #[indexed] tx_id: TxNonce,
    );

    #[event("transferPerformedSCEvent")]
    fn transfer_performed_sc_event(
        &self,
        #[indexed] batch_id: u64,
        #[indexed] from: EthAddress<Self::Api>,
        #[indexed] to: ManagedAddress,
        #[indexed] token_id: TokenIdentifier,
        #[indexed] amount: BigUint,
        #[indexed] tx_id: TxNonce,
    );

    #[event("transferFailedInvalidDestination")]
    fn transfer_failed_invalid_destination_event(
        &self,
        #[indexed] batch_id: u64,
        #[indexed] tx_id: u64,
    );

    #[event("tokenWithTransferRole")]
    fn token_with_transfer_role_event(&self, #[indexed] token_id: TokenIdentifier);

    #[event("transferFailedInvalidToken")]
    fn transfer_failed_invalid_token_event(&self, #[indexed] batch_id: u64, #[indexed] tx_id: u64);

    #[event("transferFailedFrozenDestinationAccount")]
    fn transfer_failed_frozen_destination_account_event(
        &self,
        #[indexed] batch_id: u64,
        #[indexed] tx_id: u64,
    );

    #[event("transferOverMaxAmount")]
    fn transfer_over_max_amount_event(&self, #[indexed] batch_id: u64, #[indexed] tx_id: u64);

    #[event("unprocessedRefundTxs")]
    fn unprocessed_refund_txs_event(&self, #[indexed] tx_id: u64);
}
