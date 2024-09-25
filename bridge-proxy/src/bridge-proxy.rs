#![no_std]
use multiversx_sc::{imports::*, storage::StorageKey};

pub mod bridge_proxy_contract_proxy;
pub mod bridged_tokens_wrapper_proxy;
pub mod config;
pub mod esdt_safe_proxy;

use transaction::{CallData, EthTransaction};
const MIN_GAS_LIMIT_FOR_SC_CALL: u64 = 10_000_000;
const DEFAULT_GAS_LIMIT_FOR_REFUND_CALLBACK: u64 = 20_000_000; // 20 million
const CHAIN_SPECIFIC_TO_UNIVERSAL_TOKEN_MAPPING: &[u8] = b"chainSpecificToUniversalMapping";
const DELAY_BEFORE_OWNER_CAN_CANCEL_TRANSACTION: u64 = 10;

#[multiversx_sc::contract]
pub trait BridgeProxyContract:
    config::ConfigModule + multiversx_sc_modules::pause::PauseModule
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
            || call_data.gas_limit == 0
            || call_data.gas_limit < MIN_GAS_LIMIT_FOR_SC_CALL
        {
            self.finish_execute_gracefully(tx_id);
            return;
        }

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

        let block_epoch = self.blockchain().get_block_epoch();
        self.ongoing_execution(tx_id).set(block_epoch);
        tx_call.register_promise();
    }

    #[endpoint(cancel)]
    fn cancel(&self, tx_id: usize) {
        let tx_start_epoch = self.ongoing_execution(tx_id).get();
        let current_block_epoch = self.blockchain().get_block_epoch();
        require!(
            current_block_epoch - tx_start_epoch > DELAY_BEFORE_OWNER_CAN_CANCEL_TRANSACTION,
            "Transaction can't be cancelled yet"
        );

        let tx = self.get_pending_transaction_by_id(tx_id);
        let payment = self.payments(tx_id).get();
        self.tx().to(tx.refund_address).payment(payment).transfer();
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
        let payment = self.payments(tx_id).get();
        let bridged_tokens_wrapper_addr = self.bridged_tokens_wrapper_address().get();

        let mut storage_key = StorageKey::new(CHAIN_SPECIFIC_TO_UNIVERSAL_TOKEN_MAPPING);
        storage_key.append_item(&tx.token_id);

        let chain_specific_to_universal_token_mapper: SingleValueMapper<
            TokenIdentifier,
            ManagedAddress,
        > = SingleValueMapper::<_, _, ManagedAddress>::new_from_address(
            self.bridged_tokens_wrapper_address().get(),
            storage_key,
        );

        let chain_specific_token_id = chain_specific_to_universal_token_mapper.get();

        // Check if token is native or wrapped
        if chain_specific_token_id == payment.token_identifier {
            self.tx()
                .to(bridged_tokens_wrapper_addr)
                .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
                .unwrap_token_create_transaction(&tx.token_id, tx.from)
                .single_esdt(
                    &payment.token_identifier,
                    payment.token_nonce,
                    &payment.amount,
                )
                .sync_call();
        } else {
            self.tx()
                .to(self.esdt_safe_contract_address().get())
                .typed(esdt_safe_proxy::EsdtSafeProxy)
                .create_transaction(tx.from)
                .single_esdt(&tx.token_id, 0, &tx.amount)
                .sync_call();
        }
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

    fn update_lowest_tx_id(&self) {
        let mut new_lowest = self.lowest_tx_id().get();
        let len = self.pending_transactions().len();

        while new_lowest < len && self.pending_transactions().item_is_empty(new_lowest) {
            new_lowest += 1;
        }

        self.lowest_tx_id().set(new_lowest);
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
