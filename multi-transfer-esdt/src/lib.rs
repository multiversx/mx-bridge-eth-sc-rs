#![no_std]

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

        self.require_local_mint_role_set(&token_id)?;
        self.token_whitelist().insert(token_id);

        Ok(())
    }

    #[endpoint(transferEsdtToken)]
    fn transfer_esdt_token(
        &self,
        to: Address,
        token_id: TokenIdentifier,
        amount: Self::BigUint,
    ) -> SCResult<()> {
        only_owner!(self, "only owner may call this function");
        require!(!to.is_zero(), "Can't transfer to address zero");

        self.send().esdt_local_mint(
            self.blockchain().get_gas_left(),
            token_id.as_esdt_identifier(),
            &amount,
        );

        match self.send().direct_esdt_via_transf_exec(
            &to,
            token_id.as_esdt_identifier(),
            &amount,
            self.data_or_empty(&to, b"offchain transfer"),
        ) {
            Result::Ok(()) => Ok(()),
            Result::Err(_) => sc_error!("Transfer failed"),
        }
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

    fn require_local_mint_role_set(&self, _token_id: &TokenIdentifier) -> SCResult<()> {
        /* TODO: Uncomment on next elrond-wasm version
        let roles = self
            .blockchain()
            .get_esdt_local_roles(token_id.as_esdt_identifier());
        require!(
            roles.contains(&EsdtLocalRole::Mint),
            "Must set local mint role first"
        );
        */

        Ok(())
    }

    // storage

    #[storage_mapper("tokenWhitelist")]
    fn token_whitelist(&self) -> SetMapper<Self::Storage, TokenIdentifier>;
}
