#![no_std]

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait MaxBridgedAmountModule {
    #[only_owner]
    #[endpoint(setMaxBridgedAmount)]
    fn set_max_bridged_amount(&self, token_id: TokenIdentifier, max_amount: BigUint) {
        self.max_bridged_amount(&token_id).set(&max_amount);
    }

    fn is_above_max_amount(&self, token_id: &TokenIdentifier, amount: &BigUint) -> bool {
        let max_amount = self.max_bridged_amount(token_id).get();
        if max_amount > 0 {
            amount > &max_amount
        } else {
            false
        }
    }

    fn require_below_max_amount(&self, token_id: &TokenIdentifier, amount: &BigUint) {
        require!(
            !self.is_above_max_amount(token_id, amount),
            "Deposit over max amount"
        );
    }

    #[view(getMaxBridgedAmount)]
    #[storage_mapper("maxBridgedAmount")]
    fn max_bridged_amount(&self, token_id: &TokenIdentifier) -> SingleValueMapper<BigUint>;
}
