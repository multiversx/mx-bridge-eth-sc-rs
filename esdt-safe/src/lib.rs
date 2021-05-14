#![no_std]
#![allow(non_snake_case)]

use eth_address::*;
use transaction::*;

elrond_wasm::imports!();

#[elrond_wasm_derive::contract]
pub trait EsdtSafe {
    #[init]
    fn init(
        &self,
        fee_estimator_contract_address: Address,
        #[var_args] token_whitelist: VarArgs<TokenIdentifier>,
    ) {
        self.fee_estimator_contract_address()
            .set(&fee_estimator_contract_address);

        for token in token_whitelist.into_vec() {
            self.token_whitelist().insert(token.clone());
        }
    }

    // endpoints - owner-only
    // the owner will probably be a multisig SC

    #[endpoint(addTokenToWhitelist)]
    fn add_token_to_whitelist(&self, token_id: TokenIdentifier) -> SCResult<()> {
        only_owner!(self, "only owner may call this function");

        self.token_whitelist().insert(token_id);

        Ok(())
    }

    #[endpoint(removeTokenFromWhitelist)]
    fn remove_token_from_whitelist(&self, token_id: TokenIdentifier) -> SCResult<()> {
        only_owner!(self, "only owner may call this function");

        self.token_whitelist().remove(&token_id);

        Ok(())
    }

    #[endpoint(getNextPendingTransaction)]
    fn get_next_pending_transaction(
        &self,
    ) -> SCResult<OptionalResult<TxAsMultiResult<Self::BigUint>>> {
        only_owner!(self, "only owner may call this function");

        match self.pending_transaction_address_nonce_list().pop_front() {
            Some((sender, nonce)) => {
                self.transaction_status(&sender, nonce)
                    .set(&TransactionStatus::InProgress);

                let tx = self.transactions_by_nonce(&sender).get(nonce);

                Ok(OptionalResult::Some(tx.into_multiresult()))
            }
            None => Ok(OptionalResult::None),
        }
    }

    #[endpoint(setTransactionStatus)]
    fn set_transaction_status(
        &self,
        sender: Address,
        nonce: TxNonce,
        transaction_status: TransactionStatus,
    ) -> SCResult<AsyncCall<Self::SendApi>> {
        only_owner!(self, "only owner may call this function");

        require!(
            self.transaction_status(&sender, nonce).get() == TransactionStatus::InProgress,
            "Transaction has to be executed first"
        );

        match transaction_status {
            TransactionStatus::Executed => {
                self.transaction_status(&sender, nonce)
                    .set(&TransactionStatus::Executed);

                let tx = self.transactions_by_nonce(&sender).get(nonce);

                Ok(self.burn_esdt_token(&tx.token_identifier, &tx.amount))
            }
            TransactionStatus::Rejected => {
                self.transaction_status(&sender, nonce)
                    .set(&TransactionStatus::Rejected);

                let tx = self.transactions_by_nonce(&sender).get(nonce);

                Ok(self.refund_esdt_token(tx.from, tx.token_identifier, tx.amount))
            }
            _ => sc_error!("Transaction status may only be set to Executed or Rejected"),
        }
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

    fn burn_esdt_token(
        &self,
        token_identifier: &TokenIdentifier,
        amount: &Self::BigUint,
    ) -> AsyncCall<Self::SendApi> {
        ESDTSystemSmartContractProxy::new_proxy_obj(self.send())
            .burn(token_identifier.as_esdt_identifier(), amount)
            .async_call()
    }

    fn refund_esdt_token(
        &self,
        to: Address,
        token_id: TokenIdentifier,
        amount: Self::BigUint,
    ) -> AsyncCall<Self::SendApi> {
        ContractCall::<Self::SendApi, ()>::new(
            self.send(),
            to.clone(),
            BoxedBytes::from(self.data_or_empty(&to, b"refund")),
        )
        .with_token_transfer(token_id, amount)
        .async_call()
    }

    fn data_or_empty(&self, to: &Address, data: &'static [u8]) -> &[u8] {
        if self.blockchain().is_smart_contract(to) {
            &[]
        } else {
            data
        }
    }

    fn reserve_fee(&self, from: Address) {
        self.ethereum_fee_prepay_proxy(self.fee_estimator_contract_address().get())
            .reserve_fee(from, TransactionType::Erc20, Priority::Low)
            .execute_on_dest_context(self.blockchain().get_gas_left());
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
}
