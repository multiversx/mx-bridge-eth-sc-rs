#![no_std]

mod dfp_big_uint;
use core::ops::Deref;

pub use dfp_big_uint::DFPBigUint;
use transaction::PaymentsVec;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

impl<M: ManagedTypeApi> DFPBigUint<M> {}

#[multiversx_sc::contract]
pub trait BridgedTokensWrapper: multiversx_sc_modules::pause::PauseModule {
    #[init]
    fn init(&self) {
        self.set_paused(true);
    }

    #[only_owner]
    #[endpoint(addWrappedToken)]
    fn add_wrapped_token(&self, universal_bridged_token_ids: TokenIdentifier, num_decimals: u32) {
        self.require_mint_and_burn_roles(&universal_bridged_token_ids);
        self.token_decimals_num(&universal_bridged_token_ids)
            .set(num_decimals);
        self.universal_bridged_token_ids()
            .insert(universal_bridged_token_ids);
    }

    #[only_owner]
    #[endpoint(updateWrappedToken)]
    fn update_wrapped_token(
        &self,
        universal_bridged_token_ids: TokenIdentifier,
        num_decimals: u32,
    ) {
        require!(
            self.universal_bridged_token_ids()
                .contains(&universal_bridged_token_ids),
            "Universal token was not added yet"
        );
        self.token_decimals_num(&universal_bridged_token_ids)
            .set(num_decimals);
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
            self.token_decimals_num(&token).clear();
        }

        chain_specific_tokens.clear();
        self.token_decimals_num(&universal_bridged_token_ids)
            .clear();
    }

    #[only_owner]
    #[endpoint(whitelistToken)]
    fn whitelist_token(
        &self,
        chain_specific_token_id: TokenIdentifier,
        chain_specific_token_decimals: u32,
        universal_bridged_token_ids: TokenIdentifier,
    ) {
        self.require_mint_and_burn_roles(&universal_bridged_token_ids);

        let chain_to_universal_mapper =
            self.chain_specific_to_universal_mapping(&chain_specific_token_id);
        require!(
            chain_to_universal_mapper.is_empty(),
            "Chain-specific token is already mapped to another universal token"
        );

        self.token_decimals_num(&chain_specific_token_id)
            .set(chain_specific_token_decimals);

        chain_to_universal_mapper.set(&universal_bridged_token_ids);

        let _ = self
            .chain_specific_token_ids(&universal_bridged_token_ids)
            .insert(chain_specific_token_id);

        self.universal_bridged_token_ids()
            .insert(universal_bridged_token_ids);
    }

    #[only_owner]
    #[endpoint(updateWhitelistedToken)]
    fn update_whitelisted_token(
        &self,
        chain_specific_token_id: TokenIdentifier,
        chain_specific_token_decimals: u32,
    ) {
        let chain_to_universal_mapper =
            self.chain_specific_to_universal_mapping(&chain_specific_token_id);
        require!(
            !chain_to_universal_mapper.is_empty(),
            "Chain-specific token was not whitelisted yet"
        );

        self.token_decimals_num(&chain_specific_token_id)
            .set(chain_specific_token_decimals);
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
        self.token_decimals_num(&chain_specific_token_id).clear();
    }

    #[payable("*")]
    #[endpoint(depositLiquidity)]
    fn deposit_liquidity(&self) {
        let (payment_token, payment_amount) = self.call_value().single_fungible_esdt();
        self.token_liquidity(&payment_token)
            .update(|liq| *liq += payment_amount);
    }

    /// Will wrap what it can, and send back the rest unchanged
    #[payable("*")]
    #[endpoint(wrapTokens)]
    fn wrap_tokens(&self) -> PaymentsVec<Self::Api> {
        require!(self.not_paused(), "Contract is paused");
        let original_payments = self.call_value().all_esdt_transfers().deref().clone();
        if original_payments.is_empty() {
            return original_payments;
        }

        let mut new_payments = ManagedVec::new();

        for payment in &original_payments {
            let universal_token_id_mapper =
                self.chain_specific_to_universal_mapping(&payment.token_identifier);

            // if there is chain specific -> universal mapping, then the token is whitelisted
            if universal_token_id_mapper.is_empty() {
                new_payments.push(payment);
                continue;
            }
            let universal_token_id = universal_token_id_mapper.get();
            self.require_tokens_have_set_decimals_num(
                &universal_token_id,
                &payment.token_identifier,
            );
            self.token_liquidity(&payment.token_identifier)
                .update(|value| *value += &payment.amount);
            let converted_amount = self.get_converted_amount(
                &payment.token_identifier,
                &universal_token_id,
                payment.amount,
            );

            self.send()
                .esdt_local_mint(&universal_token_id, 0, &converted_amount);
            new_payments.push(EsdtTokenPayment::new(
                universal_token_id,
                0,
                converted_amount,
            ));
        }

        let caller = self.blockchain().get_caller();
        self.send().direct_multi(&caller, &new_payments);

        new_payments
    }

    #[payable("*")]
    #[endpoint(unwrapToken)]
    fn unwrap_token(&self, requested_token: TokenIdentifier) {
        require!(self.not_paused(), "Contract is paused");
        let (payment_token, payment_amount) = self.call_value().single_fungible_esdt();
        require!(payment_amount > 0u32, "Must pay more than 0 tokens!");

        let universal_bridged_token_ids = self
            .chain_specific_to_universal_mapping(&requested_token)
            .get();
        require!(
            payment_token == universal_bridged_token_ids,
            "Esdt token unavailable"
        );
        self.require_tokens_have_set_decimals_num(&payment_token, &requested_token);

        let chain_specific_token_id = &requested_token;
        let converted_amount = self.get_converted_amount(
            &payment_token,
            chain_specific_token_id,
            payment_amount.clone(),
        );

        self.token_liquidity(chain_specific_token_id).update(|liq| {
            require!(
                converted_amount <= *liq,
                "Contract does not have enough funds"
            );

            *liq -= &converted_amount;
        });

        self.send()
            .esdt_local_burn(&universal_bridged_token_ids, 0, &payment_amount);

        let caller = self.blockchain().get_caller();
        self.send()
            .direct_esdt(&caller, chain_specific_token_id, 0, &converted_amount);
    }

    fn get_converted_amount(
        &self,
        from: &TokenIdentifier,
        to: &TokenIdentifier,
        amount: BigUint,
    ) -> BigUint {
        let from_decimals = self.token_decimals_num(from).get();
        let to_decimals = self.token_decimals_num(to).get();
        let converted_amount = DFPBigUint::from_raw(amount, from_decimals);
        converted_amount.convert(to_decimals).to_raw()
    }

    fn require_mint_and_burn_roles(&self, token_id: &TokenIdentifier) {
        let roles = self.blockchain().get_esdt_local_roles(token_id);

        require!(
            roles.has_role(&EsdtLocalRole::Mint) && roles.has_role(&EsdtLocalRole::Burn),
            "Must set local role first"
        );
    }

    fn require_tokens_have_set_decimals_num(
        &self,
        universal_token: &TokenIdentifier,
        chain_token: &TokenIdentifier,
    ) {
        require!(
            !self.token_decimals_num(universal_token).is_empty(),
            "Universal token requires updating"
        );
        require!(
            !self.token_decimals_num(chain_token).is_empty(),
            "Chain-specific token requires updating"
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

    #[storage_mapper("token_decimals_num")]
    fn token_decimals_num(&self, token: &TokenIdentifier) -> SingleValueMapper<u32>;
}
