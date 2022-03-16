#![no_std]

use transaction::PaymentsVec;

elrond_wasm::imports!();

#[elrond_wasm::contract]
pub trait WrappedBridgedUsdc {
    #[init]
    fn init(
        &self,
        universal_bridged_usdc_token_id: TokenIdentifier,
        #[var_args] chain_specific_tokens: MultiValueEncoded<TokenIdentifier>,
    ) {
        self.universal_bridged_usdc_token_id()
            .set(universal_bridged_usdc_token_id);

        for token_id in chain_specific_tokens {
            let _ = self.chain_specific_usdc_token_ids().insert(token_id);
        }
    }

    #[only_owner]
    #[endpoint(whitelistUsdc)]
    fn whitelist_usdc(&self, chain_specific_usdc_token_id: TokenIdentifier) {
        let _ = self
            .chain_specific_usdc_token_ids()
            .insert(chain_specific_usdc_token_id);
    }

    #[only_owner]
    #[endpoint(blacklistUsdc)]
    fn blacklist_usdc(&self, chain_specific_usdc_token_id: TokenIdentifier) {
        let _ = self
            .chain_specific_usdc_token_ids()
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

    /// Will wrap what it can, and send back the rest unchanged
    #[endpoint(wrapMultipleTokens)]
    fn wrap_multiple_tokens(&self) -> PaymentsVec<Self::Api> {
        let original_payments = self.call_value().all_esdt_transfers();
        if original_payments.is_empty() {
            return original_payments;
        }

        let mut new_payments = ManagedVec::new();
        let token_whitelist = self.chain_specific_usdc_token_ids();
        let universal_token_id = self.universal_bridged_usdc_token_id().get();

        for p in &original_payments {
            let new_payment = if token_whitelist.contains(&p.token_identifier) {
                self.send()
                    .esdt_local_mint(&universal_token_id, 0, &p.amount);

                EsdtTokenPayment {
                    token_type: EsdtTokenType::Fungible,
                    token_identifier: universal_token_id.clone(),
                    token_nonce: 0,
                    amount: p.amount,
                }
            } else {
                p
            };

            new_payments.push(new_payment);
        }

        let caller = self.blockchain().get_caller();
        self.send().direct_multi(&caller, &new_payments, &[]);

        new_payments
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
