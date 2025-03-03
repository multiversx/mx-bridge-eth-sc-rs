use multiversx_sc::imports::*;

use transaction::EthTransaction;

#[multiversx_sc::module]
pub trait ConfigModule {
    #[storage_mapper("pending_transactions")]
    fn pending_transactions(&self) -> MapMapper<usize, EthTransaction<Self::Api>>;

    #[view(refundTransactions)]
    #[storage_mapper("refundTransactions")]
    fn refund_transactions(&self) -> MapMapper<usize, EthTransaction<Self::Api>>;

    #[storage_mapper("payments")]
    fn payments(&self, tx_id: usize) -> SingleValueMapper<EsdtTokenPayment<Self::Api>>;

    #[storage_mapper("batch_id")]
    fn batch_id(&self, tx_id: usize) -> SingleValueMapper<u64>;

    #[view(highestTxId)]
    #[storage_mapper("highest_tx_id")]
    fn highest_tx_id(&self) -> SingleValueMapper<usize>;

    #[storage_mapper("ongoingExecution")]
    fn ongoing_execution(&self, tx_id: usize) -> SingleValueMapper<u64>;
}
