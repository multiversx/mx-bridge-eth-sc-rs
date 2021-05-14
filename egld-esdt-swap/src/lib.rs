#![no_std]

elrond_wasm::imports!();

const EGLD_DECIMALS: usize = 18;
const INITIAL_SUPPLY: u32 = 1;

#[elrond_wasm_derive::contract]
pub trait EgldEsdtSwap {
    #[init]
    fn init(&self) {}

    // endpoints - owner-only

    #[payable("EGLD")]
    #[endpoint(issueWrappedEgld)]
    fn issue_wrapped_egld(
        &self,
        token_display_name: BoxedBytes,
        token_ticker: BoxedBytes,
        #[payment] issue_cost: Self::BigUint,
    ) -> SCResult<AsyncCall<Self::SendApi>> {
        only_owner!(self, "only owner may call this function");
        require!(
            self.wrapped_egld_token_identifier().is_empty(),
            "wrapped egld was already issued"
        );

        Ok(ESDTSystemSmartContractProxy::new_proxy_obj(self.send())
            .issue_fungible(
                issue_cost,
                &token_display_name,
                &token_ticker,
                &Self::BigUint::from(INITIAL_SUPPLY),
                FungibleTokenProperties {
                    num_decimals: EGLD_DECIMALS,
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

    /// Address to set role for
    #[endpoint(setLocalRoles)]
    fn set_local_roles(&self, address: Address) -> SCResult<AsyncCall<Self::SendApi>> {
        only_owner!(self, "only owner may call this function");
        require!(
            !self.wrapped_egld_token_identifier().is_empty(),
            "Wrapped eGLD was not issued yet"
        );

        let token_id = self.wrapped_egld_token_identifier().get();

        Ok(ESDTSystemSmartContractProxy::new_proxy_obj(self.send())
            .set_special_roles(
                &address,
                token_id.as_esdt_identifier(),
                &[EsdtLocalRole::Mint, EsdtLocalRole::Burn],
            )
            .async_call())
    }

    // endpoints

    #[payable("EGLD")]
    #[endpoint(wrapEgld)]
    fn wrap_egld(&self, #[payment] payment: Self::BigUint) -> SCResult<()> {
        require!(payment > 0, "Payment must be more than 0");
        require!(
            !self.wrapped_egld_token_identifier().is_empty(),
            "Wrapped eGLD was not issued yet"
        );

        let wrapped_egld_token_id = self.wrapped_egld_token_identifier().get();
        self.send().esdt_local_mint(
            self.blockchain().get_gas_left(),
            wrapped_egld_token_id.as_esdt_identifier(),
            &payment,
        );

        let caller = self.blockchain().get_caller();
        match self.send().direct_esdt_via_transf_exec(
            &caller,
            wrapped_egld_token_id.as_esdt_identifier(),
            &payment,
            self.data_or_empty(&caller, b"wrapping"),
        ) {
            Result::Ok(()) => Ok(()),
            Result::Err(_) => sc_error!("Wrapping failed"),
        }
    }

    #[payable("*")]
    #[endpoint(unwrapEgld)]
    fn unwrap_egld(
        &self,
        #[payment] payment: Self::BigUint,
        #[payment_token] token_identifier: TokenIdentifier,
    ) -> SCResult<()> {
        let wrapped_egld_token_id = self.wrapped_egld_token_identifier().get();

        require!(
            !self.wrapped_egld_token_identifier().is_empty(),
            "Wrapped eGLD was not issued yet"
        );
        require!(token_identifier.is_esdt(), "Only ESDT tokens accepted");
        require!(
            token_identifier == wrapped_egld_token_id,
            "Wrong esdt token"
        );
        require!(payment > 0, "Must pay more than 0 tokens!");
        // this should never happen, but we'll check anyway
        require!(
            payment <= self.blockchain().get_sc_balance(),
            "Contract does not have enough funds"
        );

        self.send().esdt_local_burn(
            self.blockchain().get_gas_left(),
            wrapped_egld_token_id.as_esdt_identifier(),
            &payment,
        );

        // 1 wrapped eGLD = 1 eGLD, so we pay back the same amount
        let caller = self.blockchain().get_caller();
        self.send().direct_egld(
            &caller,
            &payment,
            self.data_or_empty(&caller, b"unwrapping"),
        );

        Ok(())
    }

    // views

    #[view(getLockedEgldBalance)]
    fn get_locked_egld_balance(&self) -> Self::BigUint {
        self.blockchain().get_sc_balance()
    }

    #[view(getWrappedEgldRemaining)]
    fn get_wrapped_egld_remaining(&self) -> Self::BigUint {
        self.blockchain().get_esdt_balance(
            &self.blockchain().get_sc_address(),
            self.wrapped_egld_token_identifier()
                .get()
                .as_esdt_identifier(),
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
        #[payment_token] token_id: TokenIdentifier,
        #[payment] returned_tokens: Self::BigUint,
        #[call_result] result: AsyncCallResult<()>,
    ) -> OptionalResult<AsyncCall<Self::SendApi>> {
        // callback is called with ESDTTransfer of the newly issued token, with the amount requested,
        // so we can get the token identifier and amount from the call data
        match result {
            AsyncCallResult::Ok(()) => {
                self.wrapped_egld_token_identifier().set(&token_id);

                OptionalResult::Some(
                    ESDTSystemSmartContractProxy::new_proxy_obj(self.send())
                        .set_special_roles(
                            &self.blockchain().get_sc_address(),
                            token_id.as_esdt_identifier(),
                            &[EsdtLocalRole::Mint, EsdtLocalRole::Burn],
                        )
                        .async_call(),
                )
            }
            AsyncCallResult::Err(_) => {
                // refund payment to caller, which is the sc owner
                if token_id.is_egld() && returned_tokens > 0 {
                    self.send().direct_egld(
                        &self.blockchain().get_owner_address(),
                        &returned_tokens,
                        &[],
                    );
                }

                OptionalResult::None
            }
        }
    }

    // storage

    // 1 eGLD = 1 wrapped eGLD, and they are interchangeable through this contract

    #[view(getWrappedEgldTokenIdentifier)]
    #[storage_mapper("wrappedEgldTokenIdentifier")]
    fn wrapped_egld_token_identifier(&self) -> SingleValueMapper<Self::Storage, TokenIdentifier>;
}
