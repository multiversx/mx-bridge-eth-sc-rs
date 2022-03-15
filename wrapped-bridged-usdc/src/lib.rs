#![no_std]

elrond_wasm::imports!();

#[elrond_wasm::contract]
pub trait WrappedBridgedUsdc {
    #[init]
    fn init(&self, wrapped_usdc_token_id: TokenIdentifier) {
        self.wrapped_usdc_token_id().set(wrapped_usdc_token_id);
    }

    #[only_owner]
    #[endpoint(whitelistUsdc)]
    fn whitelist_usdc(&self, usdc_token_id: TokenIdentifier) {
        self.usdc_token_ids().insert(usdc_token_id);
    }

    #[only_owner]
    #[endpoint(blacklistUsdc)]
    fn blacklist_usdc(&self, usdc_token_id: TokenIdentifier) {
        self.usdc_token_ids().swap_remove(&usdc_token_id);
    }

    // endpoints

    #[payable("*")]
    #[endpoint(wrapUsdc)]
    fn wrap_usdc(
        &self,
        #[payment_token] payment_token: TokenIdentifier,
        #[payment_amount] payment_amount: BigUint,
    ) {
        let usdc_token_id = &payment_token;
        require!(
            self.usdc_token_ids().contains(usdc_token_id),
            "Wrong esdt token"
        );

        let wrapped_usdc_token_id = self.wrapped_usdc_token_id().get();
        require!(payment_amount > 0u32, "Payment must be more than 0");

        self.send()
            .esdt_local_mint(&wrapped_usdc_token_id, 0, &payment_amount);

        let caller = self.blockchain().get_caller();

        self.send()
            .direct(&caller, &wrapped_usdc_token_id, 0, &payment_amount, &[]);
    }

    #[payable("*")]
    #[endpoint(unwrapUsdc)]
    fn unwrap_usdc(
        &self,
        #[payment_token] payment_token: TokenIdentifier,
        #[payment_amount] payment_amount: BigUint,
        requested_token: TokenIdentifier,
    ) {
        let usdc_token_id = &requested_token;
        require!(
            self.usdc_token_ids().contains(usdc_token_id),
            "Esdt token unavailable"
        );

        let wrapped_usdc_token_id = self.wrapped_usdc_token_id().get();
        require!(payment_token == wrapped_usdc_token_id, "Wrong esdt token");

        require!(payment_amount > 0u32, "Must pay more than 0 tokens!");
        // this should never happen, but we'll check anyway
        require!(
            payment_amount <= self.get_liquidity(usdc_token_id),
            "Contract does not have enough funds"
        );

        self.send()
            .esdt_local_burn(&wrapped_usdc_token_id, 0, &payment_amount);

        // 1 wrapped USDC = 1 USDC, so we pay back the same amount
        let caller = self.blockchain().get_caller();

        self.send()
            .direct(&caller, usdc_token_id, 0, &payment_amount, &[]);
    }

    // views

    #[view(getLiquidity)]
    fn get_liquidity(&self, usdc_token: &TokenIdentifier) -> BigUint {
        self.blockchain().get_sc_balance(usdc_token, 0)
    }

    // private

    fn needs_execution(&self, caller: &ManagedAddress, function: &ManagedBuffer) -> bool {
        self.blockchain().is_smart_contract(caller) && !function.is_empty()
    }

    // storage

    // 1 USDC = 1 wrapped USDC, and they are interchangeable through this contract

    #[view(getWrappedUsdcTokenId)]
    #[storage_mapper("wrappedUSDCTokenId")]
    fn wrapped_usdc_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getUsdcTokenIds)]
    #[storage_mapper("USDCTokenIds")]
    fn usdc_token_ids(&self) -> UnorderedSetMapper<TokenIdentifier>;
}
