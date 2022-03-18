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
        let mut whitelist = self.chain_specific_token_ids(&universal_bridged_token_ids);
        self.add_wrapped_token(universal_bridged_token_ids);

        for token_id in chain_specific_tokens {
            let _ = whitelist.insert(token_id);
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

        let mut chain_specific_tokens = self.chain_specific_token_ids(&universal_bridged_token_ids);
        for token in chain_specific_tokens.iter() {
            self.chain_specific_to_universal_mapping(&token).clear();
        }

        chain_specific_tokens.clear();
    }

    #[only_owner]
    #[endpoint(whitelistToken)]
    fn whitelist_token(
        &self,
        chain_specific_token_id: TokenIdentifier,
        universal_bridged_token_ids: TokenIdentifier,
    ) {
        let chain_to_universal_mapper =
            self.chain_specific_to_universal_mapping(&chain_specific_token_id);
        require!(
            chain_to_universal_mapper.is_empty(),
            "Chain-specific token is already mapped to another universal token"
        );

        chain_to_universal_mapper.set(&universal_bridged_token_ids);

        let _ = self
            .chain_specific_token_ids(&universal_bridged_token_ids)
            .insert(chain_specific_token_id);
    }

    #[only_owner]
    #[endpoint(blacklistToken)]
    fn blacklist_token(&self, chain_specific_token_id: TokenIdentifier) {
        let chain_to_universal_mapper =
            self.chain_specific_to_universal_mapping(&chain_specific_token_id);

        let universal_bridged_token_ids = chain_to_universal_mapper.get();

        let _ = self
            .chain_specific_token_ids(&universal_bridged_token_ids)
            .swap_remove(&chain_specific_token_id);

        chain_to_universal_mapper.clear();
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
            let universal_token_id_mapper =
                self.chain_specific_to_universal_mapping(&payment.token_identifier);

            // if there is chain specific -> universal mapping, then the token is whitelisted
            let new_payment = if !universal_token_id_mapper.is_empty() {
                let universal_token_id = universal_token_id_mapper.get();
                self.send()
                    .esdt_local_mint(&universal_token_id, 0, &payment.amount);

                self.token_liquidity(&payment.token_identifier)
                    .update(|value| *value += &payment.amount);

                EsdtTokenPayment {
                    token_type: EsdtTokenType::Fungible,
                    token_identifier: universal_token_id.clone(),
                    token_nonce: 0,
                    amount: payment.amount,
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
        require!(payment_amount > 0u32, "Must pay more than 0 tokens!");

        let universal_bridged_token_ids = self
            .chain_specific_to_universal_mapping(&requested_token)
            .get();
        require!(
            payment_token == universal_bridged_token_ids,
            "Esdt token unavailable"
        );

        let chain_specific_token_id = &requested_token;
        let token_liquidity_mapper = self.token_liquidity(chain_specific_token_id);
        let liquidity_amount = token_liquidity_mapper.get();
        require!(
            payment_amount <= liquidity_amount,
            "Contract does not have enough funds"
        );

        token_liquidity_mapper.set(&liquidity_amount - &payment_amount);

        self.send()
            .esdt_local_burn(&universal_bridged_token_ids, 0, &payment_amount);

        let caller = self.blockchain().get_caller();
        self.send()
            .direct(&caller, chain_specific_token_id, 0, &payment_amount, &[]);
    }

    fn require_mint_and_burn_roles(&self, token_id: &TokenIdentifier) {
        let roles = self.blockchain().get_esdt_local_roles(token_id);

        require!(
            roles.has_role(&EsdtLocalRole::Mint) && roles.has_role(&EsdtLocalRole::Burn),
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
