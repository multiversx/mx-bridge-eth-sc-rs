#![no_std]

use multiversx_sc::derive_imports::*;
use multiversx_sc::imports::*;

pub const PERCENTAGE_TOTAL: u32 = 10_000; // precision of 2 decimals
pub static INVALID_PERCENTAGE_SUM_OVER_ERR_MSG: &[u8] = b"Percentages do not add up to 100%";

#[type_abi]
#[derive(NestedEncode, NestedDecode, ManagedVecItem, Clone)]
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

                    self.tx()
                        .to(&pair.address)
                        .single_esdt(&token_id, 0, &amount_to_send)
                        .transfer();
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
        mint_burn_token: bool,
        native_token: bool,
        opt_default_price_per_gas_unit: OptionalValue<BigUint>,
    ) {
        self.token_ticker(&token_id).set(&ticker);

        if let OptionalValue::Some(default_price_per_gas_unit) = opt_default_price_per_gas_unit {
            self.default_price_per_gas_unit(&token_id)
                .set(&default_price_per_gas_unit);
        }
        if !mint_burn_token {
            require!(native_token, "Only native tokens can be stored!");
        }
        self.mint_burn_token(&token_id).set(mint_burn_token);
        self.native_token(&token_id).set(native_token);
        let _ = self.token_whitelist().insert(token_id.clone());
    }

    #[only_owner]
    #[endpoint(removeTokenFromWhitelist)]
    fn remove_token_from_whitelist(&self, token_id: TokenIdentifier) {
        self.token_ticker(&token_id).clear();
        self.default_price_per_gas_unit(&token_id).clear();

        self.mint_burn_token(&token_id).clear();
        self.native_token(&token_id).clear();
        self.token_whitelist().swap_remove(&token_id);
    }

    #[endpoint(getTokens)]
    fn get_tokens(&self, token_id: &TokenIdentifier, amount: &BigUint) -> bool {
        let caller = self.blockchain().get_caller();
        require!(
            caller == self.multi_transfer_contract_address().get(),
            "Only MultiTransfer can get tokens"
        );

        if !self.mint_burn_token(token_id).get() {
            let total_balances_mapper = self.total_balances(token_id);
            if &total_balances_mapper.get() >= amount {
                total_balances_mapper.update(|total| {
                    *total -= amount;
                });
                self.tx()
                    .to(ToCaller)
                    .single_esdt(token_id, 0, amount)
                    .transfer();

                return true;
            } else {
                return false;
            }
        }

        let burn_balances_mapper = self.burn_balances(token_id);
        let mint_balances_mapper = self.mint_balances(token_id);
        if self.native_token(token_id).get() {
            require!(
                burn_balances_mapper.get() >= &mint_balances_mapper.get() + amount,
                "Not enough burned tokens!"
            );
        }

        let mint_executed = self.internal_mint(token_id, amount);
        if !mint_executed {
            return false;
        }
        self.tx()
            .to(ToCaller)
            .single_esdt(token_id, 0, amount)
            .transfer();

        mint_balances_mapper.update(|minted| {
            *minted += amount;
        });

        true
    }

    // private

    fn internal_mint(&self, token_id: &TokenIdentifier, amount: &BigUint) -> bool {
        if !self.is_local_role_set(token_id, &EsdtLocalRole::Mint) {
            return false;
        }
        self.send().esdt_local_mint(token_id, 0, amount);
        return true;
    }

    fn internal_burn(&self, token_id: &TokenIdentifier, amount: &BigUint) -> bool {
        if !self.is_local_role_set(token_id, &EsdtLocalRole::Burn) {
            return false;
        }
        self.send().esdt_local_burn(token_id, 0, amount);
        return true;
    }

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

    #[only_owner]
    #[endpoint(setMultiTransferContractAddress)]
    fn set_multi_transfer_contract_address(&self, opt_new_address: OptionalValue<ManagedAddress>) {
        match opt_new_address {
            OptionalValue::Some(sc_addr) => {
                self.multi_transfer_contract_address().set(&sc_addr);
            }
            OptionalValue::None => self.multi_transfer_contract_address().clear(),
        }
    }

    #[endpoint(setTotalBalances)]
    #[label("test-only")]
    fn set_total_balances(&self, token_id: &TokenIdentifier, value: BigUint) {
        self.total_balances(token_id).set_if_empty(value);
    }

    #[endpoint(setMintBalances)]
    #[label("test-only")]
    fn set_mint_balances(&self, token_id: &TokenIdentifier, value: BigUint) {
        self.mint_balances(token_id).set_if_empty(value);
    }

    // storage

    #[view(getAllKnownTokens)]
    #[storage_mapper("tokenWhitelist")]
    fn token_whitelist(&self) -> UnorderedSetMapper<TokenIdentifier>;

    #[view(isNativeToken)]
    #[storage_mapper("nativeTokens")]
    fn native_token(&self, token: &TokenIdentifier) -> SingleValueMapper<bool>;

    #[view(isMintBurnToken)]
    #[storage_mapper("mintBurnToken")]
    fn mint_burn_token(&self, token: &TokenIdentifier) -> SingleValueMapper<bool>;

    #[view(getMultiTransferContractAddress)]
    #[storage_mapper("multiTransferContractAddress")]
    fn multi_transfer_contract_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getAccumulatedTransactionFees)]
    #[storage_mapper("accumulatedTransactionFees")]
    fn accumulated_transaction_fees(
        &self,
        token_id: &TokenIdentifier,
    ) -> SingleValueMapper<BigUint>;

    #[view(getTotalBalances)]
    #[storage_mapper("totalBalances")]
    fn total_balances(&self, token_id: &TokenIdentifier) -> SingleValueMapper<BigUint>;

    #[view(getMintBalances)]
    #[storage_mapper("mintBalances")]
    fn mint_balances(&self, token_id: &TokenIdentifier) -> SingleValueMapper<BigUint>;

    #[view(getBurnBalances)]
    #[storage_mapper("burnBalances")]
    fn burn_balances(&self, token_id: &TokenIdentifier) -> SingleValueMapper<BigUint>;
}
