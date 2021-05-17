#![no_std]

elrond_wasm::imports!();

#[elrond_wasm_derive::contract]
pub trait EgldEsdtSwap {
    #[init]
    fn init(&self) {}

    // endpoints - owner-only

    #[endpoint(setWrappedEgldTokenId)]
    fn set_wrapped_egld_token_id(&self, token_id: TokenIdentifier) -> SCResult<()> {
        only_owner!(self, "only owner may call this function");
        require!(token_id.is_valid_esdt_identifier(), "Invalid token id");

        /* TODO: Uncomment on next elrond-wasm version
        let roles = self
            .blockchain()
            .get_esdt_local_roles(token_id.as_esdt_identifier());
        require!(
            roles.contains(&EsdtLocalRole::Mint) && roles.contains(&EsdtLocalRole::Burn),
            "Must set local roles first"
        );
        */

        self.wrapped_egld_token_id().set(&token_id);

        Ok(())
    }

    // endpoints

    #[payable("EGLD")]
    #[endpoint(wrapEgld)]
    fn wrap_egld(&self, #[payment] payment: Self::BigUint) -> SCResult<()> {
        require!(payment > 0, "Payment must be more than 0");
        require!(
            !self.wrapped_egld_token_id().is_empty(),
            "Wrapped eGLD was not issued yet"
        );

        let wrapped_egld_token_id = self.wrapped_egld_token_id().get();
        self.send().esdt_local_mint(
            self.blockchain().get_gas_left(),
            wrapped_egld_token_id.as_esdt_identifier(),
            &payment,
        );

        let caller = self.blockchain().get_caller();
        match self.send().direct_esdt_via_transf_exec(
            &caller,
            wrapped_egld_token_id.as_esdt_identifier(),
            &payment,
            self.data_or_empty(&caller, b"wrapping"),
        ) {
            Result::Ok(()) => Ok(()),
            Result::Err(_) => sc_error!("Wrapping failed"),
        }
    }

    #[payable("*")]
    #[endpoint(unwrapEgld)]
    fn unwrap_egld(
        &self,
        #[payment] payment: Self::BigUint,
        #[payment_token] token_id: TokenIdentifier,
    ) -> SCResult<()> {
        let wrapped_egld_token_id = self.wrapped_egld_token_id().get();

        require!(
            !self.wrapped_egld_token_id().is_empty(),
            "Wrapped eGLD was not issued yet"
        );
        require!(token_id.is_esdt(), "Only ESDT tokens accepted");
        require!(token_id == wrapped_egld_token_id, "Wrong esdt token");
        require!(payment > 0, "Must pay more than 0 tokens!");
        // this should never happen, but we'll check anyway
        require!(
            payment <= self.blockchain().get_sc_balance(),
            "Contract does not have enough funds"
        );

        self.send().esdt_local_burn(
            self.blockchain().get_gas_left(),
            wrapped_egld_token_id.as_esdt_identifier(),
            &payment,
        );

        // 1 wrapped eGLD = 1 eGLD, so we pay back the same amount
        let caller = self.blockchain().get_caller();
        self.send().direct_egld(
            &caller,
            &payment,
            self.data_or_empty(&caller, b"unwrapping"),
        );

        Ok(())
    }

    // views

    #[view(getLockedEgldBalance)]
    fn get_locked_egld_balance(&self) -> Self::BigUint {
        self.blockchain().get_sc_balance()
    }

    #[view(getWrappedEgldRemaining)]
    fn get_wrapped_egld_remaining(&self) -> Self::BigUint {
        self.blockchain().get_esdt_balance(
            &self.blockchain().get_sc_address(),
            self.wrapped_egld_token_id().get().as_esdt_identifier(),
            0,
        )
    }

    // private

    fn data_or_empty(&self, to: &Address, data: &'static [u8]) -> &[u8] {
        if self.blockchain().is_smart_contract(to) {
            &[]
        } else {
            data
        }
    }

    // storage

    // 1 eGLD = 1 wrapped eGLD, and they are interchangeable through this contract

    #[view(getWrappedEgldTokenId)]
    #[storage_mapper("wrappedEgldTokenId")]
    fn wrapped_egld_token_id(&self) -> SingleValueMapper<Self::Storage, TokenIdentifier>;
}
