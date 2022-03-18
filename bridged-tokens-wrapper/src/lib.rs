#![no_std]

use transaction::PaymentsVec;

elrond_wasm::imports!();

#[elrond_wasm::contract]
pub trait BridgedTokensWrapper {
    #[init]
    fn init(
        &self,
        universal_bridged_token_ids: TokenIdentifier,
        #[var_args] chain_specific_tokens: MultiValueEncoded<TokenIdentifier>,
    ) {
        self.add_wrapped_token(universal_bridged_token_ids.clone());

        for token_id in chain_specific_tokens {
            let _ = self
                .chain_specific_token_ids(&universal_bridged_token_ids)
                .insert(token_id);
        }
    }

    #[only_owner]
    #[endpoint(addWrappedToken)]
    fn add_wrapped_token(&self, universal_bridged_token_ids: TokenIdentifier) {
        self.require_mint_and_burn_roles(&universal_bridged_token_ids);
        self.universal_bridged_token_ids()
            .insert(universal_bridged_token_ids);
    }

    #[only_owner]
    #[endpoint(removeWrappedToken)]
    fn remove_wrapped_token(&self, universal_bridged_token_ids: TokenIdentifier) {
        let _ = self
            .universal_bridged_token_ids()
            .swap_remove(&universal_bridged_token_ids);

        let chain_specific_tokens = &self.chain_specific_token_ids(&universal_bridged_token_ids);

        for token in chain_specific_tokens.iter() {
            self.chain_specific_to_universal_mapping(&token).clear();
        }

        let _ = self
            .chain_specific_token_ids(&universal_bridged_token_ids)
            .clear();
    }

    #[only_owner]
    #[endpoint(whitelistToken)]
    fn whitelist_token(
        &self,
        chain_specific_token_id: TokenIdentifier,
        universal_bridged_token_ids: TokenIdentifier,
    ) {
        let _ = self
            .chain_specific_token_ids(&universal_bridged_token_ids)
            .insert(chain_specific_token_id.clone());

        let _ = self
            .chain_specific_to_universal_mapping(&chain_specific_token_id)
            .set(universal_bridged_token_ids);
    }

    #[only_owner]
    #[endpoint(blacklistToken)]
    fn blacklist_token(&self, chain_specific_token_id: TokenIdentifier) {
        let universal_bridged_token_ids = self
            .chain_specific_to_universal_mapping(&chain_specific_token_id)
            .get();

        let _ = self
            .chain_specific_token_ids(&universal_bridged_token_ids)
            .swap_remove(&chain_specific_token_id);

        self.chain_specific_to_universal_mapping(&chain_specific_token_id)
            .clear();
    }

    /// Will wrap what it can, and send back the rest unchanged
    #[payable("*")]
    #[endpoint(wrapTokens)]
    fn wrap_tokens(&self) -> PaymentsVec<Self::Api> {
        let original_payments = self.call_value().all_esdt_transfers();
        if original_payments.is_empty() {
            return original_payments;
        }

        let mut new_payments = ManagedVec::new();

        for payment in &original_payments {
            let universal_token_id = self
                .chain_specific_to_universal_mapping(&payment.token_identifier)
                .get();
            let token_whitelist = self.chain_specific_token_ids(&universal_token_id);
            let new_payment = if token_whitelist.contains(&payment.token_identifier) {
                self.send()
                    .esdt_local_mint(&universal_token_id, 0, &payment.amount);

                self.token_liquidity(&payment.token_identifier)
                    .update(|value| *value += &payment.amount);

                EsdtTokenPayment {
                    token_type: EsdtTokenType::Fungible,
                    token_identifier: universal_token_id.clone(),
                    token_nonce: 0,
                    amount: payment.amount.clone(),
                }
            } else {
                payment
            };

            new_payments.push(new_payment);
        }

        let caller = self.blockchain().get_caller();

        self.send().direct_multi(&caller, &new_payments, &[]);

        new_payments
    }

    #[payable("*")]
    #[endpoint(unwrapToken)]
    fn unwrap_token(
        &self,
        #[payment_token] payment_token: TokenIdentifier,
        #[payment_amount] payment_amount: BigUint,
        requested_token: TokenIdentifier,
    ) {
        let universal_bridged_token_ids = self
            .chain_specific_to_universal_mapping(&requested_token)
            .get();
        require!(
            payment_token == universal_bridged_token_ids,
            "Esdt token unavailable"
        );

        let chain_specific_token_id = &requested_token;

        require!(payment_amount > 0u32, "Must pay more than 0 tokens!");
        require!(
            payment_amount <= self.token_liquidity(chain_specific_token_id).get(),
            "Contract does not have enough funds"
        );

        self.token_liquidity(chain_specific_token_id)
            .update(|value| *value -= &payment_amount);

        self.send()
            .esdt_local_burn(&universal_bridged_token_ids, 0, &payment_amount);

        let caller = self.blockchain().get_caller();

        self.send()
            .direct(&caller, chain_specific_token_id, 0, &payment_amount, &[]);
    }

    fn require_mint_and_burn_roles(&self, token_id: &TokenIdentifier) {
        let roles = self.blockchain().get_esdt_local_roles(token_id);

        require!(
            roles.has_role(&EsdtLocalRole::Mint) & roles.has_role(&EsdtLocalRole::Burn),
            "Must set local role first"
        );
    }

    #[view(getUniversalBridgedTokenIds)]
    #[storage_mapper("universalBridgedTokenIds")]
    fn universal_bridged_token_ids(&self) -> UnorderedSetMapper<TokenIdentifier>;

    #[view(getTokenLiquidity)]
    #[storage_mapper("tokenLiquidity")]
    fn token_liquidity(&self, token: &TokenIdentifier) -> SingleValueMapper<BigUint>;

    #[view(getChainSpecificToUniversalMapping)]
    #[storage_mapper("chainSpecificToUniversalMapping")]
    fn chain_specific_to_universal_mapping(
        &self,
        token: &TokenIdentifier,
    ) -> SingleValueMapper<TokenIdentifier>;

    #[view(getchainSpecificTokenIds)]
    #[storage_mapper("chainSpecificTokenIds")]
    fn chain_specific_token_ids(
        &self,
        universal_token_id: &TokenIdentifier,
    ) -> UnorderedSetMapper<TokenIdentifier>;
}
