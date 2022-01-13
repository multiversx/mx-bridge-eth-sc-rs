#![no_std]
#![allow(non_snake_case)]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use core::convert::TryFrom;

use eth_address::*;
use fee_estimator_module::GWEI_STRING;
use transaction::{transaction_status::TransactionStatus, Transaction};

const DEFAULT_MAX_TX_BATCH_SIZE: usize = 10;
const DEFAULT_MAX_TX_BATCH_BLOCK_DURATION: u64 = 100; // ~10 minutes

#[elrond_wasm::contract]
pub trait EsdtSafe:
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
    #[endpoint(setTransactionBatchStatus)]
    fn set_transaction_batch_status(
        &self,
        batch_id: u64,
        #[var_args] tx_statuses: ManagedVarArgs<TransactionStatus>,
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

        for (tx, tx_status) in tx_batch.iter().zip(tx_statuses.to_vec().iter()) {
            // Since tokens don't exist in the EsdtSafe in the case of a refund transaction
            // we have no tokens to burn, nor to refund
            if tx.is_refund_tx {
                continue;
            }

            match tx_status {
                TransactionStatus::Executed => {
                    // local burn role might be removed while tx is executed
                    // tokens will remain locked forever in that case
                    // otherwise, the whole batch would fail
                    if self.is_local_role_set(&tx.token_identifier, &EsdtLocalRole::Burn) {
                        self.burn_esdt_token(&tx.token_identifier, &tx.amount);
                    }
                }
                TransactionStatus::Rejected => {
                    self.mark_refund(
                        &ManagedAddress::try_from(tx.from)?,
                        &tx.token_identifier,
                        &tx.amount,
                    );
                }
                _ => {
                    return sc_error!("Transaction status may only be set to Executed or Rejected")
                }
            }

            self.set_status_event(batch_id, tx.nonce, tx_status);
        }

        self.clear_first_batch();

        Ok(())
    }

    #[only_owner]
    #[endpoint(addRefundBatch)]
    fn add_refund_batch(&self, refund_transactions: ManagedVec<Transaction<Self::Api>>) {
        let block_nonce = self.blockchain().get_block_nonce();
        let mut cached_token_ids = ManagedVec::<Self::Api, TokenIdentifier>::new();
        let mut cached_prices = ManagedVec::<Self::Api, BigUint>::new();
        let mut new_transactions = ManagedVec::new();

        for refund_tx in &refund_transactions {
            let required_fee = match cached_token_ids
                .iter()
                .position(|id| id == refund_tx.token_identifier)
            {
                Some(index) => cached_prices.get(index).unwrap_or_else(|| BigUint::zero()),
                None => {
                    let queried_fee = self.calculate_required_fee(&refund_tx.token_identifier);
                    cached_token_ids.push(refund_tx.token_identifier.clone());
                    cached_prices.push(queried_fee.clone());

                    queried_fee
                }
            };

            if refund_tx.amount <= required_fee {
                continue;
            }

            self.accumulated_transaction_fees(&refund_tx.token_identifier)
                .update(|fees| *fees += &required_fee);

            let actual_bridged_amount = refund_tx.amount - required_fee;
            let tx_nonce = self.get_and_save_next_tx_id();

            // "from" and "to" are inverted, since this was initially an Ethereum -> Elrond tx
            let new_tx = Transaction {
                block_nonce,
                nonce: tx_nonce,
                from: refund_tx.to,
                to: refund_tx.from,
                token_identifier: refund_tx.token_identifier,
                amount: actual_bridged_amount,
                is_refund_tx: true,
            };
            new_transactions.push(new_tx);
        }

        let batch_ids = self.add_multiple_tx_to_batch(&new_transactions);
        for (batch_id, tx) in batch_ids.iter().zip(new_transactions.iter()) {
            self.add_refund_transaction_event(batch_id, tx.nonce);
        }
    }

    // endpoints

    #[payable("*")]
    #[endpoint(createTransaction)]
    fn create_transaction(
        &self,
        #[payment_token] payment_token: TokenIdentifier,
        #[payment_amount] payment_amount: BigUint,
        to: EthAddress<Self::Api>,
    ) -> SCResult<()> {
        require!(
            self.call_value().esdt_token_nonce() == 0,
            "Only fungible ESDT tokens accepted"
        );
        self.require_token_in_whitelist(&payment_token)?;

        let required_fee = self.calculate_required_fee(&payment_token);
        require!(
            required_fee < payment_amount,
            "Transaction fees cost more than the entire bridged amount"
        );

        self.accumulated_transaction_fees(&payment_token)
            .update(|fees| *fees += &required_fee);

        let actual_bridged_amount = payment_amount - required_fee;
        let caller = self.blockchain().get_caller();
        let tx_nonce = self.get_and_save_next_tx_id();
        let tx = Transaction {
            block_nonce: self.blockchain().get_block_nonce(),
            nonce: tx_nonce,
            from: caller.as_managed_buffer().clone(),
            to: to.as_managed_buffer().clone(),
            token_identifier: payment_token,
            amount: actual_bridged_amount,
            is_refund_tx: false,
        };

        let batch_id = self.add_to_batch(tx);
        self.create_transaction_event(batch_id, tx_nonce);

        Ok(())
    }

    #[endpoint(claimRefund)]
    fn claim_refund(&self, token_id: TokenIdentifier) -> SCResult<()> {
        let caller = self.blockchain().get_caller();
        let refund_amount = self.refund_amount(&caller, &token_id).get();
        require!(refund_amount > 0, "Nothing to refund");

        self.refund_amount(&caller, &token_id).clear();
        self.send()
            .direct(&caller, &token_id, 0, &refund_amount, b"refund");

        Ok(())
    }

    #[view(getRefundAmounts)]
    fn get_refund_amounts(
        &self,
        address: ManagedAddress,
    ) -> ManagedMultiResultVec<MultiResult2<TokenIdentifier, BigUint>> {
        let mut refund_amounts = ManagedMultiResultVec::new();
        for token_id in self.token_whitelist().iter() {
            let amount = self.refund_amount(&address, &token_id).get();
            if amount > 0u32 {
                refund_amounts.push((token_id, amount).into());
            }
        }

        refund_amounts
    }

    // private

    fn burn_esdt_token(&self, token_id: &TokenIdentifier, amount: &BigUint) {
        self.send().esdt_local_burn(token_id, 0, amount);
    }

    fn mark_refund(&self, to: &ManagedAddress, token_id: &TokenIdentifier, amount: &BigUint) {
        self.refund_amount(to, token_id)
            .update(|refund| *refund += amount);
    }

    fn data_or_empty(&self, to: &ManagedAddress, data: &'static [u8]) -> &[u8] {
        if self.blockchain().is_smart_contract(to) {
            &[]
        } else {
            data
        }
    }

    // events

    #[event("createTransactionEvent")]
    fn create_transaction_event(&self, #[indexed] batch_id: u64, #[indexed] tx_id: u64);

    #[event("addRefundTransactionEvent")]
    fn add_refund_transaction_event(&self, #[indexed] batch_id: u64, #[indexed] tx_id: u64);

    #[event("setStatusEvent")]
    fn set_status_event(
        &self,
        #[indexed] batch_id: u64,
        #[indexed] tx_id: u64,
        #[indexed] tx_status: TransactionStatus,
    );

    // storage

    #[storage_mapper("refundAmount")]
    fn refund_amount(
        &self,
        address: &ManagedAddress,
        token_id: &TokenIdentifier,
    ) -> SingleValueMapper<BigUint>;
}
