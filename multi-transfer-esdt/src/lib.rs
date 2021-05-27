#![no_std]

use transaction::TransactionStatus;

elrond_wasm::imports!();

#[elrond_wasm_derive::contract]
pub trait MultiTransferEsdt {
    #[init]
    fn init(&self, #[var_args] token_whitelist: VarArgs<TokenIdentifier>) -> SCResult<()> {
        for token in token_whitelist.into_vec() {
            self.token_whitelist().insert(token);
        }

        Ok(())
    }

    // endpoints - owner-only

    /// Only add after setting localMint role
    #[endpoint(addTokenToWhitelist)]
    fn add_token_to_whitelist(&self, token_id: TokenIdentifier) -> SCResult<()> {
        only_owner!(self, "only owner may call this function");
        require!(
            self.is_local_mint_role_set(&token_id),
            "Must set local mint role first"
        );

        self.token_whitelist().insert(token_id);

        Ok(())
    }

    #[endpoint(batchTransferEsdtToken)]
    fn batch_transfer_esdt_token(
        &self,
        #[var_args] transfers: VarArgs<(Address, TokenIdentifier, Self::BigUint)>,
    ) -> SCResult<MultiResultVec<TransactionStatus>> {
        only_owner!(self, "only owner may call this function");

        let mut tx_statuses = Vec::new();

        for (to, token_id, amount) in transfers.into_vec() {
            if to.is_zero() || self.blockchain().is_smart_contract(&to) {
                tx_statuses.push(TransactionStatus::Rejected);
                continue;
            }
            if !self.token_whitelist().contains(&token_id)
                || !self.is_local_mint_role_set(&token_id)
            {
                tx_statuses.push(TransactionStatus::Rejected);
                continue;
            }

            self.send().esdt_local_mint(
                self.blockchain().get_gas_left(),
                token_id.as_esdt_identifier(),
                &amount,
            );
            self.send().direct(&to, &token_id, &amount, &[]);

            tx_statuses.push(TransactionStatus::Executed);
        }

        Ok(tx_statuses.into())
    }

    // views

    #[view(getScEsdtBalance)]
    fn get_sc_esdt_balance(&self, token_id: &TokenIdentifier) -> Self::BigUint {
        self.blockchain().get_esdt_balance(
            &self.blockchain().get_sc_address(),
            token_id.as_esdt_identifier(),
            0,
        )
    }

    #[view(getAllKnownTokens)]
    fn get_all_known_tokens(&self) -> MultiResultVec<TokenIdentifier> {
        let mut all_tokens = Vec::new();

        for token_id in self.token_whitelist().iter() {
            all_tokens.push(token_id);
        }

        all_tokens.into()
    }

    // private

    fn data_or_empty(&self, to: &Address, data: &'static [u8]) -> &[u8] {
        if self.blockchain().is_smart_contract(to) {
            &[]
        } else {
            data
        }
    }

    fn is_local_mint_role_set(&self, token_id: &TokenIdentifier) -> bool {
        let roles = self
            .blockchain()
            .get_esdt_local_roles(token_id.as_esdt_identifier());

        roles.contains(&EsdtLocalRole::Mint)
    }

    // storage

    #[storage_mapper("tokenWhitelist")]
    fn token_whitelist(&self) -> SetMapper<Self::Storage, TokenIdentifier>;
}
