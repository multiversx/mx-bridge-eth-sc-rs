#![no_std]

elrond_wasm::imports!();

const LEFTOVER_GAS: u64 = 10_000u64;

#[elrond_wasm::contract]
pub trait EgldEsdtSwap {
    #[init]
    fn init(&self, wrapped_egld_token_id: TokenIdentifier) {
        self.wrapped_egld_token_id().set(&wrapped_egld_token_id);
    }

    // endpoints

    #[only_owner]
    #[payable("EGLD")]
    #[endpoint]
    fn rebalance(&self) {}

    #[payable("EGLD")]
    #[endpoint(wrapEgld)]
    fn wrap_egld(
        &self,
        #[payment_token] payment_token: TokenIdentifier,
        #[payment_amount] payment_amount: BigUint,
        accept_funds_endpoint_name: OptionalValue<ManagedBuffer>,
    ) {
        require!(payment_token.is_egld(), "Only EGLD accepted");
        require!(payment_amount > 0u32, "Payment must be more than 0");

        let wrapped_egld_token_id = self.wrapped_egld_token_id().get();
        self.send()
            .esdt_local_mint(&wrapped_egld_token_id, 0, &payment_amount);

        let caller = self.blockchain().get_caller();
        let function = match accept_funds_endpoint_name {
            OptionalValue::Some(f) => f,
            OptionalValue::None => ManagedBuffer::new(),
        };

        if self.needs_execution(&caller, &function) {
            let gas_limit = self.blockchain().get_gas_left() - LEFTOVER_GAS;
            let _ = Self::Api::send_api_impl().direct_esdt_execute(
                &caller,
                &wrapped_egld_token_id,
                &payment_amount,
                gas_limit,
                &function,
                &ManagedArgBuffer::new_empty(),
            );
        } else {
            self.send()
                .direct(&caller, &wrapped_egld_token_id, 0, &payment_amount, &[]);
        }
    }

    #[payable("*")]
    #[endpoint(unwrapEgld)]
    fn unwrap_egld(
        &self,
        #[payment_token] payment_token: TokenIdentifier,
        #[payment_amount] payment_amount: BigUint,
        accept_funds_endpoint_name: OptionalValue<ManagedBuffer>,
    ) {
        let wrapped_egld_token_id = self.wrapped_egld_token_id().get();

        require!(payment_token == wrapped_egld_token_id, "Wrong esdt token");
        require!(payment_amount > 0u32, "Must pay more than 0 tokens!");
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
            OptionalValue::Some(f) => f,
            OptionalValue::None => ManagedBuffer::new(),
        };

        if self.needs_execution(&caller, &function) {
            let gas_limit = self.blockchain().get_gas_left() - LEFTOVER_GAS;
            let _ = Self::Api::send_api_impl().direct_egld_execute(
                &caller,
                &payment_amount,
                gas_limit,
                &function,
                &ManagedArgBuffer::new_empty(),
            );
        } else {
            self.send().direct_egld(&caller, &payment_amount, &[]);
        }
    }

    // views

    #[view(getLockedEgldBalance)]
    fn get_locked_egld_balance(&self) -> BigUint {
        self.blockchain()
            .get_sc_balance(&TokenIdentifier::egld(), 0)
    }

    // private

    fn needs_execution(&self, caller: &ManagedAddress, function: &ManagedBuffer) -> bool {
        self.blockchain().is_smart_contract(caller) && !function.is_empty()
    }

    // storage

    // 1 eGLD = 1 wrapped eGLD, and they are interchangeable through this contract

    #[view(getWrappedEgldTokenId)]
    #[storage_mapper("wrappedEgldTokenId")]
    fn wrapped_egld_token_id(&self) -> SingleValueMapper<TokenIdentifier>;
}
