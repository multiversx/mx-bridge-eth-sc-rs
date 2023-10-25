#![no_std]

multiversx_sc::imports!();

use transaction::{
    EthTransaction, EthTransactionPayment, PaymentsVec, Transaction, TxBatchSplitInFields,
};

const DEFAULT_MAX_TX_BATCH_SIZE: usize = 10;
const DEFAULT_MAX_TX_BATCH_BLOCK_DURATION: u64 = u64::MAX;
const MIN_GAS_LIMIT_FOR_SC_CALL: u64 = 10_000_000;

#[multiversx_sc::contract]
pub trait MultiTransferEsdt:
    tx_batch_module::TxBatchModule + max_bridged_amount_module::MaxBridgedAmountModule
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

    #[only_owner]
    #[endpoint(batchTransferEsdtToken)]
    fn batch_transfer_esdt_token(
        &self,
        batch_id: u64,
        transfers: MultiValueEncoded<EthTransaction<Self::Api>>,
    ) {
        let mut valid_payments_list = ManagedVec::new();
        let mut valid_dest_addresses_list = ManagedVec::new();
        let mut refund_tx_list = ManagedVec::new();

        let own_sc_address = self.blockchain().get_sc_address();
        let sc_shard = self.blockchain().get_shard_of_address(&own_sc_address);

        for eth_tx in transfers {
            let mut must_refund = false;
            if eth_tx.to.is_zero() {
                self.transfer_failed_invalid_destination(batch_id, eth_tx.tx_nonce);
                must_refund = true;
            } else if !self.is_local_role_set(&eth_tx.token_id, &EsdtLocalRole::Mint) {
                self.transfer_failed_invalid_token(batch_id, eth_tx.tx_nonce);
                must_refund = true;
            } else if self.is_above_max_amount(&eth_tx.token_id, &eth_tx.amount) {
                self.transfer_over_max_amount(batch_id, eth_tx.tx_nonce);
                must_refund = true;
            } else if self.is_account_same_shard_frozen(sc_shard, &eth_tx.to, &eth_tx.token_id) {
                self.transfer_failed_frozen_destination_account(batch_id, eth_tx.tx_nonce);
                must_refund = true;
            } else if self.blockchain().is_smart_contract(&eth_tx.to)
                && (eth_tx.data.is_empty() || eth_tx.gas_limit < MIN_GAS_LIMIT_FOR_SC_CALL)
            {
                must_refund = true;
            }

            if must_refund {
                let refund_tx = self.convert_to_refund_tx(eth_tx);
                refund_tx_list.push(refund_tx);

                continue;
            }

            let _: IgnoreValue = self
                .get_esdt_safe_contract_proxy_instance()
                .get_token(&eth_tx.token_id, &eth_tx.amount)
                .with_esdt_transfer((eth_tx.token_id.clone(), 0, eth_tx.amount.clone()))
                .execute_on_dest_context();

            // emit event before the actual transfer so we don't have to save the tx_nonces as well
            self.transfer_performed_event(batch_id, eth_tx.tx_nonce);

            if self.blockchain().is_smart_contract(&eth_tx.to.clone()) {
                let _: IgnoreValue = self
                    .get_bridge_proxy_contract_proxy_instance()
                    .deposit(&eth_tx)
                    .with_esdt_transfer((eth_tx.token_id.clone(), 0, eth_tx.amount.clone()))
                    .execute_on_dest_context();
            } else {
                valid_dest_addresses_list.push(eth_tx.to);
                valid_payments_list.push(EsdtTokenPayment::new(eth_tx.token_id, 0, eth_tx.amount));
            }
        }

        let payments_after_wrapping = self.wrap_tokens(valid_payments_list);
        self.distribute_payments(valid_dest_addresses_list, payments_after_wrapping);

        self.add_multiple_tx_to_batch(&refund_tx_list);
    }

    #[only_owner]
    #[endpoint(getAndClearFirstRefundBatch)]
    fn get_and_clear_first_refund_batch(&self) -> OptionalValue<TxBatchSplitInFields<Self::Api>> {
        let opt_current_batch = self.get_first_batch_any_status();
        if matches!(opt_current_batch, OptionalValue::Some(_)) {
            let first_batch_id = self.first_batch_id().get();
            let mut first_batch = self.pending_batches(first_batch_id);

            self.clear_first_batch(&mut first_batch);
        }

        opt_current_batch
    }

    #[only_owner]
    #[endpoint(setWrappingContractAddress)]
    fn set_wrapping_contract_address(&self, opt_new_address: OptionalValue<ManagedAddress>) {
        match opt_new_address {
            OptionalValue::Some(sc_addr) => {
                require!(
                    self.blockchain().is_smart_contract(&sc_addr),
                    "Invalid unwrapping contract address"
                );

                self.wrapping_contract_address().set(&sc_addr);
            }
            OptionalValue::None => self.wrapping_contract_address().clear(),
        }
    }

    #[only_owner]
    #[endpoint(setBridgeProxyContractAddress)]
    fn set_bridge_proxy_contract_address(&self, opt_new_address: OptionalValue<ManagedAddress>) {
        match opt_new_address {
            OptionalValue::Some(sc_addr) => {
                require!(
                    self.blockchain().is_smart_contract(&sc_addr),
                    "Invalid bridge proxy contract address"
                );

                self.bridge_proxy_contract_address().set(&sc_addr);
            }
            OptionalValue::None => self.bridge_proxy_contract_address().clear(),
        }
    }

    #[only_owner]
    #[endpoint(setEsdtSafeContractAddress)]
    fn set_esdt_safe_contract_address(&self, opt_new_address: OptionalValue<ManagedAddress>) {
        match opt_new_address {
            OptionalValue::Some(sc_addr) => {
                require!(
                    self.blockchain().is_smart_contract(&sc_addr),
                    "Invalid esdt safe contract address"
                );

                self.esdt_safe_contract_address().set(&sc_addr);
            }
            OptionalValue::None => self.esdt_safe_contract_address().clear(),
        }
    }

    #[endpoint(getFailedTxFromBridgeProxy)]
    fn get_failed_tx_from_bridge_proxy(&self) {
        let mut refund_tx_list = ManagedVec::new();

        let bridge_proxy_addr = self.bridge_proxy_contract_address().get();
        let failed_txs: MultiValueEncoded<EthTransactionPayment<Self::Api>> = self
            .bridge_proxy(bridge_proxy_addr)
            .refund_transactions()
            .execute_on_dest_context();

        for failed_tx in failed_txs {
            let refund_tx = self.convert_to_refund_tx(failed_tx.eth_tx);
            refund_tx_list.push(refund_tx);
        }

        self.add_multiple_tx_to_batch(&refund_tx_list);
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
        if self.wrapping_contract_address().is_empty() {
            return payments;
        }

        self.get_wrapping_contract_proxy_instance()
            .wrap_tokens()
            .with_multi_token_transfer(payments)
            .execute_on_dest_context()
    }

    fn distribute_payments(
        &self,
        dest_addresses: ManagedVec<ManagedAddress>,
        payments: PaymentsVec<Self::Api>,
    ) {
        for (dest, p) in dest_addresses.iter().zip(payments.iter()) {
            self.send()
                .direct_esdt(&dest, &p.token_identifier, 0, &p.amount);
        }
    }

    // proxies

    #[proxy]
    fn wrapping_contract_proxy(
        &self,
        sc_address: ManagedAddress,
    ) -> bridged_tokens_wrapper::Proxy<Self::Api>;

    fn get_wrapping_contract_proxy_instance(&self) -> bridged_tokens_wrapper::Proxy<Self::Api> {
        self.wrapping_contract_proxy(self.wrapping_contract_address().get())
    }

    #[proxy]
    fn bridge_proxy(&self, sc_address: ManagedAddress) -> bridge_proxy::Proxy<Self::Api>;

    fn get_bridge_proxy_contract_proxy_instance(&self) -> bridge_proxy::Proxy<Self::Api> {
        self.bridge_proxy(self.bridge_proxy_contract_address().get())
    }

    #[proxy]
    fn esdt_safe(&self, sc_address: ManagedAddress) -> esdt_safe::Proxy<Self::Api>;

    fn get_esdt_safe_contract_proxy_instance(&self) -> esdt_safe::Proxy<Self::Api> {
        self.esdt_safe(self.esdt_safe_contract_address().get())
    }

    // storage
    #[view(getWrappingContractAddress)]
    #[storage_mapper("wrappingContractAddress")]
    fn wrapping_contract_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getBridgeProxyContractAddress)]
    #[storage_mapper("bridgeProxyContractAddress")]
    fn bridge_proxy_contract_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getEsdtSafeContractAddress)]
    #[storage_mapper("esdtSafeContractAddress")]
    fn esdt_safe_contract_address(&self) -> SingleValueMapper<ManagedAddress>;

    // events

    #[event("transferPerformedEvent")]
    fn transfer_performed_event(&self, #[indexed] batch_id: u64, #[indexed] tx_id: u64);

    #[event("transferFailedInvalidDestination")]
    fn transfer_failed_invalid_destination(&self, #[indexed] batch_id: u64, #[indexed] tx_id: u64);

    #[event("transferFailedInvalidToken")]
    fn transfer_failed_invalid_token(&self, #[indexed] batch_id: u64, #[indexed] tx_id: u64);

    #[event("transferFailedFrozenDestinationAccount")]
    fn transfer_failed_frozen_destination_account(
        &self,
        #[indexed] batch_id: u64,
        #[indexed] tx_id: u64,
    );

    #[event("transferOverMaxAmount")]
    fn transfer_over_max_amount(&self, #[indexed] batch_id: u64, #[indexed] tx_id: u64);
}
