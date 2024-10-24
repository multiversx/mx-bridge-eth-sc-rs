use multiversx_sc::imports::*;

use transaction::EthTransaction;

#[multiversx_sc::module]
pub trait ConfigModule {
    #[storage_mapper("ownerAddress")]
    fn owner_address_storage(&self) -> SingleValueMapper<ManagedAddress<Self::Api>>;

    #[storage_mapper("pending_transactions")]
    fn pending_transactions(&self) -> VecMapper<EthTransaction<Self::Api>>;

    #[storage_mapper("payments")]
    fn payments(&self, tx_id: usize) -> SingleValueMapper<EsdtTokenPayment<Self::Api>>;

    #[storage_mapper("batch_id")]
    fn batch_id(&self, tx_id: usize) -> SingleValueMapper<u64>;

    #[view(lowestTxId)]
    #[storage_mapper("lowest_tx_id")]
    fn lowest_tx_id(&self) -> SingleValueMapper<usize>;

    #[storage_mapper("ongoingExecution")]
    fn ongoing_execution(&self, tx_id: usize) -> SingleValueMapper<u64>;
}
