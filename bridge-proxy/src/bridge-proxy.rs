#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub mod config;

use esdt_safe::ProxyTrait as _;
use transaction::EthTransaction;

#[multiversx_sc::contract]
pub trait BridgeProxyContract:
    config::ConfigModule + multiversx_sc_modules::pause::PauseModule
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
    fn deposit(&self, eth_tx: EthTransaction<Self::Api>) {
        self.require_not_paused();
        let (token_id, amount) = self.call_value().single_fungible_esdt();
        require!(token_id == eth_tx.token_id, "Invalid token id");
        require!(amount == eth_tx.amount, "Invalid amount");
        self.pending_transactions().push(&eth_tx);
    }

    #[endpoint(executeWithAsnyc)]
    fn execute_with_async(&self, tx_id: usize) {
        self.require_not_paused();
        let tx = self.get_pending_transaction_by_id(tx_id);

        require!(
            tx.call_data.is_some(),
            "There is no data for a SC call!"
        );

        let call_data = unsafe { tx.call_data.clone().unwrap_unchecked() };
        self.send()
            .contract_call::<IgnoreValue>(tx.to.clone(), call_data.endpoint.clone())
            .with_raw_arguments(call_data.args.clone().into())
            .with_esdt_transfer((tx.token_id.clone(), 0, tx.amount.clone()))
            .with_gas_limit(call_data.gas_limit)
            .async_call()
            .with_callback(self.callbacks().execution_callback(tx_id))
            .call_and_exit();
    }

    #[callback]
    fn execution_callback(
        &self,
        #[call_result] result: ManagedAsyncCallResult<()>,
        tx_id: usize,
    ) {
        if result.is_err() {
            self.refund_transaction(tx_id);
        }
        self.pending_transactions().clear_entry_unchecked(tx_id);
    }

    fn refund_transaction(&self, tx_id: usize) {
        let tx = self.get_pending_transaction_by_id(tx_id);

        let _: IgnoreValue = self
            .get_esdt_safe_proxy_instance()
            .create_transaction(tx.from)
            .with_esdt_transfer((tx.token_id.clone(), 0, tx.amount.clone()))
            .execute_on_dest_context();
    }

    #[view(getPendingTransactionById)]
    fn get_pending_transaction_by_id(&self, tx_id: usize) -> EthTransaction<Self::Api> {
        self.pending_transactions().get_or_else(tx_id, || panic!("Invalid tx id"))
    }

}
