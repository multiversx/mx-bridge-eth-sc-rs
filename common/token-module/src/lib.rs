#![no_std]

elrond_wasm::imports!();

pub const PERCENTAGE_TOTAL: u64 = 10_000; // precision of 2 decimals

#[elrond_wasm::module]
pub trait TokenModule: fee_estimator_module::FeeEstimatorModule {
    // endpoints - owner-only

    #[only_owner]
    #[endpoint(distributeFees)]
    fn distribute_fees(&self, address_percentage_pairs: Vec<(ManagedAddress, u64)>) {
        let percentage_total = BigUint::from(PERCENTAGE_TOTAL);

        for token_id in self.token_whitelist().iter() {
            let accumulated_fees = self.accumulated_transaction_fees(&token_id).get();
            if accumulated_fees == 0u32 {
                continue;
            }

            let mut remaining_fees = accumulated_fees.clone();

            for (dest_address, percentage) in &address_percentage_pairs {
                let amount_to_send =
                    &(&accumulated_fees * &BigUint::from(*percentage)) / &percentage_total;

                remaining_fees -= &amount_to_send;

                self.send()
                    .direct(dest_address, &token_id, 0, &amount_to_send, &[]);
            }

            self.accumulated_transaction_fees(&token_id)
                .set(&remaining_fees);
        }
    }

    #[only_owner]
    #[endpoint(addTokenToWhitelist)]
    fn add_token_to_whitelist(
        &self,
        token_id: TokenIdentifier,
        ticker: ManagedBuffer,
        #[var_args] opt_default_price_per_gas_unit: OptionalArg<BigUint>,
    ) -> SCResult<()> {
        self.token_ticker(&token_id).set(&ticker);

        if let OptionalArg::Some(default_price_per_gas_unit) = opt_default_price_per_gas_unit {
            self.default_price_per_gas_unit(&token_id)
                .set(&default_price_per_gas_unit);
        }

        let _ = self.token_whitelist().insert(token_id);

        Ok(())
    }

    #[only_owner]
    #[endpoint(removeTokenFromWhitelist)]
    fn remove_token_from_whitelist(&self, token_id: TokenIdentifier) {
        self.token_ticker(&token_id).clear();
        self.default_price_per_gas_unit(&token_id).clear();

        let _ = self.token_whitelist().remove(&token_id);
    }

    // private

    fn require_token_in_whitelist(&self, token_id: &TokenIdentifier) -> SCResult<()> {
        require!(
            self.token_whitelist().contains(token_id),
            "Token not in whitelist"
        );
        Ok(())
    }

    fn require_local_role_set(
        &self,
        token_id: &TokenIdentifier,
        role: &EsdtLocalRole,
    ) -> SCResult<()> {
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

    #[view(getAllKnownTokens)]
    #[storage_mapper("tokenWhitelist")]
    fn token_whitelist(&self) -> SetMapper<TokenIdentifier>;

    #[storage_mapper("accumulatedTransactionFees")]
    fn accumulated_transaction_fees(
        &self,
        token_id: &TokenIdentifier,
    ) -> SingleValueMapper<BigUint>;
}
