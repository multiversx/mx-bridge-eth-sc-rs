#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub const PERCENTAGE_TOTAL: u32 = 10_000; // precision of 2 decimals
pub static INVALID_PERCENTAGE_SUM_OVER_ERR_MSG: &[u8] = b"Percentages do not add up to 100%";

#[derive(NestedEncode, NestedDecode, TypeAbi, ManagedVecItem, Clone)]
pub struct AddressPercentagePair<M: ManagedTypeApi> {
    pub address: ManagedAddress<M>,
    pub percentage: u32,
}

#[multiversx_sc::module]
pub trait TokenModule: fee_estimator_module::FeeEstimatorModule {
    // endpoints - owner-only

    /// Distributes the accumulated fees to the given addresses.
    /// Expected arguments are pairs of (address, percentage),
    /// where percentages must add up to the PERCENTAGE_TOTAL constant
    #[only_owner]
    #[endpoint(distributeFees)]
    fn distribute_fees(
        &self,
        address_percentage_pairs: ManagedVec<AddressPercentagePair<Self::Api>>,
    ) {
        let percentage_total = BigUint::from(PERCENTAGE_TOTAL);

        let mut percentage_sum = 0u64;
        for pair in &address_percentage_pairs {
            percentage_sum += pair.percentage as u64;
        }
        require!(
            percentage_sum == PERCENTAGE_TOTAL as u64,
            INVALID_PERCENTAGE_SUM_OVER_ERR_MSG
        );

        for token_id in self.token_whitelist().iter() {
            let accumulated_fees = self.accumulated_transaction_fees(&token_id).get();
            if accumulated_fees == 0u32 {
                continue;
            }

            let mut remaining_fees = accumulated_fees.clone();

            for pair in &address_percentage_pairs {
                let amount_to_send =
                    &(&accumulated_fees * &BigUint::from(pair.percentage)) / &percentage_total;

                if amount_to_send > 0 {
                    remaining_fees -= &amount_to_send;

                    self.send()
                        .direct_esdt(&pair.address, &token_id, 0, &amount_to_send);
                }
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
        opt_default_price_per_gas_unit: OptionalValue<BigUint>,
    ) {
        self.token_ticker(&token_id).set(&ticker);

        if let OptionalValue::Some(default_price_per_gas_unit) = opt_default_price_per_gas_unit {
            self.default_price_per_gas_unit(&token_id)
                .set(&default_price_per_gas_unit);
        }

        let _ = self.token_whitelist().insert(token_id);
    }

    #[only_owner]
    #[endpoint(removeTokenFromWhitelist)]
    fn remove_token_from_whitelist(&self, token_id: TokenIdentifier) {
        self.token_ticker(&token_id).clear();
        self.default_price_per_gas_unit(&token_id).clear();

        let _ = self.token_whitelist().swap_remove(&token_id);
    }

    // private

    fn require_token_in_whitelist(&self, token_id: &TokenIdentifier) {
        require!(
            self.token_whitelist().contains(token_id),
            "Token not in whitelist"
        );
    }

    fn require_local_role_set(&self, token_id: &TokenIdentifier, role: &EsdtLocalRole) {
        require!(
            self.is_local_role_set(token_id, role),
            "Must set local role first"
        );
    }

    fn is_local_role_set(&self, token_id: &TokenIdentifier, role: &EsdtLocalRole) -> bool {
        let roles = self.blockchain().get_esdt_local_roles(token_id);

        roles.has_role(role)
    }

    // storage

    #[view(getAllKnownTokens)]
    #[storage_mapper("tokenWhitelist")]
    fn token_whitelist(&self) -> UnorderedSetMapper<TokenIdentifier>;

    #[view(getAccumulatedTransactionFees)]
    #[storage_mapper("accumulatedTransactionFees")]
    fn accumulated_transaction_fees(
        &self,
        token_id: &TokenIdentifier,
    ) -> SingleValueMapper<BigUint>;
}
