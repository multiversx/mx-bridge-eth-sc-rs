#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

mod config;

use config::EthTransactionPayment;
use transaction::EthTransaction;

/// An empty contract. To be used as a template when starting a new contract from scratch.
#[multiversx_sc::contract]
pub trait BridgeProxyContract: config::ConfigModule {
    #[init]
    fn init(&self, multi_transfer_address: ManagedAddress) {
        self.multi_transfer_address()
            .set_if_empty(&multi_transfer_address);
    }

    #[endpoint]
    fn deposit(&self, eth_tx: EthTransaction<Self::Api>) {
        let (token_id, nonce, amount) = self.call_value().single_esdt().into_tuple();
        self.eth_transaction_list()
            .push_back(EthTransactionPayment {
                token_id,
                nonce,
                amount,
                eth_tx,
            });
    }

    #[endpoint]
    fn execute(&self) {
        for loop_tx in self.eth_transaction_list().iter() {
            let tx = loop_tx.get_value_as_ref();
            self.send()
                .contract_call::<IgnoreValue>(tx.eth_tx.to.clone(), tx.eth_tx.data.clone())
                .with_esdt_transfer((tx.token_id.clone(), tx.nonce, tx.amount.clone()))
                .with_gas_limit(tx.eth_tx.gas_limit)
                .transfer_execute();

            //TODO Check if transaction failed, add it to `eth_failed_transaction_list`
        }
    }

    #[endpoint(executeWithAsnyc)]
    fn execute_with_async(&self) {
        let tx_node = self
            .eth_transaction_list()
            .front()
            .unwrap_or_else(|| sc_panic!("No more ETH transactions!"));
        let tx = tx_node.get_value_as_ref();
        
        self.send()
            .contract_call::<IgnoreValue>(tx.eth_tx.to.clone(), tx.eth_tx.data.clone())
            .with_esdt_transfer((tx.token_id.clone(), tx.nonce, tx.amount.clone()))
            .with_gas_limit(tx.eth_tx.gas_limit)
            .async_call()
            .with_callback(self.callbacks().failed_execution_callback(tx))
            .call_and_exit();
    }

    #[callback]
    fn failed_execution_callback(&self, tx: &EthTransactionPayment<Self::Api>) {
        self.eth_failed_transaction_list().push_back(tx.clone());
    }

    #[endpoint(refundTransactions)]
    fn refund_transactions(&self) {}
}
