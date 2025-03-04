#![no_std]

use multiversx_sc::api::ManagedTypeApi;
use multiversx_sc::derive_imports::*;
use multiversx_sc::imports::*;

#[type_abi]
#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, Clone, ManagedVecItem)]
pub struct CalledData<M: ManagedTypeApi> {
    pub size: u64,
    pub address: ManagedAddress<M>,
    pub token_identifier: TokenIdentifier<M>,
    pub buff: ManagedBuffer<M>,
}

#[multiversx_sc::contract]
pub trait TestCallerContract {
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}

    #[only_owner]
    #[endpoint(withdrawToken)]
    fn withdraw_token(&self, token: TokenIdentifier) {
        let balance = self.blockchain().get_sc_balance(&EgldOrEsdtTokenIdentifier::esdt(token.clone()), 0);
        let owner = self.blockchain().get_owner_address();
        self.tx()
            .to(owner)
            .single_esdt(&token, 0, &balance)
            .transfer();
    }

    #[payable("*")]
    #[endpoint(callPayable)]
    fn call_payable(&self) {}

    #[endpoint(callNonPayable)]
    fn call_non_payable(&self) {}

    #[payable("*")]
    #[endpoint(callPayableWithParams)]
    fn call_payable_with_params(&self, size: u64, address: ManagedAddress) {
        let payment = self.call_value().single_esdt();
        let token_identifier = payment.token_identifier.clone();

        let data = CalledData {
            size,
            address,
            token_identifier,
            buff: ManagedBuffer::new(),
        };

        _ = self.called_data_params().push(&data);
    }

    #[payable("*")]
    #[endpoint(callPayableWithBuff)]
    fn call_payable_with_buff(&self, buff: ManagedBuffer) {
        let payment = self.call_value().single_esdt();
        let token_identifier = payment.token_identifier.clone();

        let data = CalledData {
            size: 0,
            address: ManagedAddress::zero(),
            token_identifier,
            buff,
        };

        _ = self.called_data_params().push(&data);
    }

    #[view(getCalledDataParams)]
    fn get_called_data_params(&self) -> MultiValueEncoded<CalledData<Self::Api>> {
        let mut values = MultiValueEncoded::new();
        let len = self.called_data_params().len();

        for i in 1..=len {
            if self.called_data_params().item_is_empty(i) {
                continue;
            }
            let value = self.called_data_params().get_unchecked(i);
            values.push(value);
        }
        values
    }

    #[storage_mapper("calledDataParams")]
    fn called_data_params(&self) -> VecMapper<CalledData<Self::Api>>;
}
