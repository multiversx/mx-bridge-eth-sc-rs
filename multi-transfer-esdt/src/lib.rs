#![no_std]

use transaction::TransactionStatus;

elrond_wasm::imports!();

pub type SingleTransferTuple<BigUint> = (Address, TokenIdentifier, BigUint);

#[elrond_wasm_derive::contract]
pub trait MultiTransferEsdt {
    #[init]
    fn init(&self, #[var_args] token_whitelist: VarArgs<TokenIdentifier>) -> SCResult<()> {
        for token in token_whitelist.into_vec() {
            require!(token.is_valid_esdt_identifier(), "Invalid token ID");
            self.token_whitelist().insert(token);
        }

        Ok(())
    }

    // endpoints - owner-only

    #[endpoint(addTokenToWhitelist)]
    fn add_token_to_whitelist(&self, token_id: TokenIdentifier) -> SCResult<()> {
        only_owner!(self, "only owner may call this function");
        require!(token_id.is_valid_esdt_identifier(), "Invalid token ID");
        require!(
            self.is_local_mint_role_set(&token_id),
            "Must set local mint role first"
        );

        self.token_whitelist().insert(token_id);

        Ok(())
    }

    #[endpoint(removeTokenFromWhitelist)]
    fn remove_token_from_whitelist(&self, token_id: TokenIdentifier) -> SCResult<()> {
        only_owner!(self, "only owner may call this function");

        self.token_whitelist().remove(&token_id);

        Ok(())
    }

    #[endpoint(batchTransferEsdtToken)]
    fn batch_transfer_esdt_token(
        &self,
        #[var_args] transfers: VarArgs<SingleTransferTuple<Self::BigUint>>,
    ) -> SCResult<MultiResultVec<TransactionStatus>> {
        only_owner!(self, "only owner may call this function");

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

            self.send().esdt_local_mint(token_id, amount);
            self.send().direct(to, token_id, amount, &[i as u8]);

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

    fn is_local_mint_role_set(&self, token_id: &TokenIdentifier) -> bool {
        let roles = self.blockchain().get_esdt_local_roles(token_id);

        roles.contains(&EsdtLocalRole::Mint)
    }

    // storage

    #[storage_mapper("tokenWhitelist")]
    fn token_whitelist(&self) -> SetMapper<Self::Storage, TokenIdentifier>;
}
