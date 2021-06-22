#![no_std]
#![allow(non_snake_case)]

use eth_address::*;
use transaction::*;

elrond_wasm::imports!();

const DEFAULT_MAX_TX_BATCH_SIZE: usize = 10;
const DEFAULT_MAX_BLOCK_NONCE_DIFF: u64 = 100;

#[elrond_wasm_derive::contract]
pub trait EsdtSafe {
    #[init]
    fn init(
        &self,
        fee_estimator_contract_address: Address,
        #[var_args] token_whitelist: VarArgs<TokenIdentifier>,
    ) -> SCResult<()> {
        self.fee_estimator_contract_address()
            .set(&fee_estimator_contract_address);

        for token in token_whitelist.into_vec() {
            require!(token.is_valid_esdt_identifier(), "Invalid token ID");
            self.token_whitelist().insert(token.clone());
        }

        self.max_tx_batch_size().set(&DEFAULT_MAX_TX_BATCH_SIZE);
        self.max_block_nonce_diff()
            .set(&DEFAULT_MAX_BLOCK_NONCE_DIFF);

        Ok(())
    }

    // endpoints - owner-only
    // the owner will probably be a multisig SC

    #[endpoint(addTokenToWhitelist)]
    fn add_token_to_whitelist(&self, token_id: TokenIdentifier) -> SCResult<()> {
        self.require_caller_owner()?;
        self.require_local_burn_role_set(&token_id)?;

        self.token_whitelist().insert(token_id);

        Ok(())
    }

    #[endpoint(removeTokenFromWhitelist)]
    fn remove_token_from_whitelist(&self, token_id: TokenIdentifier) -> SCResult<()> {
        self.require_caller_owner()?;

        self.token_whitelist().remove(&token_id);

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

    #[endpoint(setMaxBlockNonceDiff)]
    fn set_max_block_nonce_diff(&self, new_max_block_nonce_diff: u64) -> SCResult<()> {
        self.require_caller_owner()?;
        require!(
            new_max_block_nonce_diff > 0,
            "Max block nonce diff must be more than 0"
        );

        self.max_block_nonce_diff().set(&new_max_block_nonce_diff);

        Ok(())
    }

    #[endpoint(getNextTransactionBatch)]
    fn get_next_transaction_batch(&self) -> SCResult<MultiResultVec<Transaction<Self::BigUint>>> {
        only_owner!(self, "only owner may call this function");

        let mut tx_batch = Vec::new();
        let mut first_tx_block_nonce = 0;
        let max_tx_batch_size = self.max_tx_batch_size().get();
        let max_block_nonce_diff = self.max_block_nonce_diff().get();

        while let Some(tx) = self.get_next_pending_transaction() {
            if tx_batch.is_empty() {
                first_tx_block_nonce = tx.block_nonce;
            }

            let block_nonce_diff = tx.block_nonce - first_tx_block_nonce;
            if block_nonce_diff > max_block_nonce_diff {
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

        Ok(tx_batch.into())
    }

    #[endpoint(setTransactionBatchStatus)]
    fn set_transaction_batch_status(
        &self,
        relayer_reward_address: Address,
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
                    self.transaction_status(&sender, nonce)
                        .set(&TransactionStatus::Executed);

                    let tx = self.transactions_by_nonce(&sender).get(nonce);

                    self.require_local_burn_role_set(&tx.token_identifier)?;
                    self.burn_esdt_token(&tx.token_identifier, &tx.amount);
                }
                TransactionStatus::Rejected => {
                    self.transaction_status(&sender, nonce)
                        .set(&TransactionStatus::Rejected);

                    let tx = self.transactions_by_nonce(&sender).get(nonce);

                    self.refund_esdt_token(&tx.from, &tx.token_identifier, &tx.amount);
                }
                _ => {
                    return sc_error!("Transaction status may only be set to Executed or Rejected")
                }
            }

            // pay tx fees to the relayer that processed the transaction
            self.pay_fee(sender, relayer_reward_address.clone());
        }

        Ok(())
    }

    // endpoints

    #[payable("*")]
    #[endpoint(createTransaction)]
    fn create_transaction(
        &self,
        #[payment_token] payment_token: TokenIdentifier,
        #[payment] payment: Self::BigUint,
        to: EthAddress,
    ) -> SCResult<()> {
        require!(
            self.call_value().esdt_token_nonce() == 0,
            "Only fungible ESDT tokens accepted"
        );
        require!(
            self.token_whitelist().contains(&payment_token),
            "Payment token is not on whitelist. Transaction rejected"
        );
        require!(payment > 0, "Must transfer more than 0");
        require!(!to.is_zero(), "Can't transfer to address zero");

        let caller = self.blockchain().get_caller();
        let sender_nonce = self.transactions_by_nonce(&caller).len() + 1;
        let tx = Transaction {
            block_nonce: self.blockchain().get_block_nonce(),
            nonce: sender_nonce,
            from: caller.clone(),
            to,
            token_identifier: payment_token,
            amount: payment,
        };

        self.transactions_by_nonce(&caller).push(&tx);

        self.transaction_status(&caller, sender_nonce)
            .set(&TransactionStatus::Pending);
        self.pending_transaction_address_nonce_list()
            .push_back((caller.clone(), sender_nonce));

        // reserve transaction fee beforehand
        // used prevent transaction spam
        self.reserve_fee(caller);

        Ok(())
    }

    // private

    fn burn_esdt_token(&self, token_id: &TokenIdentifier, amount: &Self::BigUint) {
        self.send().esdt_local_burn(token_id, amount);
    }

    fn refund_esdt_token(&self, to: &Address, token_id: &TokenIdentifier, amount: &Self::BigUint) {
        let _ = self
            .send()
            .direct(to, token_id, amount, self.data_or_empty(to, b"refund"));
    }

    fn data_or_empty(&self, to: &Address, data: &'static [u8]) -> &[u8] {
        if self.blockchain().is_smart_contract(to) {
            &[]
        } else {
            data
        }
    }

    fn reserve_fee(&self, _from: Address) {
        /* TODO: Uncomment once the integration is complete
        self.ethereum_fee_prepay_proxy(self.fee_estimator_contract_address().get())
            .reserve_fee(from, TransactionType::Erc20, Priority::Low)
            .execute_on_dest_context();
        */
    }

    fn pay_fee(&self, _tx_sender: Address, _relayer_reward_address: Address) {
        /* TODO: Uncomment once the integration is complete
        self.ethereum_fee_prepay_proxy(self.fee_estimator_contract_address().get())
            .pay_fee(
                tx_sender,
                relayer_reward_address,
                TransactionType::Erc20,
                Priority::Low,
            )
            .execute_on_dest_context();
        */
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

    // proxies

    #[proxy]
    fn ethereum_fee_prepay_proxy(
        &self,
        sc_address: Address,
    ) -> ethereum_fee_prepay::Proxy<Self::SendApi>;

    // storage

    // the FeeEstimator SC is an aggregator that will query the Oracles and provide an average
    // used to estimate the cost of an Ethereum tx at any given point in time

    #[view(getFeeEstimatorContractAddress)]
    #[storage_mapper("feeEstimatorContractAddress")]
    fn fee_estimator_contract_address(&self) -> SingleValueMapper<Self::Storage, Address>;

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

    // configurable

    #[storage_mapper("maxTxBatchSize")]
    fn max_tx_batch_size(&self) -> SingleValueMapper<Self::Storage, usize>;

    #[storage_mapper("maxBlockNonceDiff")]
    fn max_block_nonce_diff(&self) -> SingleValueMapper<Self::Storage, u64>;
}
