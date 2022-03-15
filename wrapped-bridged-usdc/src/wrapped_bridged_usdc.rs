#![no_std]

elrond_wasm::imports!();

#[elrond_wasm::derive::contract]
pub trait WrappedBridgedUsdc {
    #[init]
    fn init(&self, usdc_token_id: TokenIdentifier, wrapped_usdc_token_id: TokenIdentifier) {
        self.usdc_token_id().set(&usdc_token_id);
        self.wrapped_usdc_token_id().set(&wrapped_usdc_token_id);
    }

    // endpoints

    #[payable("EGLD")]
    #[endpoint(wrapUSDC)]
    fn wrap_usdc(
        &self,
        #[payment_token] payment_token: TokenIdentifier,
        #[payment_amount] payment_amount: BigUint,
        #[var_args] accept_funds_endpoint_name: OptionalArg<ManagedBuffer>,
    ) {
        let usdc_token_id = self.usdc_token_id().get();
        let wrapped_usdc_token_id = self.wrapped_usdc_token_id().get();

        require!(payment_token == usdc_token_id, "Wrong esdt token");
        require!(payment_amount > 0u32, "Payment must be more than 0");

        self.send()
            .esdt_local_mint(&wrapped_usdc_token_id, 0, &payment_amount);

        let caller = self.blockchain().get_caller();
        let function = match accept_funds_endpoint_name {
            OptionalArg::Some(f) => f,
            OptionalArg::None => ManagedBuffer::new(),
        };

        if self.needs_execution(&caller, &function) {
            let gas_limit = self.blockchain().get_gas_left() - LEFTOVER_GAS;
            let _ = Self::Api::send_api_impl().direct_esdt_execute(
                &caller,
                &wrapped_usdc_token_id,
                &payment_amount,
                gas_limit,x`x`
                &function,
                &ManagedArgBuffer::new_empty(),
            );
        } else {
            self.send()
                .direct(&caller, &wrapped_usdc_token_id, 0, &payment_amount, &[]);
        }
    }

    #[payable("*")]
    #[endpoint(unwrapEgld)]
    fn unwrap_usdc(
        &self,
        #[payment_token] payment_token: TokenIdentifier,
        #[payment_amount] payment_amount: BigUint,
        #[var_args] accept_funds_endpoint_name: OptionalArg<ManagedBuffer>,
    ) {
        let usdc_token_id = self.usdc_token_id().get();
        let wrapped_usdc_token_id = self.wrapped_usdc_token_id().get();

        require!(payment_token == wrapped_usdc_token_id, "Wrong esdt token");
        require!(payment_amount > 0u32, "Must pay more than 0 tokens!");
        // this should never happen, but we'll check anyway
        require!(
            payment_amount <= self.get_locked_balance(&usdc_token_id),
            "Contract does not have enough funds"
        );

        self.send()
            .esdt_local_burn(&wrapped_usdc_token_id, 0, &payment_amount);

        // 1 wrapped eGLD = 1 eGLD, so we pay back the same amount
        let caller = self.blockchain().get_caller();
        let function = match accept_funds_endpoint_name {
            OptionalArg::Some(f) => f,
            OptionalArg::None => ManagedBuffer::new(),
        };

        if self.needs_execution(&caller, &function) {
            let gas_limit = self.blockchain().get_gas_left() - LEFTOVER_GAS;
            let _ = Self::Api::send_api_impl().direct_esdt_execute(
                &caller,
                &usdc_token_id,
                &payment_amount,
                gas_limit,
                &function,
                &ManagedArgBuffer::new_empty(),
            );
        } else {
            self.send()
                .direct(&caller, &usdc_token_id, 0, &payment_amount, &[]);
        }
    }

    // views

    #[view(getLockedUSDCBalance)]
    fn get_locked_usdc_balance(&self, usdc_token: &TokenIdentifier) -> BigUint {
        self.blockchain().get_sc_balance(usdc_token, 0)
    }

    // private

    fn needs_execution(&self, caller: &ManagedAddress, function: &ManagedBuffer) -> bool {
        self.blockchain().is_smart_contract(caller) && !function.is_empty()
    }

    // storage

    // 1 USDC = 1 wrapped USDC, and they are interchangeable through this contract

    #[view(getWrappedUSDCTokenId)]
    #[storage_mapper("wrappedUSDCTokenId")]
    fn wrapped_usdc_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getUSDCTokenId)]
    #[storage_mapper("USDCTokenId")]
    fn usdc_token_id(&self) -> SingleValueMapper<TokenIdentifier>;
}
