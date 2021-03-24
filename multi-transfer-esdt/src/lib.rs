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
                    num_decimals: 0,
                    can_freeze: false,
                    can_wipe: false,
                    can_pause: false,
                    can_mint: true,
                    can_burn: false,
                    can_change_owner: false,
                    can_upgrade: true,
                    can_add_special_roles: true,
                },
            )
            .async_call()
            .with_callback(self.callbacks().esdt_issue_callback()))
    }

    #[endpoint(setLocalMintRole)]
    fn set_local_mint_role(&self, token_id: TokenIdentifier) -> SCResult<AsyncCall<BigUint>> {
        only_owner!(self, "only owner may call this function");

        require!(
            self.issued_tokens().contains(&token_id),
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
            self.issued_tokens().contains(&token_id),
            "Token has to be issued first"
        );

        self.try_mint(&token_id, &amount)
    }

    #[endpoint(transferEsdtToken)]
    fn transfer_esdt_token(
        &self,
        to: Address,
        token_id: TokenIdentifier,
        amount: BigUint,
    ) -> SCResult<()> {
        only_owner!(self, "only owner may call this function");

        require!(!to.is_zero(), "Can't transfer to address zero");

        let esdt_balance = self.get_sc_esdt_balance(&token_id);
        if esdt_balance < amount {
            let extra_needed = &amount - &esdt_balance;

            sc_try!(self.try_mint(&token_id, &extra_needed));
        }

        self.send().direct_esdt_via_transf_exec(
            &to,
            token_id.as_esdt_identifier(),
            &amount,
            self.data_or_empty(&to, b"offchain transfer"),
        );

        Ok(())
    }

    // views

    #[view(getScEsdtBalance)]
    fn get_sc_esdt_balance(&self, token_id: &TokenIdentifier) -> BigUint {
        self.get_esdt_balance(&self.get_sc_address(), token_id.as_esdt_identifier(), 0)
    }

    // private

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
                self.issued_tokens().insert(token_identifier);
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

    #[storage_mapper("issuedTokens")]
    fn issued_tokens(&self) -> SetMapper<Self::Storage, TokenIdentifier>;

    #[view(areLocalRolesSet)]
    #[storage_mapper("areLocalRolesSet")]
    fn are_local_roles_set(
        &self,
        token_id: &TokenIdentifier,
    ) -> SingleValueMapper<Self::Storage, bool>;
}
