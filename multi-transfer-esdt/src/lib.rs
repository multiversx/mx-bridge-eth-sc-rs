#![no_std]

use fee_estimator_module::DENOMINATION;
use transaction::TransactionStatus;

elrond_wasm::imports!();

pub type SingleTransferTuple<BigUint> = (Address, TokenIdentifier, BigUint);

#[elrond_wasm_derive::contract]
pub trait MultiTransferEsdt: fee_estimator_module::FeeEstimatorModule {
    #[init]
    fn init(
        &self,
        required_fee_per_transaction_in_dollars: Self::BigUint,
        fee_estimator_contract_address: Address,
        #[var_args] token_whitelist: VarArgs<TokenIdentifier>,
    ) -> SCResult<()> {
        for token in token_whitelist.into_vec() {
            require!(token.is_valid_esdt_identifier(), "Invalid token ID");
            self.token_whitelist().insert(token);
        }

        self.required_fee_per_transaction_in_dollars()
            .set(&required_fee_per_transaction_in_dollars);
        self.fee_estimator_contract_address()
            .set(&fee_estimator_contract_address);

        Ok(())
    }

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
        #[var_args] opt_default_value_in_dollars: OptionalArg<Self::BigUint>,
    ) -> SCResult<()> {
        self.require_caller_owner()?;
        require!(token_id.is_valid_esdt_identifier(), "Invalid token ID");
        require!(
            self.is_local_mint_role_set(&token_id),
            "Must set local mint role first"
        );

        let default_value_in_dollars = opt_default_value_in_dollars
            .into_option()
            .unwrap_or_default();

        self.default_value_in_dollars(&token_id)
            .set(&default_value_in_dollars);
        self.token_whitelist().insert(token_id);

        Ok(())
    }

    #[endpoint(removeTokenFromWhitelist)]
    fn remove_token_from_whitelist(&self, token_id: TokenIdentifier) -> SCResult<()> {
        self.require_caller_owner()?;

        self.token_whitelist().remove(&token_id);
        self.default_value_in_dollars(&token_id).clear();

        Ok(())
    }

    #[endpoint(batchTransferEsdtToken)]
    fn batch_transfer_esdt_token(
        &self,
        #[var_args] transfers: VarArgs<SingleTransferTuple<Self::BigUint>>,
    ) -> SCResult<MultiResultVec<TransactionStatus>> {
        self.require_caller_owner()?;

        let mut tx_statuses = Vec::new();

        for (i, (to, token_id, amount)) in transfers.into_vec().iter().enumerate() {
            if to.is_zero() || self.blockchain().is_smart_contract(to) {
                tx_statuses.push(TransactionStatus::Rejected);
                continue;
            }
            if !self.token_whitelist().contains(token_id) || !self.is_local_mint_role_set(token_id)
            {
                tx_statuses.push(TransactionStatus::Rejected);
                continue;
            }

            let token_value_in_dollars = self.get_value_in_dollars(token_id);
            let required_fee_in_dollars = self.required_fee_per_transaction_in_dollars().get();
            let reserved_fee_in_tokens =
                &(required_fee_in_dollars * DENOMINATION.into()) / &token_value_in_dollars;

            if &reserved_fee_in_tokens >= amount {
                tx_statuses.push(TransactionStatus::Rejected);
                continue;
            }

            let amount_to_send = amount - &reserved_fee_in_tokens;

            self.accumulated_transaction_fees(token_id)
                .update(|fees| *fees += &reserved_fee_in_tokens);

            self.send().esdt_local_mint(token_id, amount);
            self.send()
                .direct(to, token_id, &amount_to_send, &[i as u8]);

            tx_statuses.push(TransactionStatus::Executed);
        }

        Ok(tx_statuses.into())
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

    fn data_or_empty(&self, to: &Address, data: &'static [u8]) -> &[u8] {
        if self.blockchain().is_smart_contract(to) {
            &[]
        } else {
            data
        }
    }

    fn is_local_mint_role_set(&self, token_id: &TokenIdentifier) -> bool {
        let roles = self.blockchain().get_esdt_local_roles(token_id);

        roles.contains(&EsdtLocalRole::Mint)
    }

    // storage

    #[storage_mapper("tokenWhitelist")]
    fn token_whitelist(&self) -> SetMapper<Self::Storage, TokenIdentifier>;

    #[view(getRequiredFeePerTransactionInDollars)]
    #[storage_mapper("requiredFeePerTransactionInDollars")]
    fn required_fee_per_transaction_in_dollars(
        &self,
    ) -> SingleValueMapper<Self::Storage, Self::BigUint>;

    #[storage_mapper("accumulatedTransactionFees")]
    fn accumulated_transaction_fees(
        &self,
        token_id: &TokenIdentifier,
    ) -> SingleValueMapper<Self::Storage, Self::BigUint>;
}
