#![no_std]

elrond_wasm::imports!();

#[elrond_wasm_derive::contract]
pub trait EgldEsdtSwap {
    #[init]
    fn init(&self, wrapped_egld_token_id: TokenIdentifier) -> SCResult<()> {
        require!(
            wrapped_egld_token_id.is_valid_esdt_identifier(),
            "Invalid token id"
        );

        self.wrapped_egld_token_id().set(&wrapped_egld_token_id);

        Ok(())
    }

    // endpoints

    #[payable("EGLD")]
    #[endpoint(wrapEgld)]
    fn wrap_egld(
        &self,
        #[payment] payment: Self::BigUint,
        #[var_args] accept_funds_endpoint_name: OptionalArg<BoxedBytes>,
    ) -> SCResult<OptionalResult<AsyncCall<Self::SendApi>>> {
        require!(payment > 0, "Payment must be more than 0");

        let wrapped_egld_token_id = self.wrapped_egld_token_id().get();

        self.require_local_role_set(&wrapped_egld_token_id, &EsdtLocalRole::Mint)?;
        self.send()
            .esdt_local_mint(&wrapped_egld_token_id, &payment);

        let caller = self.blockchain().get_caller();
        let function = accept_funds_endpoint_name
            .into_option()
            .unwrap_or_else(BoxedBytes::empty);

        if self.needs_execution(&caller, &function) {
            Ok(OptionalResult::Some(self.transfer_and_execute_via_async(
                &caller,
                &wrapped_egld_token_id,
                &payment,
                &function,
            )))
        } else {
            self.send()
                .direct(&caller, &wrapped_egld_token_id, &payment, b"wrapping");
            Ok(OptionalResult::None)
        }
    }

    #[payable("*")]
    #[endpoint(unwrapEgld)]
    fn unwrap_egld(
        &self,
        #[payment] payment: Self::BigUint,
        #[payment_token] token_id: TokenIdentifier,
        #[var_args] accept_funds_endpoint_name: OptionalArg<BoxedBytes>,
    ) -> SCResult<OptionalResult<AsyncCall<Self::SendApi>>> {
        let wrapped_egld_token_id = self.wrapped_egld_token_id().get();

        require!(token_id.is_esdt(), "Only ESDT tokens accepted");
        require!(token_id == wrapped_egld_token_id, "Wrong esdt token");
        require!(payment > 0, "Must pay more than 0 tokens!");
        // this should never happen, but we'll check anyway
        require!(
            payment <= self.blockchain().get_sc_balance(),
            "Contract does not have enough funds"
        );

        self.require_local_role_set(&wrapped_egld_token_id, &EsdtLocalRole::Burn)?;
        self.send()
            .esdt_local_burn(&wrapped_egld_token_id, &payment);

        // 1 wrapped eGLD = 1 eGLD, so we pay back the same amount
        let caller = self.blockchain().get_caller();
        let function = accept_funds_endpoint_name
            .into_option()
            .unwrap_or_else(BoxedBytes::empty);

        if self.needs_execution(&caller, &function) {
            Ok(OptionalResult::Some(self.transfer_and_execute_via_async(
                &caller,
                &TokenIdentifier::egld(),
                &payment,
                &function,
            )))
        } else {
            self.send()
                .direct(&caller, &TokenIdentifier::egld(), &payment, b"unwrapping");
            Ok(OptionalResult::None)
        }
    }

    // views

    #[view(getLockedEgldBalance)]
    fn get_locked_egld_balance(&self) -> Self::BigUint {
        self.blockchain().get_sc_balance()
    }

    // private

    fn needs_execution(&self, caller: &Address, function: &BoxedBytes) -> bool {
        self.blockchain().is_smart_contract(caller) && !function.is_empty()
    }

    fn transfer_and_execute_via_async(
        &self,
        caller: &Address,
        token_id: &TokenIdentifier,
        amount: &Self::BigUint,
        function: &BoxedBytes,
    ) -> AsyncCall<Self::SendApi> {
        let contract_call: ContractCall<Self::SendApi, ()> =
            ContractCall::new(self.send(), caller.clone(), function.clone())
                .with_token_transfer(token_id.clone(), amount.clone());

        contract_call
            .async_call()
            .with_callback(self.callbacks().async_transfer_execute_callback(caller))
    }

    fn require_local_role_set(
        &self,
        token_id: &TokenIdentifier,
        role: &EsdtLocalRole,
    ) -> SCResult<()> {
        let roles = self.blockchain().get_esdt_local_roles(token_id);
        require!(roles.contains(role), "Must set local role first");

        Ok(())
    }

    // callbacks

    #[callback]
    fn async_transfer_execute_callback(
        &self,
        caller: &Address,
        #[call_result] result: AsyncCallResult<()>,
    ) {
        match result {
            AsyncCallResult::Ok(_) => {}
            AsyncCallResult::Err(_) => {
                let (returned_tokens, token_identifier) = self.call_value().payment_token_pair();
                if returned_tokens > 0 {
                    self.send()
                        .direct(caller, &token_identifier, &returned_tokens, &[]);
                }
            }
        }
    }

    // storage

    // 1 eGLD = 1 wrapped eGLD, and they are interchangeable through this contract

    #[view(getWrappedEgldTokenId)]
    #[storage_mapper("wrappedEgldTokenId")]
    fn wrapped_egld_token_id(&self) -> SingleValueMapper<Self::Storage, TokenIdentifier>;
}
