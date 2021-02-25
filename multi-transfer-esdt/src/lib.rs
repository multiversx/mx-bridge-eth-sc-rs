#![no_std]

use elrond_wasm::HexCallDataSerializer;

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

// erd1qqqqqqqqqqqqqqqpqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzllls8a5w6u
const ESDT_SYSTEM_SC_ADDRESS_ARRAY: [u8; 32] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0xff, 0xff,
];

const ESDT_ISSUE_COST: u64 = 5000000000000000000; // 5 eGLD

const ESDT_ISSUE_STRING: &[u8] = b"issue";
const ESDT_MINT_STRING: &[u8] = b"mint";

#[derive(TopEncode, TopDecode)]
pub enum EsdtOperation<BigUint: BigUintApi> {
    None,
    Issue,
    Mint(TokenIdentifier, BigUint), // token and amount minted
}

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
        num_decimals: u8,
        #[payment] payment: BigUint,
    ) -> SCResult<()> {
        only_owner!(self, "only owner may call this function");

        require!(
            payment == BigUint::from(ESDT_ISSUE_COST),
            "Wrong payment, should pay exactly 5 eGLD for ESDT token issue"
        );

        self.issue_esdt_token(
            &token_display_name,
            &token_ticker,
            &initial_supply,
            num_decimals,
        );

        Ok(())
    }

    #[endpoint(mintEsdtToken)]
    fn mint_esdt_token_endpoint(
        &self,
        token_identifier: TokenIdentifier,
        amount: BigUint,
    ) -> SCResult<()> {
        only_owner!(self, "only owner may call this function");

        require!(
            self.esdt_token_balance().contains_key(&token_identifier),
            "Token has to be issued first"
        );

        self.mint_esdt_token(&token_identifier, &amount);

        Ok(())
    }

    #[endpoint(transferEsdtToken)]
    fn transfer_esdt_token_endpoint(
        &self,
        from: Address,
        to: Address,
        token_identifier: TokenIdentifier,
        amount: BigUint,
    ) -> SCResult<()> {
        only_owner!(self, "only owner may call this function");

        require!(
            self.get_esdt_token_balance(&token_identifier) >= amount,
            "Not enough ESDT balance"
        );

        self.transfer_esdt_token(&from, &to, &token_identifier, &amount);

        Ok(())
    }

    #[endpoint(batchTransferEsdtToken)]
    fn batch_transfer_esdt_token(
        &self,
        #[var_args] args: VarArgs<MultiArg4<Address, Address, TokenIdentifier, BigUint>>,
    ) -> SCResult<()> {
        only_owner!(self, "only owner may call this function");

        for multi_arg in args.into_vec().into_iter() {
            let (from, to, token_identifier, amount) = multi_arg.into_tuple();

            require!(
                self.get_esdt_token_balance(&token_identifier) >= amount,
                "Not enough ESDT balance"
            );

            self.transfer_esdt_token(&from, &to, &token_identifier, &amount);
        }

        Ok(())
    }

    // views

    #[view(getEsdtTokenBalance)]
    fn get_esdt_token_balance(&self, token_identifier: &TokenIdentifier) -> BigUint {
        self.esdt_token_balance()
            .get(token_identifier)
            .unwrap_or_else(|| BigUint::zero())
    }

    /// returns list of all the available tokens and their balance
    #[view(getAvailableTokensList)]
    fn get_available_tokens_list(&self) -> MultiResultVec<MultiResult2<TokenIdentifier, BigUint>> {
        let mut result = Vec::new();

        for (token_identifier, amount) in self.esdt_token_balance().iter() {
            result.push((token_identifier, amount).into());
        }

        result.into()
    }

    // private

    fn issue_esdt_token(
        &self,
        token_display_name: &BoxedBytes,
        token_ticker: &BoxedBytes,
        initial_supply: &BigUint,
        num_decimals: u8,
    ) {
        let mut serializer = HexCallDataSerializer::new(ESDT_ISSUE_STRING);

        serializer.push_argument_bytes(token_display_name.as_slice());
        serializer.push_argument_bytes(token_ticker.as_slice());
        serializer.push_argument_bytes(&initial_supply.to_bytes_be());
        serializer.push_argument_bytes(&[num_decimals]);

        serializer.push_argument_bytes(&b"canFreeze"[..]);
        serializer.push_argument_bytes(&b"false"[..]);

        serializer.push_argument_bytes(&b"canWipe"[..]);
        serializer.push_argument_bytes(&b"false"[..]);

        serializer.push_argument_bytes(&b"canPause"[..]);
        serializer.push_argument_bytes(&b"false"[..]);

        serializer.push_argument_bytes(&b"canMint"[..]);
        serializer.push_argument_bytes(&b"true"[..]);

        serializer.push_argument_bytes(&b"canBurn"[..]);
        serializer.push_argument_bytes(&b"true"[..]);

        serializer.push_argument_bytes(&b"canChangeOwner"[..]);
        serializer.push_argument_bytes(&b"false"[..]);

        serializer.push_argument_bytes(&b"canUpgrade"[..]);
        serializer.push_argument_bytes(&b"true"[..]);

        // save data for callback
        self.set_temporary_storage_esdt_operation(&self.get_tx_hash(), &EsdtOperation::Issue);

        self.send().async_call_raw(
            &Address::from(ESDT_SYSTEM_SC_ADDRESS_ARRAY),
            &BigUint::from(ESDT_ISSUE_COST),
            serializer.as_slice(),
        );
    }

    fn mint_esdt_token(&self, token_identifier: &TokenIdentifier, amount: &BigUint) {
        let mut serializer = HexCallDataSerializer::new(ESDT_MINT_STRING);
        serializer.push_argument_bytes(token_identifier.as_slice());
        serializer.push_argument_bytes(&amount.to_bytes_be());

        // save data for callback
        self.set_temporary_storage_esdt_operation(
            &self.get_tx_hash(),
            &EsdtOperation::Mint(token_identifier.clone(), amount.clone()),
        );

        self.send().async_call_raw(
            &Address::from(ESDT_SYSTEM_SC_ADDRESS_ARRAY),
            &BigUint::zero(),
            serializer.as_slice(),
        );
    }

    fn transfer_esdt_token(
        &self,
        from: &Address,
        to: &Address,
        token_identifier: &TokenIdentifier,
        amount: &BigUint,
    ) {
        self.decrease_esdt_token_balance(&token_identifier, &amount);

        let data = [b"transfer from ", from.as_bytes()].concat();
        self.send()
            .direct_esdt_via_transf_exec(&to, token_identifier.as_slice(), &amount, &data);
    }

    fn increase_esdt_token_balance(&self, token_identifier: &TokenIdentifier, amount: &BigUint) {
        let mut total_balance = self.get_esdt_token_balance(token_identifier);
        total_balance += amount;
        self.set_esdt_token_balance(token_identifier, amount);
    }

    fn decrease_esdt_token_balance(&self, token_identifier: &TokenIdentifier, amount: &BigUint) {
        let mut total_balance = self.get_esdt_token_balance(token_identifier);
        total_balance -= amount;
        self.set_esdt_token_balance(token_identifier, amount);
    }

    fn set_esdt_token_balance(&self, token_identifier: &TokenIdentifier, amount: &BigUint) {
        self.esdt_token_balance()
            .insert(token_identifier.clone(), amount.clone());
    }

    // callbacks

    #[callback_raw]
    fn callback_raw(&self, #[var_args] result: AsyncCallResult<VarArgs<BoxedBytes>>) {
        let success = matches!(result, AsyncCallResult::Ok(_));
        let original_tx_hash = self.get_tx_hash();

        let esdt_operation = self.get_temporary_storage_esdt_operation(&original_tx_hash);
        match esdt_operation {
            EsdtOperation::None => return,
            EsdtOperation::Issue => self.perform_esdt_issue_callback(success),
            EsdtOperation::Mint(token_identifier, amount) => {
                self.perform_esdt_mint_callback(success, &token_identifier, &amount)
            }
        };

        self.clear_temporary_storage_esdt_operation(&original_tx_hash);
    }

    fn perform_esdt_issue_callback(&self, success: bool) {
        // callback is called with ESDTTransfer of the newly issued token, with the amount requested,
        // so we can get the token identifier and amount from the call data
        let token_identifier = self.call_value().token();
        let initial_supply = self.call_value().esdt_value();

        if success {
            self.esdt_token_balance()
                .insert(token_identifier, initial_supply);
        }

        // nothing to do in case of error
    }

    fn perform_esdt_mint_callback(
        &self,
        success: bool,
        token_identifier: &TokenIdentifier,
        amount: &BigUint,
    ) {
        if success {
            self.increase_esdt_token_balance(token_identifier, amount);
        }

        // nothing to do in case of error
    }

    // storage

    // list of available tokens

    #[storage_mapper("esdtTokenBalance")]
    fn esdt_token_balance(&self) -> MapMapper<Self::Storage, TokenIdentifier, BigUint>;

    // temporary storage for ESDT operations. Used in callback.

    #[storage_get("temporaryStorageEsdtOperation")]
    fn get_temporary_storage_esdt_operation(
        &self,
        original_tx_hash: &H256,
    ) -> EsdtOperation<BigUint>;

    #[storage_set("temporaryStorageEsdtOperation")]
    fn set_temporary_storage_esdt_operation(
        &self,
        original_tx_hash: &H256,
        esdt_operation: &EsdtOperation<BigUint>,
    );

    #[storage_clear("temporaryStorageEsdtOperation")]
    fn clear_temporary_storage_esdt_operation(&self, original_tx_hash: &H256);
}
