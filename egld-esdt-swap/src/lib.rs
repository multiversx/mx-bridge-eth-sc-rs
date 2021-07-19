#![no_std]

elrond_wasm::imports!();

const GAS_FOR_CALLBACK: u64 = 500_000u64;

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
            self.require_has_enough_gas_for_transfer_via_async()?;
            Ok(OptionalResult::Some(self.transfer_via_async(
                caller,
                wrapped_egld_token_id,
                payment,
                function,
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
        let egld_token_id = TokenIdentifier::egld();

        if self.needs_execution(&caller, &function) {
            self.require_has_enough_gas_for_transfer_via_async()?;
            Ok(OptionalResult::Some(self.transfer_via_async(
                caller,
                egld_token_id,
                payment,
                function,
            )))
        } else {
            self.send()
                .direct(&caller, &egld_token_id, &payment, b"unwrapping");
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

    fn require_has_enough_gas_for_transfer_via_async(&self) -> SCResult<()> {
        let gas_needed_for_callback = GAS_FOR_CALLBACK;
        let gas_leftover_required = gas_needed_for_callback;
        let total_gas_needed=  gas_needed_for_callback + gas_leftover_required;

        require!(self.blockchain().get_gas_left() > total_gas_needed, "Not enough gas");
        Ok(())
    }

    fn transfer_via_async(
        &self,
        caller: Address,
        token_id: TokenIdentifier,
        amount: Self::BigUint,
        function: BoxedBytes,
    ) -> AsyncCall<Self::SendApi> {
        let gas_limit = self.blockchain().get_gas_left() - GAS_FOR_CALLBACK;

        let contract_call: ContractCall<Self::SendApi, ()> =
            ContractCall::new(self.send(), caller.clone(), function)
                .with_token_transfer(token_id, amount)
                .with_gas_limit(gas_limit);

        contract_call
            .async_call()
            .with_callback(self.callbacks().transfer_via_async_callback(caller))
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

    fn revert_operation_and_send(
        &self,
        address: Address,
        returned_token_id: TokenIdentifier,
        returned_amount: Self::BigUint,
    ) {
        let egld_token_id = TokenIdentifier::egld();
        let wrapped_egld_token_id = self.wrapped_egld_token_id().get();

        if returned_token_id == egld_token_id {
            self.send()
                .esdt_local_mint(&wrapped_egld_token_id, &returned_amount);
            self.send()
                .direct(&address, &wrapped_egld_token_id, &returned_amount, &[]);
        } else {
            self.send()
                .esdt_local_burn(&wrapped_egld_token_id, &returned_amount);
            self.send()
                .direct(&address, &egld_token_id, &returned_amount, &[]);
        }
    }

    // callbacks

    #[callback]
    fn transfer_via_async_callback(
        &self,
        caller: Address,
        #[call_result] result: AsyncCallResult<()>,
    ) {
        match result {
            AsyncCallResult::Ok(_) => {}
            AsyncCallResult::Err(_) => {
                let (returned_tokens, token_identifier) = self.call_value().payment_token_pair();
                if returned_tokens != 0 {
                    self.revert_operation_and_send(caller, token_identifier, returned_tokens);
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
