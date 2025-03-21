#![no_std]
use multiversx_sc::imports::*;

pub mod bridge_proxy_contract_proxy;
pub mod bridged_tokens_wrapper_proxy;
pub mod config;
pub mod esdt_safe_proxy;

use transaction::{CallData, EthTransaction};
const MIN_GAS_LIMIT_FOR_SC_CALL: u64 = 10_000_000;
const MAX_GAS_LIMIT_FOR_SC_CALL: u64 = 249999999;
const DEFAULT_GAS_LIMIT_FOR_REFUND_CALLBACK: u64 = 20_000_000; // 20 million
const DELAY_BEFORE_OWNER_CAN_CANCEL_TRANSACTION: u64 = 300;

#[multiversx_sc::contract]
pub trait BridgeProxyContract:
    config::ConfigModule
    + multiversx_sc_modules::pause::PauseModule
{
    #[init]
    fn init(&self, opt_multi_transfer_address: OptionalValue<ManagedAddress>) {
        self.set_multi_transfer_contract_address(opt_multi_transfer_address);
        self.set_paused(true);
    }

    #[upgrade]
    fn upgrade(&self) {
        self.set_paused(true);
    }

    #[payable("*")]
    #[endpoint]
    fn deposit(&self, eth_tx: EthTransaction<Self::Api>, batch_id: u64) {
        self.require_not_paused();
        let caller = self.blockchain().get_caller();
        let payment = self.call_value().single_esdt();
        require!(
            caller == self.multi_transfer_address().get(),
            "Only MultiTransfer can do deposits"
        );
        let next_tx_id = self.get_next_tx_id();
        self.pending_transactions().insert(next_tx_id, eth_tx);
        self.payments(next_tx_id).set(&payment);
        self.batch_id(next_tx_id).set(batch_id);
    }

    #[endpoint(execute)]
    fn execute(&self, tx_id: usize) {
        self.require_not_paused();
        require!(
            self.ongoing_execution(tx_id).is_empty(),
            "Transaction is already being executed"
        );
        let tx = self.get_pending_transaction_by_id(tx_id);
        let payment = self.payments(tx_id).get();

        require!(payment.amount != 0, "No amount bridged");

        let call_data: CallData<Self::Api> = if tx.call_data.is_some() {
            let unwraped_call_data = unsafe { tx.call_data.unwrap_no_check() };

            let Ok(call_data) = CallData::top_decode(unwraped_call_data) else {
                self.finish_execute_gracefully(tx_id);
                return;
            };

            call_data
        } else {
            CallData::default()
        };

        if call_data.endpoint.is_empty()
            || call_data.gas_limit < MIN_GAS_LIMIT_FOR_SC_CALL
            || call_data.gas_limit > MAX_GAS_LIMIT_FOR_SC_CALL
        {
            self.finish_execute_gracefully(tx_id);
            return;
        }

        let gas_left = self.blockchain().get_gas_left();
        require!(
            gas_left > call_data.gas_limit + DEFAULT_GAS_LIMIT_FOR_REFUND_CALLBACK,
            "Not enough gas to execute"
        );

        let tx_call = self
            .tx()
            .to(&tx.to)
            .raw_call(call_data.endpoint)
            .gas(call_data.gas_limit)
            .callback(self.callbacks().execution_callback(tx_id))
            .with_extra_gas_for_callback(DEFAULT_GAS_LIMIT_FOR_REFUND_CALLBACK)
            .with_esdt_transfer(payment);

        let tx_call = if call_data.args.is_some() {
            let args = unsafe { call_data.args.unwrap_no_check() };
            tx_call.arguments_raw(args.into())
        } else {
            tx_call
        };

        let block_round = self.blockchain().get_block_round();
        self.ongoing_execution(tx_id).set(block_round);
        tx_call.register_promise();
    }

    // TODO: will activate endpoint in a future release
    // #[endpoint(cancel)]
    fn cancel(&self, tx_id: usize) {
        let tx_start_round = self.ongoing_execution(tx_id).get();
        let current_block_round = self.blockchain().get_block_round();
        require!(
            current_block_round - tx_start_round > DELAY_BEFORE_OWNER_CAN_CANCEL_TRANSACTION,
            "Transaction can't be cancelled yet"
        );

        let tx = self.get_pending_transaction_by_id(tx_id);
        let payment = self.payments(tx_id).get();
        self.tx().to(tx.to).payment(payment).transfer();
        self.cleanup_transaction(tx_id);
    }
    #[promises_callback]
    fn execution_callback(&self, #[call_result] result: ManagedAsyncCallResult<()>, tx_id: usize) {
        if result.is_err() {
            self.refund_transaction(tx_id);
        }
        self.cleanup_transaction(tx_id);
    }

    fn refund_transaction(&self, tx_id: usize) {
        let tx = self.get_pending_transaction_by_id(tx_id);
        let esdt_safe_contract_address = self.esdt_safe_contract_address().get();

        let unwrapped_token = self.unwrap_token(&tx.token_id, tx_id);
        let batch_id = self.batch_id(tx_id).get();
        self.tx()
            .to(esdt_safe_contract_address)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .create_transaction(
                tx.from,
                OptionalValue::Some(esdt_safe_proxy::RefundInfo {
                    address: tx.to,
                    initial_batch_id: batch_id,
                    initial_nonce: tx.tx_nonce,
                }),
            )
            .single_esdt(
                &unwrapped_token.token_identifier,
                unwrapped_token.token_nonce,
                &unwrapped_token.amount,
            )
            .sync_call();
    }

    fn unwrap_token(&self, requested_token: &TokenIdentifier, tx_id: usize) -> EsdtTokenPayment {
        let payment = self.payments(tx_id).get();
        let bridged_tokens_wrapper_address = self.bridged_tokens_wrapper_address().get();

        if requested_token == &payment.token_identifier {
            return payment;
        }

        let transfers = self
            .tx()
            .to(&bridged_tokens_wrapper_address)
            .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
            .unwrap_token(requested_token)
            .single_esdt(
                &payment.token_identifier,
                payment.token_nonce,
                &payment.amount,
            )
            .returns(ReturnsBackTransfers)
            .sync_call();

        require!(
            transfers.total_egld_amount == 0,
            "Expected only one esdt payment"
        );
        require!(
            transfers.esdt_payments.len() == 1,
            "Expected only one esdt payment"
        );
        transfers.esdt_payments.get(0)
    }

    fn finish_execute_gracefully(&self, tx_id: usize) {
        self.refund_transaction(tx_id);
        self.cleanup_transaction(tx_id);
    }

    fn cleanup_transaction(&self, tx_id: usize) {
        self.pending_transactions().remove(&tx_id);
        self.ongoing_execution(tx_id).clear();
    }

    fn get_next_tx_id(&self) -> usize {
        let mut next_tx_id = self.highest_tx_id().get();
        next_tx_id += 1;
        self.highest_tx_id().set(next_tx_id);
        next_tx_id
    }

    #[view(getPendingTransactionById)]
    fn get_pending_transaction_by_id(&self, tx_id: usize) -> EthTransaction<Self::Api> {
        let tx = self.pending_transactions().get(&tx_id);
        require!(tx.is_some(), "Invalid tx id");
        tx.unwrap()
    }

    #[view(getPendingTransactions)]
    fn get_pending_transactions(
        &self,
    ) -> MultiValueEncoded<MultiValue2<usize, EthTransaction<Self::Api>>> {
        let mut transactions = MultiValueEncoded::new();
        for (tx_id, tx) in self.pending_transactions().iter() {
            transactions.push(MultiValue2((tx_id, tx)));
        }
        transactions
    }
}
