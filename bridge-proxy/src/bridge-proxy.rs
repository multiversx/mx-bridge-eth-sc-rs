#![no_std]
use multiversx_sc::{contract_base::ManagedSerializer, imports::*};

pub mod bridge_proxy_contract_proxy;
pub mod bridged_tokens_wrapper;
pub mod bridged_tokens_wrapper_proxy;
pub mod config;

use transaction::{CallData, EthTransaction};

const MIN_GAS_LIMIT_FOR_SC_CALL: u64 = 10_000_000;
const DEFAULT_GAS_LIMIT_FOR_REFUND_CALLBACK: u64 = 20_000_000; // 20 million

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
        let tx = self.get_pending_transaction_by_id(tx_id);
        let payment = self.payments(tx_id).get();

        require!(payment.amount != 0, "No amount bridged");

        let call_data: CallData<Self::Api> = if tx.call_data.is_some() {
            let unwraped_call_data = unsafe { tx.call_data.unwrap_no_check() };
            let mb_aux = ManagedBufferReadToEnd::from(unwraped_call_data);
            let managed_serializer = ManagedSerializer::new();
            let call_data: CallData<Self::Api> =
                managed_serializer.top_decode_from_managed_buffer(&mb_aux.into_managed_buffer());
            call_data
        } else {
            CallData::default()
        };

        let mut refund = false;

        if call_data.endpoint.is_empty()
            || call_data.gas_limit == 0
            || call_data.gas_limit < MIN_GAS_LIMIT_FOR_SC_CALL
        {
            refund = true;
        }

        if refund {
            self.refund_transaction(tx_id);
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

        tx_call.register_promise();
    }

    #[promises_callback]
    fn execution_callback(&self, #[call_result] result: ManagedAsyncCallResult<()>, tx_id: usize) {
        if result.is_err() {
            self.refund_transaction(tx_id);
        }
        let lowest_tx_id = self.lowest_tx_id().get();
        if tx_id < lowest_tx_id {
            self.lowest_tx_id().set(tx_id + 1);
        }
        self.pending_transactions().clear_entry_unchecked(tx_id);
    }

    fn refund_transaction(&self, tx_id: usize) {
        let tx = self.get_pending_transaction_by_id(tx_id);
        let payment = self.payments(tx_id).get();
        let esdt_safe_addr = self.bridged_tokens_wrapper_address().get();

        self.tx()
            .to(esdt_safe_addr)
            .typed(bridged_tokens_wrapper::BridgedTokensWrapperProxy)
            .unwrap_token_create_transaction(&tx.token_id, tx.from)
            .single_esdt(
                &payment.token_identifier,
                payment.token_nonce,
                &payment.amount,
            )
            .sync_call();
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
