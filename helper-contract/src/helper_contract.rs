#![no_std]

#[allow(unused_imports)]
use multiversx_sc::imports::*;

use transaction::{EthTransaction, EthTxAsMultiValue};
use sc_proxies::bridge_proxy_contract_proxy;

#[multiversx_sc::contract]
pub trait HelperContract {
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}

    #[view(getCalleeAddress)]
    #[storage_mapper("callee_address")]
    fn callee_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[endpoint(setCallee)]
    fn set_callee(&self, address: ManagedAddress) {
        self.callee_address().set(&address);
    }

    #[endpoint(callDeposit)]
    #[payable("*")]
    fn call_deposit(&self, batch_id: u64, eth_tx_multivalue: EthTxAsMultiValue<Self::Api>) {
        let callee = self.callee_address().get();

        let (payment_token, payment_amount) = self.call_value().single_fungible_esdt();

        let (from, to, token_id, amount, tx_nonce, call_data) = eth_tx_multivalue.into_tuple();

        let eth_tx = EthTransaction {
            from,
            to,
            token_id,
            amount,
            tx_nonce,
            call_data,
        };

        self.tx()
            .to(callee)
            .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
            .deposit(eth_tx, batch_id)
            .single_esdt(&payment_token, 0, &payment_amount)
            .sync_call();
    }
}
