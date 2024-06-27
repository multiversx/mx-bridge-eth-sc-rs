multiversx_sc::imports!();
multiversx_sc::derive_imports!();

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
    #[endpoint(setupEsdtSafe)]
    fn set_esdt_safe_contract_address(
        &self,
        opt_esdt_safe_address: OptionalValue<ManagedAddress>,
    ) {
        match opt_esdt_safe_address {
            OptionalValue::Some(sc_addr) => {
                require!(
                    self.blockchain().is_smart_contract(&sc_addr),
                    "Invalid multi-transfer address"
                );
                self.esdt_safe_address().set(&sc_addr);
            }
            OptionalValue::None => self.esdt_safe_address().clear(),
        }
    }

    fn get_esdt_safe_proxy_instance(&self) -> esdt_safe::Proxy<Self::Api> {
        self.esdt_safe_proxy(self.esdt_safe_address().get())
    }

    #[view(getMultiTransferAddress)]
    #[storage_mapper("multiTransferAddress")]
    fn multi_transfer_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getEsdtSafeAddress)]
    #[storage_mapper("esdtSafeAddress")]
    fn esdt_safe_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[proxy]
    fn esdt_safe_proxy(&self, sc_address: ManagedAddress) -> esdt_safe::Proxy<Self::Api>;

    #[view(getPendingTransactions)]
    #[storage_mapper("pending_transactions")]
    fn pending_transactions(&self) -> VecMapper<EthTransaction<Self::Api>>;
}
