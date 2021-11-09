#![no_std]

elrond_wasm::imports!();

const LEFTOVER_GAS: u64 = 10_000u64;

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
        #[payment_token] payment_token: TokenIdentifier,
        #[payment_amount] payment_amount: Self::BigUint,
        #[var_args] accept_funds_endpoint_name: OptionalArg<BoxedBytes>,
    ) -> SCResult<()> {
        require!(payment_token.is_egld(), "Only EGLD accepted");
        require!(payment_amount > 0, "Payment must be more than 0");

        let wrapped_egld_token_id = self.wrapped_egld_token_id().get();
        self.send()
            .esdt_local_mint(&wrapped_egld_token_id, 0, &payment_amount);

        let caller = self.blockchain().get_caller();
        let function = match accept_funds_endpoint_name {
            OptionalArg::Some(f) => f,
            OptionalArg::None => BoxedBytes::empty(),
        };

        if self.needs_execution(&caller, &function) {
            let gas_limit = self.blockchain().get_gas_left() - LEFTOVER_GAS;
            self.send()
                .direct_esdt_execute(
                    &caller,
                    &wrapped_egld_token_id,
                    &payment_amount,
                    gas_limit,
                    function.as_slice(),
                    &ArgBuffer::new(),
                )
                .into()
        } else {
            self.send()
                .direct(&caller, &wrapped_egld_token_id, 0, &payment_amount, &[]);
            Ok(())
        }
    }

    #[payable("*")]
    #[endpoint(unwrapEgld)]
    fn unwrap_egld(
        &self,
        #[payment_token] payment_token: TokenIdentifier,
        #[payment_amount] payment_amount: Self::BigUint,
        #[var_args] accept_funds_endpoint_name: OptionalArg<BoxedBytes>,
    ) -> SCResult<()> {
        let wrapped_egld_token_id = self.wrapped_egld_token_id().get();

        require!(payment_token == wrapped_egld_token_id, "Wrong esdt token");
        require!(payment_amount > 0, "Must pay more than 0 tokens!");
        // this should never happen, but we'll check anyway
        require!(
            payment_amount <= self.get_locked_egld_balance(),
            "Contract does not have enough funds"
        );

        self.send()
            .esdt_local_burn(&wrapped_egld_token_id, 0, &payment_amount);

        // 1 wrapped eGLD = 1 eGLD, so we pay back the same amount
        let caller = self.blockchain().get_caller();
        let function = match accept_funds_endpoint_name {
            OptionalArg::Some(f) => f,
            OptionalArg::None => BoxedBytes::empty(),
        };

        if self.needs_execution(&caller, &function) {
            let gas_limit = self.blockchain().get_gas_left() - LEFTOVER_GAS;
            self.send()
                .direct_egld_execute(
                    &caller,
                    &payment_amount,
                    gas_limit,
                    function.as_slice(),
                    &ArgBuffer::new(),
                )
                .into()
        } else {
            self.send().direct_egld(&caller, &payment_amount, &[]);
            Ok(())
        }
    }

    // views

    #[view(getLockedEgldBalance)]
    fn get_locked_egld_balance(&self) -> Self::BigUint {
        self.blockchain()
            .get_sc_balance(&TokenIdentifier::egld(), 0)
    }

    // private

    fn needs_execution(&self, caller: &Address, function: &BoxedBytes) -> bool {
        self.blockchain().is_smart_contract(caller) && !function.is_empty()
    }

    // storage

    // 1 eGLD = 1 wrapped eGLD, and they are interchangeable through this contract

    #[view(getWrappedEgldTokenId)]
    #[storage_mapper("wrappedEgldTokenId")]
    fn wrapped_egld_token_id(&self) -> SingleValueMapper<Self::Storage, TokenIdentifier>;
}
