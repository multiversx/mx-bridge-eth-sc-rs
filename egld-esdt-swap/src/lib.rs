#![no_std]

elrond_wasm::imports!();
const DEFAULT_GAS_LEFTOVER: u64 = 100_000;

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
    ) -> SCResult<()> {
        require!(payment > 0, "Payment must be more than 0");

        let wrapped_egld_token_id = self.wrapped_egld_token_id().get();

        self.require_local_role_set(&wrapped_egld_token_id, &EsdtLocalRole::Mint)?;
        self.send()
            .esdt_local_mint(&wrapped_egld_token_id, &payment);

        let caller = self.blockchain().get_caller();
        let function = accept_funds_endpoint_name.into_option().unwrap_or(b""[..].into());
        let gas_limit = self.blockchain().get_gas_left() - DEFAULT_GAS_LEFTOVER;

        SCResult::from_result(self.send().direct_esdt_execute(
            &caller,
            &wrapped_egld_token_id,
            &payment,
            gas_limit,
            &function.as_slice(),
            &ArgBuffer::new(),
        ))
    }

    #[payable("*")]
    #[endpoint(unwrapEgld)]
    fn unwrap_egld(
        &self,
        #[payment] payment: Self::BigUint,
        #[payment_token] token_id: TokenIdentifier,
        #[var_args] accept_funds_endpoint_name: OptionalArg<BoxedBytes>,
    ) -> SCResult<()> {
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
        let function = accept_funds_endpoint_name.into_option().unwrap_or(b""[..].into());
        let gas_limit = self.blockchain().get_gas_left() - DEFAULT_GAS_LEFTOVER;

        SCResult::from_result(self.send().direct_egld_execute(
            &caller,
            &payment,
            gas_limit,
            &function.as_slice(),
            &ArgBuffer::new(),
        ))
    }

    // views

    #[view(getLockedEgldBalance)]
    fn get_locked_egld_balance(&self) -> Self::BigUint {
        self.blockchain().get_sc_balance()
    }

    // private

    fn data_or_empty(&self, to: &Address, data: &'static [u8]) -> &[u8] {
        if self.blockchain().is_smart_contract(to) {
            &[]
        } else {
            data
        }
    }

    fn require_local_role_set(&self, token_id: &TokenIdentifier, role: &EsdtLocalRole) -> SCResult<()> {
        let roles = self.blockchain().get_esdt_local_roles(token_id);
        require!(
            roles.contains(role),
            "Must set local role first"
        );

        Ok(())
    }

    // storage

    // 1 eGLD = 1 wrapped eGLD, and they are interchangeable through this contract

    #[view(getWrappedEgldTokenId)]
    #[storage_mapper("wrappedEgldTokenId")]
    fn wrapped_egld_token_id(&self) -> SingleValueMapper<Self::Storage, TokenIdentifier>;
}
