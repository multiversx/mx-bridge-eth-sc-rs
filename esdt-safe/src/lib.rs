#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use elrond_wasm::{api::StorageReadApi, HexCallDataSerializer};

const ESDT_SYSTEM_SC_ADDRESS_ARRAY: [u8; 32] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0xff, 0xff,
];

const ESDT_BURN_STRING: &[u8] = b"ESDTBurn";

#[derive(TopEncode, TopDecode, TypeAbi)]
pub struct Transaction<BigUint: BigUintApi> {
    from: Address,
    to: Address,
    token_identifier: TokenIdentifier,
    amount: BigUint,
}

#[derive(TopEncode, TopDecode, TypeAbi, PartialEq)]
pub enum TransactionStatus {
    None,
    Pending,
    InProgress,
    Executed,
    Rejected,
}

#[elrond_wasm_derive::contract(EsdtSafeImpl)]
pub trait EsdtSafe {
    #[init]
    fn init(&self, transaction_fee: BigUint, token_whitelist: &[TokenIdentifier]) {
        self.set_transaction_fee(&transaction_fee);

        for token in token_whitelist {
            self.token_whitelist().insert(token.clone());
        }
    }

    // endpoints - owner-only
    // the owner will probably be a multisig SC

    #[endpoint(setTransactionFee)]
    fn set_transaction_fee_endpoint(&self, transaction_fee: BigUint) -> SCResult<()> {
        only_owner!(self, "only owner may call this function");

        self.set_transaction_fee(&transaction_fee);

        Ok(())
    }

    #[endpoint(addTokenToWhitelist)]
    fn add_token_to_whitelist(&self, token_identifier: TokenIdentifier) -> SCResult<()> {
        only_owner!(self, "only owner may call this function");

        self.token_whitelist().insert(token_identifier);

        Ok(())
    }

    #[endpoint(removeTokenFromWhitelist)]
    fn remove_token_from_whitelist(&self, token_identifier: TokenIdentifier) -> SCResult<()> {
        only_owner!(self, "only owner may call this function");

        self.token_whitelist().remove(&token_identifier);

        Ok(())
    }

    #[endpoint(getNextPendingTransaction)]
    fn get_next_pending_transaction(&self) -> SCResult<OptionalResult<Transaction<BigUint>>> {
        only_owner!(self, "only owner may call this function");

        match self.pending_transaction_address_nonce_list().pop_front() {
            Some((sender, nonce)) => {
                self.set_transaction_status(&sender, nonce, TransactionStatus::InProgress);

                Ok(OptionalResult::Some(
                    self.transactions_by_nonce(&sender).get(nonce),
                ))
            }
            None => Ok(OptionalResult::None),
        }
    }

    #[endpoint(setTransactionStatus)]
    fn set_transaction_status_endpoint(
        &self,
        sender: Address,
        nonce: usize,
        transaction_status: TransactionStatus,
    ) -> SCResult<()> {
        only_owner!(self, "only owner may call this function");

        require!(
            self.get_transaction_status(&sender, nonce) == TransactionStatus::InProgress,
            "Transaction has to be executed first"
        );

        match transaction_status {
            TransactionStatus::Executed => {
                self.set_transaction_status(&sender, nonce, TransactionStatus::Executed);

                let tx = self.transactions_by_nonce(&sender).get(nonce);
                self.burn_esdt_token(&tx.token_identifier, &tx.amount);
            }
            TransactionStatus::Rejected => {
                self.set_transaction_status(&sender, nonce, TransactionStatus::Rejected);

                let tx = self.transactions_by_nonce(&sender).get(nonce);
                self.refund_esdt_token(&tx.from, &tx.token_identifier, &tx.amount);
            }
            _ => return sc_error!("Transaction status may only be set to Executed or Rejected"),
        }

        self.set_transaction_status(&sender, nonce, transaction_status);

        Ok(())
    }

    #[endpoint]
    fn claim(&self) -> SCResult<()> {
        only_owner!(self, "only owner may call this function");

        self.send().direct_egld(
            &self.get_caller(),
            &self.get_claimable_transaction_fee(),
            b"claim",
        );

        self.set_claimable_transaction_fee(&BigUint::zero());

        Ok(())
    }

    // endpoints

    #[payable("EGLD")]
    #[endpoint(depositEgldForTransactionFee)]
    fn deposit_egld_for_transaction_fee(&self, #[payment] payment: BigUint) {
        let caller = self.get_caller();
        let mut caller_deposit = self.get_deposit(&caller);
        caller_deposit += payment;
        self.set_deposit(&caller, &caller_deposit);
    }

    /// amount argument is optional, defaults to max possible if not provided
    #[endpoint(whithdrawDeposit)]
    fn whithdraw_deposit(&self, #[var_args] opt_amount: OptionalArg<BigUint>) -> SCResult<()> {
        let caller = self.get_caller();
        let caller_deposit = self.get_deposit(&caller);
        let amount = match opt_amount {
            OptionalArg::Some(value) => value,
            OptionalArg::None => caller_deposit.clone(),
        };

        require!(amount <= caller_deposit, "Trying to whithdraw too much");

        let deposit_remaining = &caller_deposit - &amount;
        self.send().direct_egld(&caller, &amount, b"whitdrawal");
        self.set_deposit(&caller, &deposit_remaining);

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
        let caller_deposit = self.get_deposit(&caller);
        let transaction_fee = self.get_transaction_fee();

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

        let sender_nonce = self.get_transactions_by_nonce_len(&caller);

        self.set_transaction_status(&caller, sender_nonce, TransactionStatus::Pending);
        self.pending_transaction_address_nonce_list()
            .push_back((caller.clone(), sender_nonce));

        // deduct fee from deposit and add to claimable fees pool
        let deposit_remaining = &caller_deposit - &transaction_fee;
        self.set_deposit(&caller, &deposit_remaining);

        let mut claimable_transaction_fee = self.get_claimable_transaction_fee();
        claimable_transaction_fee += transaction_fee;
        self.set_claimable_transaction_fee(&claimable_transaction_fee);

        Ok(())
    }

    // private

    fn burn_esdt_token(&self, token_identifier: &TokenIdentifier, amount: &BigUint) {
        let mut serializer = HexCallDataSerializer::new(ESDT_BURN_STRING);
        serializer.push_argument_bytes(token_identifier.as_slice());
        serializer.push_argument_bytes(&amount.to_bytes_be());

        self.send().async_call_raw(
            &Address::from(ESDT_SYSTEM_SC_ADDRESS_ARRAY),
            &BigUint::zero(),
            serializer.as_slice(),
        );
    }

    fn refund_esdt_token(
        &self,
        to: &Address,
        token_identifier: &TokenIdentifier,
        amount: &BigUint,
    ) {
        self.send()
            .direct_esdt(to, token_identifier.as_slice(), amount, b"refund");
    }

    // storage

    // transaction fee, can only be set by owner

    #[view(getTransactionFee)]
    #[storage_get("transactionFee")]
    fn get_transaction_fee(&self) -> BigUint;

    #[storage_set("transactionFee")]
    fn set_transaction_fee(&self, transaction_fee: &BigUint);

    // transaction fees available for claiming, only added to this pool after the transaction was added in Pending status

    #[view(getClaimableTransactionFee)]
    #[storage_get("claimableTransactionFee")]
    fn get_claimable_transaction_fee(&self) -> BigUint;

    #[storage_set("claimableTransactionFee")]
    fn set_claimable_transaction_fee(&self, claimable_transaction_fee: &BigUint);

    // token whitelist

    #[storage_mapper("tokenWhitelist")]
    fn token_whitelist(&self) -> SetMapper<Self::Storage, TokenIdentifier>;

    // eGLD amounts deposited by each address, for the sole purpose of paying for transaction fees

    #[view(getDeposit)]
    #[storage_get("deposit")]
    fn get_deposit(&self, address: &Address) -> BigUint;

    #[storage_set("deposit")]
    fn set_deposit(&self, address: &Address, deposit: &BigUint);

    // transactions for each address, sorted by nonce
    // due to how VecMapper works internally, nonces will start at 1

    #[storage_mapper("transactionsByNonce")]
    fn transactions_by_nonce(
        &self,
        address: &Address,
    ) -> VecMapper<Self::Storage, Transaction<BigUint>>;

    #[storage_get("transactionStatus")]
    fn get_transaction_status(&self, sender: &Address, nonce: usize) -> TransactionStatus;

    #[storage_set("transactionStatus")]
    fn set_transaction_status(
        &self,
        sender: &Address,
        nonce: usize,
        transaction_status: TransactionStatus,
    );

    #[storage_mapper("pendingTransactionList")]
    fn pending_transaction_address_nonce_list(
        &self,
    ) -> LinkedListMapper<Self::Storage, (Address, usize)>;

    // TODO: Remove in the next patch, VecMapper will have a len() method then

    fn get_transactions_by_nonce_len(&self, address: &Address) -> usize {
        self.get_storage_raw()
            .storage_load_u64(&[b"transactionsByNonce", address.as_bytes()].concat())
            as usize
    }
}
