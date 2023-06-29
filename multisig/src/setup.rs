multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use eth_address::EthAddress;

use fee_estimator_module::ProxyTrait as _;
use max_bridged_amount_module::ProxyTrait as _;
use multi_transfer_esdt::ProxyTrait as _;
use multiversx_sc_modules::pause::ProxyTrait as _;
use token_module::ProxyTrait as _;
use tx_batch_module::ProxyTrait as _;

#[multiversx_sc::module]
pub trait SetupModule:
    crate::multisig_general::MultisigGeneralModule
    + crate::storage::StorageModule
    + crate::util::UtilModule
    + multiversx_sc_modules::pause::PauseModule
{
    #[only_owner]
    #[endpoint(upgradeChildContractFromSource)]
    fn upgrade_child_contract_from_source(
        &self,
        child_sc_address: ManagedAddress,
        source_address: ManagedAddress,
        is_payable: bool,
        init_args: MultiValueEncoded<ManagedBuffer>,
    ) {
        let mut metadata = CodeMetadata::UPGRADEABLE;
        if is_payable {
            // TODO: Replace with PayableBySc when it's available
            metadata |= CodeMetadata::PAYABLE;
        }

        let gas = self.blockchain().get_gas_left();
        Self::Api::send_api_impl().upgrade_from_source_contract(
            &child_sc_address,
            gas,
            &BigUint::zero(),
            &source_address,
            metadata,
            &init_args.to_arg_buffer(),
        );
    }

    #[only_owner]
    #[endpoint(addBoardMember)]
    fn add_board_member_endpoint(&self, board_member: ManagedAddress) {
        self.add_board_member(&board_member);
    }

    #[only_owner]
    #[endpoint(removeUser)]
    fn remove_user(&self, board_member: ManagedAddress) {
        self.remove_board_member(&board_member);
        let num_board_members = self.num_board_members().get();
        require!(num_board_members > 0, "cannot remove all board members");
        require!(
            self.quorum().get() <= num_board_members,
            "quorum cannot exceed board size"
        );
    }

    /// Cuts a fixed amount from a board member's stake.
    /// This should be used only in cases where the board member
    /// is being actively malicious.
    ///
    /// After stake is cut, the board member would have to stake again
    /// to be able to sign actions.
    #[only_owner]
    #[endpoint(slashBoardMember)]
    fn slash_board_member(&self, board_member: ManagedAddress) {
        self.remove_user(board_member.clone());

        let slash_amount = self.slash_amount().get();

        // remove slashed amount from user stake amountself
        self.amount_staked(&board_member)
            .update(|stake| *stake -= &slash_amount);

        // add it to total slashed amount pool
        self.slashed_tokens_amount()
            .update(|slashed_amt| *slashed_amt += slash_amount);
    }

    #[only_owner]
    #[endpoint(changeQuorum)]
    fn change_quorum(&self, new_quorum: usize) {
        require!(
            new_quorum <= self.num_board_members().get(),
            "quorum cannot exceed board size"
        );
        self.quorum().set(new_quorum);
    }

    /// Maps an ESDT token to an ERC20 address. Used by relayers.
    #[only_owner]
    #[endpoint(addMapping)]
    fn add_mapping(&self, erc20_address: EthAddress<Self::Api>, token_id: TokenIdentifier) {
        require!(
            self.erc20_address_for_token_id(&token_id).is_empty(),
            "Mapping already exists for token ID"
        );
        require!(
            self.token_id_for_erc20_address(&erc20_address).is_empty(),
            "Mapping already exists for ERC20 token"
        );

        self.erc20_address_for_token_id(&token_id)
            .set(&erc20_address);
        self.token_id_for_erc20_address(&erc20_address)
            .set(&token_id);
    }

    #[only_owner]
    #[endpoint(clearMapping)]
    fn clear_mapping(&self, erc20_address: EthAddress<Self::Api>, token_id: TokenIdentifier) {
        require!(
            !self.erc20_address_for_token_id(&token_id).is_empty(),
            "Mapping does not exist for ERC20 token"
        );
        require!(
            !self.token_id_for_erc20_address(&erc20_address).is_empty(),
            "Mapping does not exist for token id"
        );

        let mapped_erc_20 = self.erc20_address_for_token_id(&token_id).get();
        let mapped_token_id = self.token_id_for_erc20_address(&erc20_address).get();

        require!(
            erc20_address.raw_addr == mapped_erc_20.raw_addr && token_id == mapped_token_id,
            "Invalid mapping"
        );

        self.erc20_address_for_token_id(&token_id).clear();
        self.token_id_for_erc20_address(&erc20_address).clear();
    }

    #[only_owner]
    #[endpoint(pauseEsdtSafe)]
    fn pause_esdt_safe(&self) {
        let _: IgnoreValue = self
            .get_esdt_safe_proxy_instance()
            .pause_endpoint()
            .execute_on_dest_context();
    }

    #[only_owner]
    #[endpoint(unpauseEsdtSafe)]
    fn unpause_esdt_safe(&self) {
        let _: IgnoreValue = self
            .get_esdt_safe_proxy_instance()
            .unpause_endpoint()
            .execute_on_dest_context();
    }

    #[only_owner]
    #[endpoint(changeFeeEstimatorContractAddress)]
    fn change_fee_estimator_contract_address(&self, new_address: ManagedAddress) {
        let _: IgnoreValue = self
            .get_esdt_safe_proxy_instance()
            .set_fee_estimator_contract_address(new_address)
            .execute_on_dest_context();
    }

    /// Sets the gas limit being used for Ethereum transactions
    /// This is used in the EsdtSafe contract to determine the fee amount
    ///
    /// fee_amount = eth_gas_limit * price_per_gas_unit
    ///
    /// where price_per_gas_unit is queried from the aggregator (fee estimator SC)
    #[only_owner]
    #[endpoint(changeElrondToEthGasLimit)]
    fn change_elrond_to_eth_gas_limit(&self, new_gas_limit: BigUint) {
        let _: IgnoreValue = self
            .get_esdt_safe_proxy_instance()
            .set_eth_tx_gas_limit(new_gas_limit)
            .execute_on_dest_context();
    }

    /// Default price being used if the aggregator lacks a mapping for this token
    /// or the aggregator address is not set
    #[only_owner]
    #[endpoint(changeDefaultPricePerGasUnit)]
    fn change_default_price_per_gas_unit(&self, token_id: TokenIdentifier, new_value: BigUint) {
        let _: IgnoreValue = self
            .get_esdt_safe_proxy_instance()
            .set_default_price_per_gas_unit(token_id, new_value)
            .execute_on_dest_context();
    }

    /// Token ticker being used when querying the aggregator for GWEI prices
    #[only_owner]
    #[endpoint(changeTokenTicker)]
    fn change_token_ticker(&self, token_id: TokenIdentifier, new_ticker: ManagedBuffer) {
        let _: IgnoreValue = self
            .get_esdt_safe_proxy_instance()
            .set_token_ticker(token_id, new_ticker)
            .execute_on_dest_context();
    }

    #[only_owner]
    #[endpoint(esdtSafeAddTokenToWhitelist)]
    fn esdt_safe_add_token_to_whitelist(
        &self,
        token_id: TokenIdentifier,
        ticker: ManagedBuffer,
        opt_default_price_per_gas_unit: OptionalValue<BigUint>,
    ) {
        let _: IgnoreValue = self
            .get_esdt_safe_proxy_instance()
            .add_token_to_whitelist(token_id, ticker, opt_default_price_per_gas_unit)
            .execute_on_dest_context();
    }

    #[only_owner]
    #[endpoint(esdtSafeRemoveTokenFromWhitelist)]
    fn esdt_safe_remove_token_from_whitelist(&self, token_id: TokenIdentifier) {
        let _: IgnoreValue = self
            .get_esdt_safe_proxy_instance()
            .remove_token_from_whitelist(token_id)
            .execute_on_dest_context();
    }

    /// Sets maximum batch size for the EsdtSafe SC.
    /// If a batch reaches this amount of transactions, it is considered full,
    /// and a new incoming transaction will be put into a new batch.
    #[only_owner]
    #[endpoint(esdtSafeSetMaxTxBatchSize)]
    fn esdt_safe_set_max_tx_batch_size(&self, new_max_tx_batch_size: usize) {
        let _: IgnoreValue = self
            .get_esdt_safe_proxy_instance()
            .set_max_tx_batch_size(new_max_tx_batch_size)
            .execute_on_dest_context();
    }

    /// Sets the maximum block duration in which an EsdtSafe batch accepts transactions
    /// For a batch to be considered "full", it has to either reach `maxTxBatchSize` transactions,
    /// or have txBatchBlockDuration blocks pass since the first tx was added in the batch
    #[only_owner]
    #[endpoint(esdtSafeSetMaxTxBatchBlockDuration)]
    fn esdt_safe_set_max_tx_batch_block_duration(&self, new_max_tx_batch_block_duration: u64) {
        let _: IgnoreValue = self
            .get_esdt_safe_proxy_instance()
            .set_max_tx_batch_block_duration(new_max_tx_batch_block_duration)
            .execute_on_dest_context();
    }

    /// Sets the maximum bridged amount for the token for the Elrond -> Ethereum direction.
    /// Any attempt to transfer over this amount will be rejected.
    #[only_owner]
    #[endpoint(esdtSafeSetMaxBridgedAmountForToken)]
    fn esdt_safe_set_max_bridged_amount_for_token(
        &self,
        token_id: TokenIdentifier,
        max_amount: BigUint,
    ) {
        let _: IgnoreValue = self
            .get_esdt_safe_proxy_instance()
            .set_max_bridged_amount(token_id, max_amount)
            .execute_on_dest_context();
    }

    /// Same as the function above, but for Ethereum -> Elrond transactions.
    #[only_owner]
    #[endpoint(multiTransferEsdtSetMaxBridgedAmountForToken)]
    fn multi_transfer_esdt_set_max_bridged_amount_for_token(
        &self,
        token_id: TokenIdentifier,
        max_amount: BigUint,
    ) {
        let _: IgnoreValue = self
            .get_multi_transfer_esdt_proxy_instance()
            .set_max_bridged_amount(token_id, max_amount)
            .execute_on_dest_context();
    }

    /// Any failed Ethereum -> Elrond transactions are added into so-called "refund batches"
    /// This configures the size of a batch.
    #[only_owner]
    #[endpoint(multiTransferEsdtSetMaxRefundTxBatchSize)]
    fn multi_transfer_esdt_set_max_refund_tx_batch_size(&self, new_max_tx_batch_size: usize) {
        let _: IgnoreValue = self
            .get_multi_transfer_esdt_proxy_instance()
            .set_max_tx_batch_size(new_max_tx_batch_size)
            .execute_on_dest_context();
    }

    /// Max block duration for refund batches. Default is "infinite" (u64::MAX)
    /// and only max batch size matters
    #[only_owner]
    #[endpoint(multiTransferEsdtSetMaxRefundTxBatchBlockDuration)]
    fn multi_transfer_esdt_set_max_refund_tx_batch_block_duration(
        &self,
        new_max_tx_batch_block_duration: u64,
    ) {
        let _: IgnoreValue = self
            .get_multi_transfer_esdt_proxy_instance()
            .set_max_tx_batch_block_duration(new_max_tx_batch_block_duration)
            .execute_on_dest_context();
    }

    /// Sets the wrapping contract address.
    /// This contract is used to map multiple tokens to a universal one.
    /// Useful in cases where a single token (USDC for example)
    /// is being transferred from multiple chains.
    ///
    /// They will all have different token IDs, but can be swapped 1:1 in the wrapping SC.
    /// The wrapping is done automatically, so the user only receives the universal token.
    #[only_owner]
    #[endpoint(multiTransferEsdtSetWrappingContractAddress)]
    fn multi_transfer_esdt_set_wrapping_contract_address(
        &self,
        opt_wrapping_contract_address: OptionalValue<ManagedAddress>,
    ) {
        let _: IgnoreValue = self
            .get_multi_transfer_esdt_proxy_instance()
            .set_wrapping_contract_address(opt_wrapping_contract_address)
            .execute_on_dest_context();
    }
}
