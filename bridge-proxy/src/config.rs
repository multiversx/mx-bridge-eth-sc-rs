multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use transaction::EthTransaction;

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, Clone)]
pub struct EthTransactionPayment<M: ManagedTypeApi> {
    pub token_id: TokenIdentifier<M>,
    pub nonce: u64,
    pub amount: BigUint<M>,
    pub eth_tx: EthTransaction<M>,
}

#[multiversx_sc::module]
pub trait ConfigModule {
    #[only_owner]
    #[endpoint(setupMultiTransfer)]
    fn setup_multi_transfer(&self, multi_transfer_address: ManagedAddress) {
        require!(
            self.blockchain().is_smart_contract(&multi_transfer_address),
            "Invalid multi-transfer address"
        );

        self.multi_transfer_address().set(&multi_transfer_address);
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
