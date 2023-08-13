multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use transaction::EthTransactionPayment;

#[multiversx_sc::module]
pub trait ConfigModule {
    #[only_owner]
    #[endpoint(setupMultiTransfer)]
    fn set_multi_transfer_contract_address(&self, opt_multi_transfer_address: OptionalValue<ManagedAddress>) {
        match opt_multi_transfer_address {
            OptionalValue::Some(sc_addr) => {
                require!(
                    self.blockchain().is_smart_contract(&sc_addr),
                    "Invalid multi-transfer address"
                );
                self.multi_transfer_address().set(&sc_addr);
            }
            OptionalValue::None => self.multi_transfer_address().clear(),
        }
    }

    #[view(getEthTransactionById)]
    fn get_eth_transaction_by_id(&self, id: u32) -> ManagedBuffer<Self::Api> {
        let eth_tx_list = self.eth_transaction_list();
        match eth_tx_list.get_node_by_id(id) {
            Some(tx) => tx.get_value_cloned().eth_tx.data,
            None => sc_panic!("No transaction with this id!")
        }
    }

    #[view(getMultiTransferAddress)]
    #[storage_mapper("multiTransferAddress")]
    fn multi_transfer_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getEthTransactionList)]
    #[storage_mapper("eth_transaction_list")]
    fn eth_transaction_list(&self) -> LinkedListMapper<EthTransactionPayment<Self::Api>>;

    #[view(getEthFailedTransactionList)]
    #[storage_mapper("eth_failed_transaction_list")]
    fn eth_failed_transaction_list(&self) -> LinkedListMapper<EthTransactionPayment<Self::Api>>;
}
