#![no_std]

elrond_wasm::imports!();

#[elrond_wasm_derive::module]
pub trait TokenModule: fee_estimator_module::FeeEstimatorModule {
    // endpoints - owner-only

    /// Owner is a multisig SC, so we can't send directly to the owner or caller address here
    #[endpoint(claimAccumulatedFees)]
    fn claim_accumulated_fees(&self, dest_address: Address) -> SCResult<()> {
        self.require_caller_owner()?;
        require!(
            !self.blockchain().is_smart_contract(&dest_address),
            "Cannot transfer to smart contract dest_address"
        );

        for token_id in self.token_whitelist().iter() {
            let accumulated_fees = self.accumulated_transaction_fees(&token_id).get();
            if accumulated_fees > 0 {
                self.accumulated_transaction_fees(&token_id).clear();

                self.send()
                    .direct(&dest_address, &token_id, &accumulated_fees, &[]);
            }
        }

        Ok(())
    }

    #[endpoint(addTokenToWhitelist)]
    fn add_token_to_whitelist(
        &self,
        token_id: TokenIdentifier,
        #[var_args] opt_default_price_per_gwei: OptionalArg<Self::BigUint>,
    ) -> SCResult<()> {
        self.require_caller_owner()?;

        let default_price_per_gwei = opt_default_price_per_gwei.into_option().unwrap_or_default();

        self.default_price_per_gwei(&token_id)
            .set(&default_price_per_gwei);
        self.token_whitelist().insert(token_id);

        Ok(())
    }

    #[endpoint(removeTokenFromWhitelist)]
    fn remove_token_from_whitelist(&self, token_id: TokenIdentifier) -> SCResult<()> {
        self.require_caller_owner()?;

        self.token_whitelist().remove(&token_id);
        self.default_price_per_gwei(&token_id).clear();

        Ok(())
    }

    // views

    #[view(getAllKnownTokens)]
    fn get_all_known_tokens(&self) -> MultiResultVec<TokenIdentifier> {
        let mut all_tokens = Vec::new();

        for token_id in self.token_whitelist().iter() {
            all_tokens.push(token_id);
        }

        all_tokens.into()
    }

    // private

    fn require_caller_owner(&self) -> SCResult<()> {
        only_owner!(self, "only owner may call this function");

        Ok(())
    }

    fn require_local_role_set(&self, token_id: &TokenIdentifier, role: &EsdtLocalRole) -> SCResult<()> {
        require!(
            self.is_local_role_set(token_id, role),
            "Must set local role first"
        );

        Ok(())
    }

    fn is_local_role_set(&self, token_id: &TokenIdentifier, role: &EsdtLocalRole) -> bool {
        let roles = self.blockchain().get_esdt_local_roles(token_id);

        roles.contains(role)
    }
    
    // storage

    #[storage_mapper("tokenWhitelist")]
    fn token_whitelist(&self) -> SetMapper<Self::Storage, TokenIdentifier>;

    #[storage_mapper("accumulatedTransactionFees")]
    fn accumulated_transaction_fees(
        &self,
        token_id: &TokenIdentifier,
    ) -> SingleValueMapper<Self::Storage, Self::BigUint>;
}
