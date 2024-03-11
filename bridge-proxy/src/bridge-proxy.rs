#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub mod config;

use transaction::{EthTransaction, EthTransactionPayment};

#[multiversx_sc::contract]
pub trait BridgeProxyContract:
    config::ConfigModule + multiversx_sc_modules::pause::PauseModule
{
    #[init]
    fn init(&self, opt_multi_transfer_address: OptionalValue<ManagedAddress>) {
        self.set_multi_transfer_contract_address(opt_multi_transfer_address);
    }

    #[payable("*")]
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

    #[endpoint(executeWithAsnyc)]
    fn execute_with_async(&self, tx_id: u32) {
        require!(self.not_paused(), "Contract is paused");
        let tx_node = self
            .eth_transaction_list()
            .remove_node_by_id(tx_id)
            .unwrap_or_else(|| sc_panic!("Invalid ETH transaction!"));
        let tx = tx_node.get_value_as_ref();

        require!(
            tx.eth_tx.call_data.is_some(),
            "There is no data for a SC call!"
        );

        let call_data = unsafe { tx.eth_tx.call_data.clone().unwrap_unchecked() };
        self.send()
            .contract_call::<IgnoreValue>(tx.eth_tx.to.clone(), call_data.endpoint.clone())
            .with_raw_arguments(call_data.args.clone().into())
            .with_esdt_transfer((tx.token_id.clone(), tx.nonce, tx.amount.clone()))
            .with_gas_limit(call_data.gas_limit)
            .async_call()
            .with_callback(self.callbacks().failed_execution_callback(tx))
            .call_and_exit();
    }

    #[callback]
    fn failed_execution_callback(
        &self,
        #[call_result] result: ManagedAsyncCallResult<()>,
        tx: &EthTransactionPayment<Self::Api>,
    ) {
        if result.is_err() {
            self.eth_failed_transaction_list().push_back(tx.clone());
        }
    }

    #[endpoint(refundTransactions)]
    fn refund_transactions(&self) -> MultiValueEncoded<EthTransactionPayment<Self::Api>> {
        // Send Failed Tx Structure
        let mut result = MultiValueEncoded::new();
        for tx_loop in self.eth_failed_transaction_list().iter() {
            let tx = tx_loop.get_value_cloned();
            result.push(tx);
        }

        // Send Funds
        let mut all_payments = ManagedVec::new();
        for failed_tx_loop in self.eth_failed_transaction_list().into_iter() {
            let failed_tx = failed_tx_loop.get_value_as_ref();

            all_payments.push(EsdtTokenPayment::new(
                failed_tx.token_id.clone(),
                failed_tx.nonce,
                failed_tx.amount.clone(),
            ));
        }
        self.send()
            .direct_multi(&self.multi_transfer_address().get(), &all_payments);

        result
    }
}
