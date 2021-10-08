elrond_wasm::imports!();

use crate::user_role::UserRole;
use eth_address::EthAddress;

use fee_estimator_module::ProxyTrait as _;
use token_module::ProxyTrait as _;

#[elrond_wasm_derive::module]
pub trait SetupModule:
    crate::multisig_general::MultisigGeneralModule
    + crate::storage::StorageModule
    + crate::util::UtilModule
{
    #[init]
    fn init(
        &self,
        required_stake: Self::BigUint,
        slash_amount: Self::BigUint,
        quorum: usize,
        #[var_args] board: VarArgs<Address>,
    ) -> SCResult<()> {
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

        self.num_board_members()
            .update(|nr_board_members| *nr_board_members += board.len());

        require!(
            slash_amount <= required_stake,
            "slash amount must be less than or equal to required stake"
        );
        self.required_stake_amount().set(&required_stake);
        self.slash_amount().set(&slash_amount);

        Ok(())
    }

    #[only_owner]
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
        #[var_args] token_whitelist: VarArgs<TokenIdentifier>,
    ) -> SCResult<()> {
        // since contracts can either be all deployed or none,
        // it's sufficient to check only for one of them
        require!(
            self.egld_esdt_swap_address().is_empty(),
            "This function was called already."
        );

        let mut all_tokens = token_whitelist.into_vec();
        all_tokens.push(wrapped_egld_token_id.clone());

        let gas_per_deploy = self.blockchain().get_gas_left() / 3;

        // eGLD ESDT swap deploy

        let opt_egld_esdt_swap_address = self
            .setup_egld_esdt_swap_proxy()
            .init(wrapped_egld_token_id)
            .with_gas_limit(gas_per_deploy)
            .deploy_contract(&egld_esdt_swap_code, CodeMetadata::UPGRADEABLE);

        let egld_esdt_swap_address =
            opt_egld_esdt_swap_address.ok_or("EgldEsdtSwap deploy failed")?;
        self.egld_esdt_swap_address().set(&egld_esdt_swap_address);

        // Multi-transfer ESDT deploy

        let opt_multi_transfer_esdt_address = self
            .setup_multi_transfer_esdt_proxy(Address::zero())
            .init(
                price_aggregator_contract_address.clone(),
                multi_transfer_esdt_eth_tx_gas_limit,
                all_tokens.clone().into(),
            )
            .with_gas_limit(gas_per_deploy)
            .deploy_contract(&multi_transfer_esdt_code, CodeMetadata::UPGRADEABLE);

        let multi_transfer_esdt_address =
            opt_multi_transfer_esdt_address.ok_or("MultiTransferEsdt deploy failed")?;
        self.multi_transfer_esdt_address()
            .set(&multi_transfer_esdt_address);

        // ESDT Safe deploy

        let opt_esdt_safe_address = self
            .setup_esdt_safe_proxy(Address::zero())
            .init(
                price_aggregator_contract_address,
                esdt_safe_eth_tx_gas_limit,
                all_tokens.into(),
            )
            .with_gas_limit(gas_per_deploy)
            .deploy_contract(&esdt_safe_code, CodeMetadata::UPGRADEABLE);

        let esdt_safe_address = opt_esdt_safe_address.ok_or("EsdtSafe deploy failed")?;
        self.esdt_safe_address().set(&esdt_safe_address);

        // is set only so we don't have to check for "empty" on the very first call
        // trying to deserialize a tuple from an empty storage entry would crash
        self.statuses_after_execution().set(&(0, Vec::new()));

        Ok(())
    }

    #[only_owner]
    #[endpoint(upgradeChildContract)]
    fn upgrade_child_contract(
        &self,
        sc_address: Address,
        new_code: BoxedBytes,
        #[var_args] init_args: VarArgs<BoxedBytes>,
    ) -> SCResult<()> {
        let gas = self.blockchain().get_gas_left() / 2;
        let args = (init_args.into_vec().as_slice()).into();

        self.send().upgrade_contract(
            &sc_address,
            gas,
            &Self::BigUint::zero(),
            &new_code,
            CodeMetadata::UPGRADEABLE,
            &args,
        );

        Ok(())
    }

    #[only_owner]
    #[endpoint]
    fn pause(&self) -> SCResult<()> {
        self.pause_status().set(&true);

        Ok(())
    }

    #[only_owner]
    #[endpoint]
    fn unpause(&self) -> SCResult<()> {
        self.pause_status().set(&false);

        Ok(())
    }

    #[only_owner]
    #[endpoint(addBoardMember)]
    fn add_board_member(&self, board_member: Address) -> SCResult<()> {
        self.change_user_role(board_member, UserRole::BoardMember);

        Ok(())
    }

    #[only_owner]
    #[endpoint(addProposer)]
    fn add_proposer(&self, proposer: Address) -> SCResult<()> {
        self.change_user_role(proposer, UserRole::Proposer);

        // validation required for the scenario when a board member becomes a proposer
        require!(
            self.quorum().get() <= self.num_board_members().get(),
            "quorum cannot exceed board size"
        );

        Ok(())
    }

    #[only_owner]
    #[endpoint(removeUser)]
    fn remove_user(&self, user: Address) -> SCResult<()> {
        self.change_user_role(user, UserRole::None);
        let num_board_members = self.num_board_members().get();
        let num_proposers = self.num_proposers().get();
        require!(
            num_board_members + num_proposers > 0,
            "cannot remove all board members and proposers"
        );
        require!(
            self.quorum().get() <= num_board_members,
            "quorum cannot exceed board size"
        );

        Ok(())
    }

    #[only_owner]
    #[endpoint(slashBoardMember)]
    fn slash_board_member(&self, board_member: Address) -> SCResult<()> {
        self.change_user_role(board_member.clone(), UserRole::None);
        let num_board_members = self.num_board_members().get();
        let num_proposers = self.num_proposers().get();

        require!(
            num_board_members + num_proposers > 0,
            "cannot remove all board members and proposers"
        );
        require!(
            self.quorum().get() <= num_board_members,
            "quorum cannot exceed board size"
        );

        let slash_amount = self.slash_amount().get();

        // remove slashed amount from user stake amountself
        self.amount_staked(&board_member)
            .update(|stake| *stake -= &slash_amount);

        // add it to total slashed amount pool
        self.slashed_tokens_amount()
            .update(|slashed_amt| *slashed_amt += slash_amount);

        Ok(())
    }

    #[only_owner]
    #[endpoint(changeQuorum)]
    fn change_quorum(&self, new_quorum: usize) -> SCResult<()> {
        require!(
            new_quorum <= self.num_board_members().get(),
            "quorum cannot exceed board size"
        );
        self.quorum().set(&new_quorum);

        Ok(())
    }

    #[only_owner]
    #[endpoint(addMapping)]
    fn add_mapping(&self, erc20_address: EthAddress, token_id: TokenIdentifier) -> SCResult<()> {
        require!(
            self.erc20_address_for_token_id(&token_id).is_empty(),
            "Mapping already exists for ERC20 token"
        );
        require!(
            self.token_id_for_erc20_address(&erc20_address).is_empty(),
            "Mapping already exists for token id"
        );

        self.erc20_address_for_token_id(&token_id)
            .set(&erc20_address);
        self.token_id_for_erc20_address(&erc20_address)
            .set(&token_id);

        Ok(())
    }

    #[only_owner]
    #[endpoint(clearMapping)]
    fn clear_mapping(&self, erc20_address: EthAddress, token_id: TokenIdentifier) -> SCResult<()> {
        require!(
            !self.erc20_address_for_token_id(&token_id).is_empty(),
            "Mapping does not exist for ERC20 token"
        );
        require!(
            !self.token_id_for_erc20_address(&erc20_address).is_empty(),
            "Mapping does not exist for token id"
        );

        self.erc20_address_for_token_id(&token_id).clear();
        self.token_id_for_erc20_address(&erc20_address).clear();

        Ok(())
    }

    #[only_owner]
    #[endpoint(changeFeeEstimatorContractAddress)]
    fn change_fee_estimator_contract_address(&self, new_address: Address) {
        self.setup_esdt_safe_proxy(self.esdt_safe_address().get())
            .set_fee_estimator_contract_address(new_address.clone())
            .execute_on_dest_context();

        self.setup_multi_transfer_esdt_proxy(self.esdt_safe_address().get())
            .set_fee_estimator_contract_address(new_address)
            .execute_on_dest_context();
    }

    #[only_owner]
    #[endpoint(changeDefaultPricePerGwei)]
    fn change_default_price_per_gwei(&self, token_id: TokenIdentifier, new_value: Self::BigUint) {
        self.setup_esdt_safe_proxy(self.esdt_safe_address().get())
            .set_default_price_per_gwei(token_id.clone(), new_value.clone())
            .execute_on_dest_context();

        self.setup_multi_transfer_esdt_proxy(self.esdt_safe_address().get())
            .set_default_price_per_gwei(token_id, new_value)
            .execute_on_dest_context();
    }

    #[only_owner]
    #[endpoint(esdtSafeAddTokenToWhitelist)]
    fn esdt_safe_add_token_to_whitelist(
        &self,
        token_id: TokenIdentifier,
        #[var_args] opt_default_value_in_dollars: OptionalArg<Self::BigUint>,
    ) {
        self.setup_esdt_safe_proxy(self.esdt_safe_address().get())
            .add_token_to_whitelist(token_id, opt_default_value_in_dollars)
            .execute_on_dest_context();
    }

    #[only_owner]
    #[endpoint(esdtSafeRemoveTokenFromWhitelist)]
    fn esdt_safe_remove_token_from_whitelist(&self, token_id: TokenIdentifier) {
        self.setup_esdt_safe_proxy(self.esdt_safe_address().get())
            .remove_token_from_whitelist(token_id)
            .execute_on_dest_context();
    }

    #[only_owner]
    #[endpoint(esdtSafeSetMaxTxBatchSize)]
    fn esdt_safe_set_max_tx_batch_size(&self, new_max_tx_batch_size: usize) {
        self.setup_esdt_safe_proxy(self.esdt_safe_address().get())
            .set_max_tx_batch_size(new_max_tx_batch_size)
            .execute_on_dest_context();
    }

    #[only_owner]
    #[endpoint(esdtSafeSetMaxTxBatchBlockDuration)]
    fn esdt_safe_set_max_tx_batch_block_duration(&self, new_max_tx_batch_block_duration: u64) {
        self.setup_esdt_safe_proxy(self.esdt_safe_address().get())
            .set_max_tx_batch_block_duration(new_max_tx_batch_block_duration)
            .execute_on_dest_context();
    }

    #[only_owner]
    #[endpoint(multiTransferEsdtaddTokenToWhitelist)]
    fn multi_transfer_esdt_add_token_to_whitelist(
        &self,
        token_id: TokenIdentifier,
        #[var_args] opt_default_value_in_dollars: OptionalArg<Self::BigUint>,
    ) {
        self.setup_multi_transfer_esdt_proxy(self.multi_transfer_esdt_address().get())
            .add_token_to_whitelist(token_id, opt_default_value_in_dollars)
            .execute_on_dest_context();
    }

    #[only_owner]
    #[endpoint(multiTransferEsdtRemoveTokenFromWhitelist)]
    fn multi_transfer_esdt_remove_token_from_whitelist(&self, token_id: TokenIdentifier) {
        self.setup_multi_transfer_esdt_proxy(self.multi_transfer_esdt_address().get())
            .remove_token_from_whitelist(token_id)
            .execute_on_dest_context();
    }

    #[proxy]
    fn setup_egld_esdt_swap_proxy(&self) -> egld_esdt_swap::Proxy<Self::SendApi>;

    #[proxy]
    fn setup_esdt_safe_proxy(&self, sc_address: Address) -> esdt_safe::Proxy<Self::SendApi>;

    #[proxy]
    fn setup_multi_transfer_esdt_proxy(
        &self,
        sc_address: Address,
    ) -> multi_transfer_esdt::Proxy<Self::SendApi>;
}
