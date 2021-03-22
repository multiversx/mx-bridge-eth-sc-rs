#![no_std]

elrond_wasm::imports!();

#[elrond_wasm_derive::contract(MultiTransferEsdtImpl)]
pub trait MultiTransferEsdt {
    #[init]
    fn init(&self) {}

    // endpoints - owner-only

    #[payable("EGLD")]
    #[endpoint(issueEsdtToken)]
    fn issue_esdt_token_endpoint(
        &self,
        token_display_name: BoxedBytes,
        token_ticker: BoxedBytes,
        initial_supply: BigUint,
        num_decimals: usize,
        #[payment] issue_cost: BigUint,
    ) -> SCResult<AsyncCall<BigUint>> {
        only_owner!(self, "only owner may call this function");

        Ok(ESDTSystemSmartContractProxy::new()
            .issue_fungible(
                issue_cost,
                &token_display_name,
                &token_ticker,
                &initial_supply,
                FungibleTokenProperties {
                    num_decimals,
                    can_freeze: false,
                    can_wipe: false,
                    can_pause: false,
                    can_mint: true,
                    can_burn: false,
                    can_change_owner: false,
                    can_upgrade: true,
                    can_add_special_roles: true
                }
            )
            .async_call()
            .with_callback(self.callbacks().esdt_issue_callback()))
    }

    #[endpoint(setLocalMintRole)]
    fn set_local_mint_role(&self, token_id: TokenIdentifier) -> SCResult<AsyncCall<BigUint>> {
        only_owner!(self, "only owner may call this function");

        require!(
            self.esdt_token_balance().contains_key(&token_id),
            "Token was not issued yet"
        );
        require!(
            !self.are_local_roles_set(&token_id).get(),
            "Local roles were already set"
        );

        Ok(ESDTSystemSmartContractProxy::new()
            .set_special_roles(
                &self.get_sc_address(),
                token_id.as_esdt_identifier(),
                &[EsdtLocalRole::Mint],
            )
            .async_call()
            .with_callback(self.callbacks().set_roles_callback(token_id)))
    }

    #[endpoint(mintEsdtToken)]
    fn mint_esdt_token(&self, token_id: TokenIdentifier, amount: BigUint) -> SCResult<()> {
        only_owner!(self, "only owner may call this function");

        require!(
            self.esdt_token_balance().contains_key(&token_id),
            "Token has to be issued first"
        );

        self.try_mint(&token_id, &amount)
    }

    #[endpoint(transferEsdtToken)]
    fn transfer_esdt_token_endpoint(
        &self,
        to: Address,
        token_id: TokenIdentifier,
        amount: BigUint,
    ) -> SCResult<()> {
        only_owner!(self, "only owner may call this function");

        require!(!to.is_zero(), "Can't transfer to address zero");

        let esdt_balance = self.get_esdt_token_balance(&token_id);
        if esdt_balance < amount {
            let extra_needed = &amount - &esdt_balance;

            sc_try!(self.try_mint(&token_id, &extra_needed));
        }

        self.transfer_esdt_token(&to, &token_id, &amount);

        Ok(())
    }

    #[endpoint(batchTransferEsdtToken)]
    fn batch_transfer_esdt_token(
        &self,
        #[var_args] args: VarArgs<MultiArg3<Address, TokenIdentifier, BigUint>>,
    ) -> SCResult<()> {
        only_owner!(self, "only owner may call this function");

        for multi_arg in args.into_vec().into_iter() {
            let (to, token_id, amount) = multi_arg.into_tuple();

            require!(!to.is_zero(), "Can't transfer to address zero");
            require!(
                self.get_esdt_token_balance(&token_id) >= amount,
                "Not enough ESDT balance"
            );

            self.transfer_esdt_token(&to, &token_id, &amount);
        }

        Ok(())
    }

    // views

    #[view(getEsdtTokenBalance)]
    fn get_esdt_token_balance(&self, token_id: &TokenIdentifier) -> BigUint {
        self.esdt_token_balance()
            .get(token_id)
            .unwrap_or_else(|| BigUint::zero())
    }

    /// returns list of all the available tokens and their balance
    #[view(getAvailableTokensList)]
    fn get_available_tokens_list(&self) -> MultiResultVec<MultiResult2<TokenIdentifier, BigUint>> {
        let mut result = Vec::new();

        for (token_id, amount) in self.esdt_token_balance().iter() {
            result.push((token_id, amount).into());
        }

        result.into()
    }

    // private

    fn transfer_esdt_token(
        &self,
        to: &Address,
        token_id: &TokenIdentifier,
        amount: &BigUint,
    ) {
        self.decrease_esdt_token_balance(&token_id, &amount);

        self.send().direct_esdt_via_transf_exec(
            &to,
            token_id.as_esdt_identifier(),
            &amount,
            self.data_or_empty(&to, b"offchain transfer"),
        );
    }

    fn increase_esdt_token_balance(&self, token_id: &TokenIdentifier, amount: &BigUint) {
        let mut total_balance = self.get_esdt_token_balance(token_id);
        total_balance += amount;
        self.set_esdt_token_balance(token_id, amount);
    }

    fn decrease_esdt_token_balance(&self, token_id: &TokenIdentifier, amount: &BigUint) {
        let mut total_balance = self.get_esdt_token_balance(token_id);
        total_balance -= amount;
        self.set_esdt_token_balance(token_id, amount);
    }

    fn set_esdt_token_balance(&self, token_id: &TokenIdentifier, amount: &BigUint) {
        self.esdt_token_balance()
            .insert(token_id.clone(), amount.clone());
    }

    fn data_or_empty(&self, to: &Address, data: &'static [u8]) -> &[u8] {
        if self.is_smart_contract(to) {
            &[]
        } else {
            data
        }
    }

    fn try_mint(&self, token_id: &TokenIdentifier, amount: &BigUint) -> SCResult<()> {
        require!(
            self.are_local_roles_set(token_id).get(),
            "LocalMint role not set"
        );

        self.send()
            .esdt_local_mint(self.get_gas_left(), token_id.as_esdt_identifier(), &amount);
        self.increase_esdt_token_balance(token_id, amount);

        Ok(())
    }

    // callbacks

    #[callback]
    fn esdt_issue_callback(
        &self,
        #[payment_token] token_identifier: TokenIdentifier,
        #[payment] returned_tokens: BigUint,
        #[call_result] result: AsyncCallResult<()>,
    ) {
        // callback is called with ESDTTransfer of the newly issued token, with the amount requested,
        // so we can get the token identifier and amount from the call data
        match result {
            AsyncCallResult::Ok(()) => {
                self.esdt_token_balance()
                    .insert(token_identifier, returned_tokens);
            }
            AsyncCallResult::Err(_) => {
                // refund payment to caller, which is the sc owner
                if token_identifier.is_egld() && returned_tokens > 0 {
                    self.send()
                        .direct_egld(&self.get_owner_address(), &returned_tokens, &[]);
                }
            }
        }
    }

    #[callback]
    fn set_roles_callback(
        &self,
        token_id: TokenIdentifier,
        #[call_result] result: AsyncCallResult<()>,
    ) {
        match result {
            AsyncCallResult::Ok(()) => {
                self.are_local_roles_set(&token_id).set(&true);
            }
            AsyncCallResult::Err(_) => {}
        }
    }

    // storage

    // list of available tokens

    #[storage_mapper("esdtTokenBalance")]
    fn esdt_token_balance(&self) -> MapMapper<Self::Storage, TokenIdentifier, BigUint>;

    #[view(areLocalRolesSet)]
    #[storage_mapper("areLocalRolesSet")]
    fn are_local_roles_set(
        &self,
        token_id: &TokenIdentifier,
    ) -> SingleValueMapper<Self::Storage, bool>;
}
