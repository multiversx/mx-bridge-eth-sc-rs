#![no_std]

elrond_wasm::imports!();

const INITIAL_SUPPLY: u32 = 1;

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
        #[payment] issue_cost: BigUint,
    ) -> SCResult<AsyncCall<BigUint>> {
        only_owner!(self, "only owner may call this function");

        Ok(ESDTSystemSmartContractProxy::new()
            .issue_fungible(
                issue_cost,
                &token_display_name,
                &token_ticker,
                &BigUint::from(INITIAL_SUPPLY),
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

    /// Address to set role for. Defaults to own address
    #[endpoint(setLocalMintRole)]
    fn set_local_mint_role(
        &self,
        token_id: TokenIdentifier,
        #[var_args] opt_address: OptionalArg<Address>,
    ) -> SCResult<AsyncCall<BigUint>> {
        only_owner!(self, "only owner may call this function");
        require!(
            self.issued_tokens().contains(&token_id),
            "Token was not issued yet"
        );

        let address = match opt_address {
            OptionalArg::Some(addr) => addr,
            OptionalArg::None => self.blockchain().get_sc_address(),
        };

        Ok(ESDTSystemSmartContractProxy::new()
            .set_special_roles(
                &address,
                token_id.as_esdt_identifier(),
                &[EsdtLocalRole::Mint],
            )
            .async_call())
    }

    /// This is mostly used to ensure Wrapped EGLD is "known" by this SC
    /// Only add after setting localMint role
    #[endpoint(addTokenToIssuedList)]
    fn add_token_to_issued_list(&self, token_id: TokenIdentifier) -> SCResult<()> {
        only_owner!(self, "only owner may call this function");

        self.issued_tokens().insert(token_id);

        Ok(())
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
    fn get_sc_esdt_balance(&self, token_id: &TokenIdentifier) -> BigUint {
        self.blockchain().get_esdt_balance(
            &self.blockchain().get_sc_address(),
            token_id.as_esdt_identifier(),
            0,
        )
    }

    // private

    fn data_or_empty(&self, to: &Address, data: &'static [u8]) -> &[u8] {
        if self.blockchain().is_smart_contract(to) {
            &[]
        } else {
            data
        }
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
                self.last_issued_token().set(&token_identifier);
                self.issued_tokens().insert(token_identifier);
            }
            AsyncCallResult::Err(_) => {
                // refund payment to caller, which is the sc owner
                if token_identifier.is_egld() && returned_tokens > 0 {
                    self.send().direct_egld(
                        &self.blockchain().get_owner_address(),
                        &returned_tokens,
                        &[],
                    );
                }
            }
        }
    }

    // storage

    #[storage_mapper("issuedTokens")]
    fn issued_tokens(&self) -> SetMapper<Self::Storage, TokenIdentifier>;

    #[view(getLastIssuedToken)]
    #[storage_mapper("lastIssuedToken")]
    fn last_issued_token(&self) -> SingleValueMapper<Self::Storage, TokenIdentifier>;
}
