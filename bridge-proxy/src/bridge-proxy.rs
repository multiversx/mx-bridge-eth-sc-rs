#![no_std]
use multiversx_sc::imports::*;
use multiversx_sc_modules::ongoing_operation::*;

pub mod bridge_proxy_contract_proxy;
pub mod bridged_tokens_wrapper_proxy;
pub mod config;
pub mod esdt_safe_proxy;

use transaction::{CallData, EthTransaction};
const MIN_GAS_LIMIT_FOR_SC_CALL: u64 = 10_000_000;
const MAX_GAS_LIMIT_FOR_SC_CALL: u64 = 249999999;
const DEFAULT_GAS_LIMIT_FOR_REFUND_CALLBACK: u64 = 20_000_000; // 20 million
const DELAY_BEFORE_OWNER_CAN_CANCEL_TRANSACTION: u64 = 300;
const MIN_GAS_TO_SAVE_PROGRESS: u64 = 100_000;

#[multiversx_sc::contract]
pub trait BridgeProxyContract:
    config::ConfigModule 
    + multiversx_sc_modules::pause::PauseModule
    + multiversx_sc_modules::ongoing_operation::OngoingOperationModule
{
    #[init]
    fn init(&self, opt_multi_transfer_address: OptionalValue<ManagedAddress>) {
        self.set_multi_transfer_contract_address(opt_multi_transfer_address);
        self.lowest_tx_id().set(1);
        self.set_paused(true);
    }

    #[upgrade]
    fn upgrade(&self) {
        self.set_paused(true);
    }

    #[payable("*")]
    #[endpoint]
    fn deposit(&self, eth_tx: EthTransaction<Self::Api>) {
        self.require_not_paused();
        let caller = self.blockchain().get_caller();
        let payment = self.call_value().single_esdt();
        require!(
            caller == self.multi_transfer_address().get(),
            "Only MultiTransfer can do deposits"
        );
        let tx_id = self.pending_transactions().push(&eth_tx);
        self.payments(tx_id).set(&payment);
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
        require!(gas_left > call_data.gas_limit + DEFAULT_GAS_LIMIT_FOR_REFUND_CALLBACK, "Not enough gas to execute");

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
        self.tx()
            .to(esdt_safe_contract_address)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .create_transaction(tx.from, OptionalValue::Some(tx.to))
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
        
        let transfers = self
            .tx()
            .to(bridged_tokens_wrapper_address)           .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
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
        self.pending_transactions().clear_entry_unchecked(tx_id);
        self.update_lowest_tx_id();
        self.ongoing_execution(tx_id).clear();
    }

    #[endpoint(updateLowestTxId)]
    fn update_lowest_tx_id(&self) {
        let mut new_lowest = self.lowest_tx_id().get();
        let len = self.pending_transactions().len();

        self.run_while_it_has_gas(MIN_GAS_TO_SAVE_PROGRESS, || {
            if !self.empty_element(new_lowest, len) {
                return STOP_OP;
            }

            new_lowest += 1;

            CONTINUE_OP
        });

        self.lowest_tx_id().set(new_lowest);
    }

    fn empty_element(&self, current_index: usize, len: usize) -> bool {
        current_index < len && self.pending_transactions().item_is_empty(current_index)
    }

    #[view(getPendingTransactionById)]
    fn get_pending_transaction_by_id(&self, tx_id: usize) -> EthTransaction<Self::Api> {
        self.pending_transactions()
            .get_or_else(tx_id, || panic!("Invalid tx id"))
    }

    #[view(getPendingTransactions)]
    fn get_pending_transactions(
        &self,
    ) -> MultiValueEncoded<MultiValue2<usize, EthTransaction<Self::Api>>> {
        let lowest_tx_id = self.lowest_tx_id().get();
        let len = self.pending_transactions().len();

        let mut transactions = MultiValueEncoded::new();
        for i in lowest_tx_id..=len {
            if self.pending_transactions().item_is_empty(i) {
                continue;
            }
            let tx = self.pending_transactions().get_unchecked(i);
            transactions.push(MultiValue2((i, tx)));
        }
        transactions
    }
}
