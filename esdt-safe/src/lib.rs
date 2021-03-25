#![no_std]

use transaction::*;

elrond_wasm::imports!();

#[elrond_wasm_derive::contract(EsdtSafeImpl)]
pub trait EsdtSafe {
    #[init]
    fn init(
        &self,
        transaction_fee: BigUint,
        #[var_args] token_whitelist: VarArgs<TokenIdentifier>,
    ) {
        self.transaction_fee().set(&transaction_fee);

        for token in token_whitelist.into_vec() {
            self.token_whitelist().insert(token.clone());
        }
    }

    // endpoints - owner-only
    // the owner will probably be a multisig SC

    #[endpoint(setTransactionFee)]
    fn set_transaction_fee_endpoint(&self, transaction_fee: BigUint) -> SCResult<()> {
        only_owner!(self, "only owner may call this function");

        self.transaction_fee().set(&transaction_fee);

        Ok(())
    }

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
    ) -> SCResult<MultiResult5<Nonce, Address, Address, TokenIdentifier, BigUint>> {
        only_owner!(self, "only owner may call this function");

        match self.pending_transaction_address_nonce_list().pop_front() {
            Some((sender, nonce)) => {
                self.transaction_status(&sender, nonce)
                    .set(&TransactionStatus::InProgress);

                let tx = self.transactions_by_nonce(&sender).get(nonce);

                Ok((nonce, tx.from, tx.to, tx.token_identifier, tx.amount).into())
            }
            None => Ok((
                0,
                Address::zero(),
                Address::zero(),
                TokenIdentifier::egld(),
                BigUint::zero(),
            )
                .into()),
        }
    }

    #[endpoint(setTransactionStatus)]
    fn set_transaction_status_endpoint(
        &self,
        sender: Address,
        nonce: Nonce,
        transaction_status: TransactionStatus,
    ) -> SCResult<AsyncCall<BigUint>> {
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

    #[endpoint]
    fn claim(&self) -> SCResult<()> {
        only_owner!(self, "only owner may call this function");

        let caller = self.get_caller();
        self.send().direct_egld(
            &caller,
            &self.claimable_transaction_fee().get(),
            self.data_or_empty(&caller, b"claim"),
        );

        self.claimable_transaction_fee().clear();

        Ok(())
    }

    // endpoints

    #[payable("EGLD")]
    #[endpoint(depositEgldForTransactionFee)]
    fn deposit_egld_for_transaction_fee(&self, #[payment] payment: BigUint) {
        let caller = self.get_caller();
        let mut caller_deposit = self.deposit(&caller).get();
        caller_deposit += payment;
        self.deposit(&caller).set(&caller_deposit);
    }

    /// amount argument is optional, defaults to max possible if not provided
    #[endpoint(whithdrawDeposit)]
    fn whithdraw_deposit(&self, #[var_args] opt_amount: OptionalArg<BigUint>) -> SCResult<()> {
        let caller = self.get_caller();
        let caller_deposit = self.deposit(&caller).get();
        let amount = match opt_amount {
            OptionalArg::Some(value) => value,
            OptionalArg::None => caller_deposit.clone(),
        };

        require!(amount <= caller_deposit, "Trying to whithdraw too much");

        let deposit_remaining = &caller_deposit - &amount;
        self.send()
            .direct_egld(&caller, &amount, self.data_or_empty(&caller, b"whitdrawal"));
        self.deposit(&caller).set(&deposit_remaining);

        Ok(())
    }

    #[payable("*")]
    #[endpoint(createTransaction)]
    fn create_transaction(
        &self,
        #[payment_token] payment_token: TokenIdentifier,
        #[payment] payment: BigUint,
        to: Address,
    ) -> SCResult<()> {
        require!(
            self.token_whitelist().contains(&payment_token),
            "Payment token is not on whitelist. Transaction rejected"
        );
        require!(payment > 0, "Must transfer more than 0");
        require!(!to.is_zero(), "Can't transfer to address zero");

        let caller = self.get_caller();
        let caller_deposit = self.deposit(&caller).get();
        let transaction_fee = self.transaction_fee().get();

        require!(
            caller_deposit >= transaction_fee,
            "Must deposit transaction fee first"
        );

        let tx = Transaction {
            from: caller.clone(),
            to,
            token_identifier: payment_token,
            amount: payment,
        };

        self.transactions_by_nonce(&caller).push(&tx);

        let sender_nonce = self.transactions_by_nonce(&caller).len();

        self.transaction_status(&caller, sender_nonce)
            .set(&TransactionStatus::Pending);
        self.pending_transaction_address_nonce_list()
            .push_back((caller.clone(), sender_nonce));

        // deduct fee from deposit and add to claimable fees pool
        let deposit_remaining = &caller_deposit - &transaction_fee;
        self.deposit(&caller).set(&deposit_remaining);

        let mut claimable_transaction_fee = self.claimable_transaction_fee().get();
        claimable_transaction_fee += transaction_fee;
        self.claimable_transaction_fee()
            .set(&claimable_transaction_fee);

        Ok(())
    }

    // private

    fn burn_esdt_token(
        &self,
        token_identifier: &TokenIdentifier,
        amount: &BigUint,
    ) -> AsyncCall<BigUint> {
        ESDTSystemSmartContractProxy::new()
            .burn(token_identifier.as_esdt_identifier(), amount)
            .async_call()
    }

    fn refund_esdt_token(
        &self,
        to: Address,
        token_id: TokenIdentifier,
        amount: BigUint,
    ) -> AsyncCall<BigUint> {
        ContractCall::<BigUint, ()>::new(
            to.clone(),
            token_id,
            amount,
            BoxedBytes::from(self.data_or_empty(&to, b"refund")),
        )
        .async_call()
    }

    fn data_or_empty(&self, to: &Address, data: &'static [u8]) -> &[u8] {
        if self.is_smart_contract(to) {
            &[]
        } else {
            data
        }
    }

    // storage

    // transaction fee, can only be set by owner

    #[view(getTransactionFee)]
    #[storage_mapper("transactionFee")]
    fn transaction_fee(&self) -> SingleValueMapper<Self::Storage, BigUint>;

    // transaction fees available for claiming, only added to this pool after the transaction was added in Pending status

    #[view(getClaimableTransactionFee)]
    #[storage_mapper("claimableTransactionFee")]
    fn claimable_transaction_fee(&self) -> SingleValueMapper<Self::Storage, BigUint>;

    // token whitelist

    #[storage_mapper("tokenWhitelist")]
    fn token_whitelist(&self) -> SetMapper<Self::Storage, TokenIdentifier>;

    // eGLD amounts deposited by each address, for the sole purpose of paying for transaction fees

    #[view(getDeposit)]
    #[storage_mapper("deposit")]
    fn deposit(&self, address: &Address) -> SingleValueMapper<Self::Storage, BigUint>;

    // transactions for each address, sorted by nonce
    // due to how VecMapper works internally, nonces will start at 1

    #[storage_mapper("transactionsByNonce")]
    fn transactions_by_nonce(
        &self,
        address: &Address,
    ) -> VecMapper<Self::Storage, Transaction<BigUint>>;

    #[storage_mapper("transactionStatus")]
    fn transaction_status(
        &self,
        sender: &Address,
        nonce: Nonce,
    ) -> SingleValueMapper<Self::Storage, TransactionStatus>;

    #[storage_mapper("pendingTransactionList")]
    fn pending_transaction_address_nonce_list(
        &self,
    ) -> LinkedListMapper<Self::Storage, (Address, Nonce)>;
}
