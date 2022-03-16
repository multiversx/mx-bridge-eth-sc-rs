#![no_std]

elrond_wasm::imports!();

#[elrond_wasm::contract]
pub trait WrappedBridgedUsdc {
    #[init]
    fn init(&self, universal_bridged_usdc_token_id: TokenIdentifier) {
        self.universal_bridged_usdc_token_id()
            .set(universal_bridged_usdc_token_id);
    }

    #[only_owner]
    #[endpoint(whitelistUsdc)]
    fn whitelist_usdc(&self, chain_specific_usdc_token_id: TokenIdentifier) {
        self.chain_specific_usdc_token_ids()
            .insert(chain_specific_usdc_token_id);
    }

    #[only_owner]
    #[endpoint(blacklistUsdc)]
    fn blacklist_usdc(&self, chain_specific_usdc_token_id: TokenIdentifier) {
        self.chain_specific_usdc_token_ids()
            .swap_remove(&chain_specific_usdc_token_id);
    }

    // endpoints

    #[payable("*")]
    #[endpoint(wrapUsdc)]
    fn wrap_usdc(
        &self,
        #[payment_token] payment_token: TokenIdentifier,
        #[payment_amount] payment_amount: BigUint,
    ) {
        let chain_specific_usdc_token_id = &payment_token;
        require!(
            self.chain_specific_usdc_token_ids()
                .contains(chain_specific_usdc_token_id),
            "Wrong esdt token"
        );

        let universal_bridged_usdc_token_id = self.universal_bridged_usdc_token_id().get();
        require!(payment_amount > 0u32, "Payment must be more than 0");

        self.send()
            .esdt_local_mint(&universal_bridged_usdc_token_id, 0, &payment_amount);

        let caller = self.blockchain().get_caller();

        self.send().direct(
            &caller,
            &universal_bridged_usdc_token_id,
            0,
            &payment_amount,
            &[],
        );
    }

    #[payable("*")]
    #[endpoint(unwrapUsdc)]
    fn unwrap_usdc(
        &self,
        #[payment_token] payment_token: TokenIdentifier,
        #[payment_amount] payment_amount: BigUint,
        requested_token: TokenIdentifier,
    ) {
        let chain_specific_usdc_token_id = &requested_token;
        require!(
            self.chain_specific_usdc_token_ids()
                .contains(chain_specific_usdc_token_id),
            "Esdt token unavailable"
        );

        let universal_bridged_usdc_token_id = self.universal_bridged_usdc_token_id().get();
        require!(
            payment_token == universal_bridged_usdc_token_id,
            "Wrong esdt token"
        );

        require!(payment_amount > 0u32, "Must pay more than 0 tokens!");
        require!(
            payment_amount <= self.get_liquidity(chain_specific_usdc_token_id),
            "Contract does not have enough funds"
        );

        self.send()
            .esdt_local_burn(&universal_bridged_usdc_token_id, 0, &payment_amount);

        // 1 wrapped USDC = 1 USDC, so we pay back the same amount
        let caller = self.blockchain().get_caller();

        self.send().direct(
            &caller,
            chain_specific_usdc_token_id,
            0,
            &payment_amount,
            &[],
        );
    }

    // views

    #[view(getLiquidity)]
    fn get_liquidity(&self, bridged_usdc_token: &TokenIdentifier) -> BigUint {
        self.blockchain().get_sc_balance(bridged_usdc_token, 0)
    }

    // storage

    // 1 USDC = 1 wrapped USDC, and they are interchangeable through this contract

    #[view(getWrappedUsdcTokenId)]
    #[storage_mapper("universalUsdcTokenId")]
    fn universal_bridged_usdc_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getChainSpecificUsdcTokenIds)]
    #[storage_mapper("chainSpecificUsdcTokenIds")]
    fn chain_specific_usdc_token_ids(&self) -> UnorderedSetMapper<TokenIdentifier>;
}
