use multiversx_sc::imports::*;

use transaction::EthTransaction;

#[multiversx_sc::module]
pub trait ConfigModule {
    #[only_owner]
    #[endpoint(setupMultiTransfer)]
    fn set_multi_transfer_contract_address(
        &self,
        opt_multi_transfer_address: OptionalValue<ManagedAddress>,
    ) {
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

    #[only_owner]
    #[endpoint(setBridgedTokensWrapper)]
    fn set_bridged_tokens_wrapper(&self, opt_address: OptionalValue<ManagedAddress>) {
        match opt_address {
            OptionalValue::Some(sc_addr) => {
                require!(
                    self.blockchain().is_smart_contract(&sc_addr),
                    "Invalid bridged tokens wrapper address"
                );
                self.bridged_tokens_wrapper_address().set(&sc_addr);
            }
            OptionalValue::None => self.bridged_tokens_wrapper_address().clear(),
        }
    }

    #[view(getMultiTransferAddress)]
    #[storage_mapper("multiTransferAddress")]
    fn multi_transfer_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getBridgedTokensWrapperAddress)]
    #[storage_mapper("bridgedTokensWrapperAddress")]
    fn bridged_tokens_wrapper_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("pending_transactions")]
    fn pending_transactions(&self) -> VecMapper<EthTransaction<Self::Api>>;

    #[storage_mapper("payments")]
    fn payments(&self, tx_id: usize) -> SingleValueMapper<EsdtTokenPayment<Self::Api>>;

    #[storage_mapper("lowest_tx_id")]
    fn lowest_tx_id(&self) -> SingleValueMapper<usize>;
}
