elrond_wasm::imports!();

use crate::user_role::UserRole;
use eth_address::EthAddress;

use fee_estimator_module::ProxyTrait as _;
use token_module::ProxyTrait as _;

#[elrond_wasm::module]
pub trait SetupModule:
    crate::multisig_general::MultisigGeneralModule
    + crate::storage::StorageModule
    + crate::util::UtilModule
{

    #[only_owner]
    #[endpoint(deployChildContracts)]
    fn deploy_child_contracts(
        &self,
        multi_transfer_esdt_code: ManagedBuffer,
        esdt_safe_code: ManagedBuffer,
        price_aggregator_contract_address: ManagedAddress,
        esdt_safe_eth_tx_gas_limit: BigUint,
        multi_transfer_esdt_eth_tx_gas_limit: BigUint,
    ) -> SCResult<()> {
        // since contracts can either be all deployed or none,
        // it's sufficient to check only for one of them
        require!(
            self.esdt_safe_address().is_empty(),
            "This function was called already."
        );

        let gas_per_deploy = self.blockchain().get_gas_left() / 2;

        // Multi-transfer ESDT deploy

        let (multi_transfer_esdt_address, _) = self
            .setup_multi_transfer_esdt_proxy(ManagedAddress::zero())
            .init(
                price_aggregator_contract_address.clone(),
                multi_transfer_esdt_eth_tx_gas_limit,
            )
            .with_gas_limit(gas_per_deploy)
            .deploy_contract(&multi_transfer_esdt_code, CodeMetadata::UPGRADEABLE);

        self.multi_transfer_esdt_address()
            .set(&multi_transfer_esdt_address);

        // ESDT Safe deploy

        let (esdt_safe_address, _) = self
            .setup_esdt_safe_proxy(ManagedAddress::zero())
            .init(
                price_aggregator_contract_address,
                esdt_safe_eth_tx_gas_limit,
            )
            .with_gas_limit(gas_per_deploy)
            .deploy_contract(&esdt_safe_code, CodeMetadata::UPGRADEABLE);

        self.esdt_safe_address().set(&esdt_safe_address);

        // is set only so we don't have to check for "empty" on the very first call
        // trying to deserialize a tuple from an empty storage entry would crash
        self.statuses_after_execution()
            .set(&crate::storage::StatusesAfterExecution {
                block_executed: u64::MAX,
                batch_id: u64::MAX,
                statuses: ManagedVec::new(),
            });

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
    fn add_board_member(&self, board_member: ManagedAddress) -> SCResult<()> {
        self.change_user_role(board_member, UserRole::BoardMember);

        Ok(())
    }

    #[only_owner]
    #[endpoint(removeUser)]
    fn remove_user(&self, user: ManagedAddress) -> SCResult<()> {
        self.change_user_role(user, UserRole::None);
        let num_board_members = self.num_board_members().get();
        require!(
            num_board_members > 0,
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
    fn slash_board_member(&self, board_member: ManagedAddress) -> SCResult<()> {
        self.change_user_role(board_member.clone(), UserRole::None);
        let num_board_members = self.num_board_members().get();

        require!(num_board_members > 0, "cannot remove all board members");
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
    fn add_mapping(
        &self,
        erc20_address: EthAddress<Self::Api>,
        token_id: TokenIdentifier,
    ) -> SCResult<()> {
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
    fn clear_mapping(
        &self,
        erc20_address: EthAddress<Self::Api>,
        token_id: TokenIdentifier,
    ) -> SCResult<()> {
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
    fn change_fee_estimator_contract_address(&self, new_address: ManagedAddress) {
        self.setup_esdt_safe_proxy(self.esdt_safe_address().get())
            .set_fee_estimator_contract_address(new_address.clone())
            .execute_on_dest_context();

        self.setup_multi_transfer_esdt_proxy(self.multi_transfer_esdt_address().get())
            .set_fee_estimator_contract_address(new_address)
            .execute_on_dest_context();
    }

    #[only_owner]
    #[endpoint(changeDefaultPricePerGwei)]
    fn change_default_price_per_gas_unit(&self, token_id: TokenIdentifier, new_value: BigUint) {
        self.setup_esdt_safe_proxy(self.esdt_safe_address().get())
            .set_default_price_per_gas_unit(token_id.clone(), new_value.clone())
            .execute_on_dest_context();

        self.setup_multi_transfer_esdt_proxy(self.multi_transfer_esdt_address().get())
            .set_default_price_per_gas_unit(token_id, new_value)
            .execute_on_dest_context();
    }

    #[only_owner]
    #[endpoint(changeTokenTicker)]
    fn change_token_ticker(&self, token_id: TokenIdentifier, new_ticker: ManagedBuffer) {
        self.setup_esdt_safe_proxy(self.esdt_safe_address().get())
            .set_token_ticker(token_id.clone(), new_ticker.clone())
            .execute_on_dest_context();

        self.setup_multi_transfer_esdt_proxy(self.multi_transfer_esdt_address().get())
            .set_token_ticker(token_id, new_ticker)
            .execute_on_dest_context();
    }

    #[only_owner]
    #[endpoint(esdtSafeAddTokenToWhitelist)]
    fn esdt_safe_add_token_to_whitelist(
        &self,
        token_id: TokenIdentifier,
        ticker: ManagedBuffer,
        #[var_args] opt_default_value_in_dollars: OptionalArg<BigUint>,
    ) {
        self.setup_esdt_safe_proxy(self.esdt_safe_address().get())
            .add_token_to_whitelist(token_id, ticker, opt_default_value_in_dollars)
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
        ticker: ManagedBuffer,
        #[var_args] opt_default_value_in_dollars: OptionalArg<BigUint>,
    ) {
        self.setup_multi_transfer_esdt_proxy(self.multi_transfer_esdt_address().get())
            .add_token_to_whitelist(token_id, ticker, opt_default_value_in_dollars)
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
    fn setup_esdt_safe_proxy(&self, sc_address: ManagedAddress) -> esdt_safe::Proxy<Self::Api>;

    #[proxy]
    fn setup_multi_transfer_esdt_proxy(
        &self,
        sc_address: ManagedAddress,
    ) -> multi_transfer_esdt::Proxy<Self::Api>;
}
