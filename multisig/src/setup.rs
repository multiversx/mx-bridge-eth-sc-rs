elrond_wasm::imports!();

use crate::user_role::UserRole;
use eth_address::EthAddress;

#[elrond_wasm_derive::module]
pub trait SetupModule: crate::storage::StorageModule + crate::util::UtilModule {
    #[init]
    fn init(
        &self,
        required_stake: Self::BigUint,
        slash_amount: Self::BigUint,
        quorum: usize,
        #[var_args] board: VarArgs<Address>,
    ) -> SCResult<()> {
        require!(
            !board.is_empty(),
            "board cannot be empty on init, no-one would be able to propose"
        );
        require!(quorum <= board.len(), "quorum cannot exceed board size");
        self.quorum().set(&quorum);

        let mut duplicates = false;
        self.user_mapper()
            .get_or_create_users(board.as_slice(), |user_id, new_user| {
                if !new_user {
                    duplicates = true;
                }
                self.set_user_id_to_role(user_id, UserRole::BoardMember);
            });
        require!(!duplicates, "duplicate board member");
        self.num_board_members().set(&board.len());

        require!(
            slash_amount <= required_stake,
            "slash amount must be less than or equal to required stake"
        );
        self.required_stake_amount().set(&required_stake);
        self.slash_amount().set(&slash_amount);

        Ok(())
    }

    #[endpoint(deployChildContracts)]
    fn deploy_child_contracts(
        &self,
        egld_esdt_swap_code: BoxedBytes,
        multi_transfer_esdt_code: BoxedBytes,
        ethereum_fee_prepay_code: BoxedBytes,
        esdt_safe_code: BoxedBytes,
        aggregator_address: Address,
        wrapped_egld_token_id: TokenIdentifier,
        #[var_args] token_whitelist: VarArgs<TokenIdentifier>,
    ) -> SCResult<()> {
        self.require_caller_owner()?;

        let zero_address = Address::zero();
        let mut arg_buffer_token_whitelist = ArgBuffer::new();

        arg_buffer_token_whitelist.push_argument_bytes(wrapped_egld_token_id.as_esdt_identifier());
        for token in token_whitelist.into_vec() {
            arg_buffer_token_whitelist.push_argument_bytes(token.as_esdt_identifier());
        }

        let gas_per_deploy = self.blockchain().get_gas_left() / 4;

        // eGLD ESDT swap deploy

        let mut egld_esdt_swap_arg_buffer = ArgBuffer::new();
        egld_esdt_swap_arg_buffer.push_argument_bytes(wrapped_egld_token_id.as_esdt_identifier());

        let egld_esdt_swap_address = self.send().deploy_contract(
            gas_per_deploy,
            &Self::BigUint::zero(),
            &egld_esdt_swap_code,
            CodeMetadata::DEFAULT,
            &egld_esdt_swap_arg_buffer,
        );
        require!(
            egld_esdt_swap_address != zero_address,
            "EgldEsdtSwap deploy failed"
        );
        self.egld_esdt_swap_address().set(&egld_esdt_swap_address);

        // Multi-transfer ESDT deploy

        let multi_transfer_esdt_address = self.send().deploy_contract(
            gas_per_deploy,
            &Self::BigUint::zero(),
            &multi_transfer_esdt_code,
            CodeMetadata::DEFAULT,
            &arg_buffer_token_whitelist,
        );
        require!(
            multi_transfer_esdt_address != zero_address,
            "MultiTransferEsdt deploy failed"
        );
        self.multi_transfer_esdt_address()
            .set(&multi_transfer_esdt_address);

        // Ethereum Fee Prepay deploy

        let mut ethereum_fee_prepay_arg_buffer = ArgBuffer::new();
        ethereum_fee_prepay_arg_buffer.push_argument_bytes(aggregator_address.as_bytes());

        let ethereum_fee_prepay_address = self.send().deploy_contract(
            gas_per_deploy,
            &Self::BigUint::zero(),
            &ethereum_fee_prepay_code,
            CodeMetadata::DEFAULT,
            &ethereum_fee_prepay_arg_buffer,
        );
        require!(
            ethereum_fee_prepay_address != zero_address,
            "EthereumFeePrepay deploy failed"
        );
        self.ethereum_fee_prepay_address()
            .set(&ethereum_fee_prepay_address);

        // ESDT Safe deploy

        let mut esdt_safe_arg_buffer = ArgBuffer::new();
        esdt_safe_arg_buffer.push_argument_bytes(ethereum_fee_prepay_address.as_bytes());
        esdt_safe_arg_buffer = esdt_safe_arg_buffer.concat(arg_buffer_token_whitelist);

        let esdt_safe_address = self.send().deploy_contract(
            gas_per_deploy,
            &Self::BigUint::zero(),
            &esdt_safe_code,
            CodeMetadata::DEFAULT,
            &esdt_safe_arg_buffer,
        );
        require!(esdt_safe_address != zero_address, "EsdtSafe deploy failed");
        self.esdt_safe_address().set(&esdt_safe_address);

        Ok(())
    }

    #[endpoint]
    fn pause(&self) -> SCResult<()> {
        self.require_caller_owner()?;

        self.pause_status().set(&true);

        Ok(())
    }

    #[endpoint]
    fn unpause(&self) -> SCResult<()> {
        self.require_caller_owner()?;

        self.pause_status().set(&false);

        Ok(())
    }

    #[endpoint(addMapping)]
    fn add_mapping(&self, erc20_address: EthAddress, token_id: TokenIdentifier) -> SCResult<()> {
        self.require_caller_owner()?;

        self.erc20_address_for_token_id(&token_id)
            .set(&erc20_address);
        self.token_id_for_erc20_address(&erc20_address)
            .set(&token_id);

        Ok(())
    }

    /// Add ESDT Safe to Ethereum Fee Prepay whitelist
    /// Can't be done in the previous step, as the contracts only exist after execution has finished
    #[endpoint(finishSetup)]
    fn finish_setup(&self) -> SCResult<()> {
        self.require_caller_owner()?;
        self.require_ethereum_fee_prepay_deployed()?;
        self.require_esdt_safe_deployed()?;

        let ethereum_fee_prepay_address = self.ethereum_fee_prepay_address().get();
        let esdt_safe_address = self.esdt_safe_address().get();

        self.setup_ethereum_fee_prepay_proxy(ethereum_fee_prepay_address)
            .add_to_whitelist(esdt_safe_address)
            .execute_on_dest_context();

        Ok(())
    }

    #[endpoint(esdtSafeAddTokenToWhitelist)]
    fn esdt_safe_add_token_to_whitelist(&self, token_id: TokenIdentifier) -> SCResult<()> {
        self.require_caller_owner()?;
        self.require_esdt_safe_deployed()?;

        self.setup_esdt_safe_proxy(self.esdt_safe_address().get())
            .add_token_to_whitelist(token_id)
            .execute_on_dest_context();

        Ok(())
    }

    #[endpoint(esdtSafeRemoveTokenFromWhitelist)]
    fn esdt_safe_remove_token_from_whitelist(&self, token_id: TokenIdentifier) -> SCResult<()> {
        self.require_caller_owner()?;
        self.require_esdt_safe_deployed()?;

        self.setup_esdt_safe_proxy(self.esdt_safe_address().get())
            .remove_token_from_whitelist(token_id)
            .execute_on_dest_context();

        Ok(())
    }

    #[endpoint(esdtSafeSetMaxTxBatchSize)]
    fn esdt_safe_set_max_tx_batch_size(&self, new_max_tx_batch_size: usize) -> SCResult<()> {
        self.require_caller_owner()?;
        self.require_esdt_safe_deployed()?;

        self.setup_esdt_safe_proxy(self.esdt_safe_address().get())
            .set_max_tx_batch_size(new_max_tx_batch_size)
            .execute_on_dest_context();

        Ok(())
    }

    #[endpoint(esdtSafeSetMaxBlockNonceDiff)]
    fn esdt_safe_set_max_block_nonce_diff(&self, new_max_block_nonce_diff: u64) -> SCResult<()> {
        self.require_caller_owner()?;
        self.require_esdt_safe_deployed()?;

        self.setup_esdt_safe_proxy(self.esdt_safe_address().get())
            .set_max_block_nonce_diff(new_max_block_nonce_diff)
            .execute_on_dest_context();

        Ok(())
    }

    #[endpoint(multiTransferEsdtaddTokenToWhitelist)]
    fn multi_transfer_esdt_add_token_to_whitelist(
        &self,
        token_id: TokenIdentifier,
    ) -> SCResult<()> {
        self.require_caller_owner()?;
        self.require_multi_transfer_esdt_deployed()?;

        self.setup_multi_transfer_esdt_proxy(self.multi_transfer_esdt_address().get())
            .add_token_to_whitelist(token_id)
            .execute_on_dest_context();

        Ok(())
    }

    #[endpoint(multiTransferEsdtRemoveTokenFromWhitelist)]
    fn multi_transfer_esdt_remove_token_from_whitelist(
        &self,
        token_id: TokenIdentifier,
    ) -> SCResult<()> {
        self.require_caller_owner()?;
        self.require_multi_transfer_esdt_deployed()?;

        self.setup_multi_transfer_esdt_proxy(self.multi_transfer_esdt_address().get())
            .remove_token_from_whitelist(token_id)
            .execute_on_dest_context();

        Ok(())
    }

    #[proxy]
    fn setup_esdt_safe_proxy(&self, sc_address: Address) -> esdt_safe::Proxy<Self::SendApi>;

    #[proxy]
    fn setup_multi_transfer_esdt_proxy(
        &self,
        sc_address: Address,
    ) -> multi_transfer_esdt::Proxy<Self::SendApi>;

    #[proxy]
    fn setup_ethereum_fee_prepay_proxy(
        &self,
        sc_address: Address,
    ) -> ethereum_fee_prepay::Proxy<Self::SendApi>;
}
