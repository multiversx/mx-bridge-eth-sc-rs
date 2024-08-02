#![no_std]
use multiversx_sc::{contract_base::ManagedSerializer, imports::*};

pub mod bridge_proxy_contract_proxy;
pub mod config;
pub mod esdt_safe_proxy;

use transaction::{CallData, EthTransaction};

const MIN_GAS_LIMIT_FOR_SC_CALL: u64 = 10_000_000;

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
        require!(
            caller == self.multi_transfer_address().get(),
            "Only MultiTransfer can do deposits"
        );
        self.pending_transactions().push(&eth_tx);
    }

    #[endpoint(execute)]
    fn execute(&self, tx_id: usize) {
        self.require_not_paused();
        let tx = self.get_pending_transaction_by_id(tx_id);
        let mb_aux = ManagedBufferReadToEnd::from(tx.call_data);
        let managed_serializer = ManagedSerializer::new();
        let call_data: CallData<Self::Api> =
            managed_serializer.top_decode_from_managed_buffer(&mb_aux.into_managed_buffer());

        let mut refund = false;

        if call_data.endpoint.is_empty()
            || call_data.gas_limit == 0
            || call_data.gas_limit < MIN_GAS_LIMIT_FOR_SC_CALL
        {
            refund = true;
        }

        if refund {
            self.refund_transaction(tx_id);
        }

        // let unwrapped_args = call_data.args.unwrap_or_else(|| ManagedVec::new());
        // let unwraped_args: ManagedVec<ManagedBuffer> = match call_data.args {
        //     ManagedOption::some(args) => args.unwrap(),
        //     ManagedOption::none() => ManagedVec::new(),
        // };

        let tx_call = if call_data.args.is_some() {
            let args = unsafe { call_data.args.unwrap_no_check() };
            self.tx()
                .to(&tx.to)
                .raw_call(call_data.endpoint)
                .arguments_raw(args.into())
                .gas(call_data.gas_limit) //TODO: set gas limit to this call
                .callback(self.callbacks().execution_callback(tx_id))
        } else {
            self.tx()
                .to(&tx.to)
                .raw_call(call_data.endpoint)
                // .arguments_raw(call_data.args.unwrap_no_check().into())
                .gas(call_data.gas_limit) //TODO: set gas limit to this call
                .callback(self.callbacks().execution_callback(tx_id))
        };

        if tx.amount == 0 {
            tx_call.register_promise();
        } else {
            tx_call
                .single_esdt(&tx.token_id, 0, &tx.amount)
                .register_promise();
        }
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
        let esdt_safe_addr = self.esdt_safe_address().get();

        self.tx()
            .to(esdt_safe_addr)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .create_transaction(tx.from)
            .single_esdt(&tx.token_id, 0, &tx.amount)
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
