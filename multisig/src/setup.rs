elrond_wasm::imports!();

use crate::user_role::UserRole;
use eth_address::EthAddress;

use fee_estimator_module::ProxyTrait as _;
use token_module::ProxyTrait as _;

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
        esdt_safe_code: BoxedBytes,
        price_aggregator_contract_address: Address,
        esdt_safe_eth_tx_gas_limit: Self::BigUint,
        multi_transfer_esdt_eth_tx_gas_limit: Self::BigUint,
        wrapped_egld_token_id: TokenIdentifier,
        wrapped_eth_token_id: TokenIdentifier,
        #[var_args] token_whitelist: VarArgs<TokenIdentifier>,
    ) -> SCResult<()> {
        self.require_caller_owner()?;

        // since contracts can either be all deployed or none,
        // it's sufficient to check only for one of them
        require!(
            self.egld_esdt_swap_address().is_empty(),
            "This function was called already."
        );

        // must build the token whitelist ArgBuffer twice, as there is no way to clone it

        let mut arg_buffer_token_whitelist_esdt_safe = ArgBuffer::new();
        let mut arg_buffer_token_whitelist_multi_transfer_esdt = ArgBuffer::new();

        arg_buffer_token_whitelist_esdt_safe
            .push_argument_bytes(wrapped_egld_token_id.as_esdt_identifier());
        arg_buffer_token_whitelist_esdt_safe
            .push_argument_bytes(wrapped_eth_token_id.as_esdt_identifier());
        for token in &token_whitelist.0 {
            arg_buffer_token_whitelist_esdt_safe.push_argument_bytes(token.as_esdt_identifier());
        }

        arg_buffer_token_whitelist_multi_transfer_esdt
            .push_argument_bytes(wrapped_egld_token_id.as_esdt_identifier());
        arg_buffer_token_whitelist_multi_transfer_esdt
            .push_argument_bytes(wrapped_eth_token_id.as_esdt_identifier());
        for token in &token_whitelist.0 {
            arg_buffer_token_whitelist_multi_transfer_esdt
                .push_argument_bytes(token.as_esdt_identifier());
        }

        let gas_per_deploy = self.blockchain().get_gas_left() / 3;

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
            !egld_esdt_swap_address.is_zero(),
            "EgldEsdtSwap deploy failed"
        );
        self.egld_esdt_swap_address().set(&egld_esdt_swap_address);

        // Multi-transfer ESDT deploy

        let mut arg_buffer_multi_transfer_esdt = ArgBuffer::new();
        arg_buffer_multi_transfer_esdt
            .push_argument_bytes(price_aggregator_contract_address.as_bytes());
        arg_buffer_multi_transfer_esdt
            .push_argument_bytes(&multi_transfer_esdt_eth_tx_gas_limit.to_bytes_be());
        arg_buffer_multi_transfer_esdt =
            arg_buffer_multi_transfer_esdt.concat(arg_buffer_token_whitelist_multi_transfer_esdt);

        let multi_transfer_esdt_address = self.send().deploy_contract(
            gas_per_deploy,
            &Self::BigUint::zero(),
            &multi_transfer_esdt_code,
            CodeMetadata::DEFAULT,
            &arg_buffer_multi_transfer_esdt,
        );
        require!(
            !multi_transfer_esdt_address.is_zero(),
            "MultiTransferEsdt deploy failed"
        );
        self.multi_transfer_esdt_address()
            .set(&multi_transfer_esdt_address);

        // ESDT Safe deploy

        let mut esdt_safe_arg_buffer = ArgBuffer::new();
        esdt_safe_arg_buffer.push_argument_bytes(price_aggregator_contract_address.as_bytes());
        esdt_safe_arg_buffer.push_argument_bytes(&esdt_safe_eth_tx_gas_limit.to_bytes_be());
        esdt_safe_arg_buffer = esdt_safe_arg_buffer.concat(arg_buffer_token_whitelist_esdt_safe);

        let esdt_safe_address = self.send().deploy_contract(
            gas_per_deploy,
            &Self::BigUint::zero(),
            &esdt_safe_code,
            CodeMetadata::DEFAULT,
            &esdt_safe_arg_buffer,
        );
        require!(!esdt_safe_address.is_zero(), "EsdtSafe deploy failed");
        self.esdt_safe_address().set(&esdt_safe_address);

        Ok(())
    }

    // TODO: Upgrade endpoint for each child SC

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

    #[endpoint(changeFeeEstimatorContractAddress)]
    fn change_fee_estimator_contract_address(&self, new_address: Address) -> SCResult<()> {
        self.require_caller_owner()?;

        self.setup_esdt_safe_proxy(self.esdt_safe_address().get())
            .set_fee_estimator_contract_address(new_address.clone())
            .execute_on_dest_context();

        self.setup_multi_transfer_esdt_proxy(self.esdt_safe_address().get())
            .set_fee_estimator_contract_address(new_address)
            .execute_on_dest_context();

        Ok(())
    }

    #[endpoint(changeDefaultCostPerGwei)]
    fn change_default_cost_per_gwei(
        &self,
        token_id: TokenIdentifier,
        new_value: Self::BigUint,
    ) -> SCResult<()> {
        self.require_caller_owner()?;

        self.setup_esdt_safe_proxy(self.esdt_safe_address().get())
            .set_default_cost_per_gwei(token_id.clone(), new_value.clone())
            .execute_on_dest_context();

        self.setup_multi_transfer_esdt_proxy(self.esdt_safe_address().get())
            .set_default_cost_per_gwei(token_id, new_value)
            .execute_on_dest_context();

        Ok(())
    }

    #[endpoint(esdtSafeAddTokenToWhitelist)]
    fn esdt_safe_add_token_to_whitelist(
        &self,
        token_id: TokenIdentifier,
        #[var_args] opt_default_value_in_dollars: OptionalArg<Self::BigUint>,
    ) -> SCResult<()> {
        self.require_caller_owner()?;
        self.require_esdt_safe_deployed()?;

        self.setup_esdt_safe_proxy(self.esdt_safe_address().get())
            .add_token_to_whitelist(token_id, opt_default_value_in_dollars)
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

    #[endpoint(esdtSafeSetMinBlockNonceDiff)]
    fn esdt_safe_set_min_block_nonce_diff(&self, new_min_block_nonce_diff: u64) -> SCResult<()> {
        self.require_caller_owner()?;
        self.require_esdt_safe_deployed()?;

        self.setup_esdt_safe_proxy(self.esdt_safe_address().get())
            .set_min_block_nonce_diff(new_min_block_nonce_diff)
            .execute_on_dest_context();

        Ok(())
    }

    #[endpoint(multiTransferEsdtaddTokenToWhitelist)]
    fn multi_transfer_esdt_add_token_to_whitelist(
        &self,
        token_id: TokenIdentifier,
        #[var_args] opt_default_value_in_dollars: OptionalArg<Self::BigUint>,
    ) -> SCResult<()> {
        self.require_caller_owner()?;
        self.require_multi_transfer_esdt_deployed()?;

        self.setup_multi_transfer_esdt_proxy(self.multi_transfer_esdt_address().get())
            .add_token_to_whitelist(token_id, opt_default_value_in_dollars)
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
}
