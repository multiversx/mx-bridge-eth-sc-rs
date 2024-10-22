use multiversx_sc::imports::*;

use eth_address::EthAddress;
use sc_proxies::{
    bridge_proxy_contract_proxy, bridged_tokens_wrapper_proxy, esdt_safe_proxy,
    multi_transfer_esdt_proxy,
};

#[multiversx_sc::module]
pub trait SetupModule:
    crate::multisig_general::MultisigGeneralModule
    + crate::storage::StorageModule
    + crate::util::UtilModule
    + crate::events::EventsModule
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
        self.send_raw().upgrade_from_source_contract(
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
        self.add_mapping_event(erc20_address, token_id);
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
        self.clear_mapping_event(erc20_address, token_id);
    }

    #[only_owner]
    #[payable("*")]
    #[endpoint(initSupplyEsdtSafe)]
    fn init_supply_esdt_safe(&self, token_id: TokenIdentifier, amount: BigUint) {
        let esdt_safe_addr = self.esdt_safe_address().get();
        let (payment_token, payment_amount) = self.call_value().single_fungible_esdt();

        self.tx()
            .to(esdt_safe_addr)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .init_supply(token_id, amount)
            .single_esdt(&payment_token, 0, &payment_amount) // enforce only single FT transfer
            .sync_call();
    }

    #[only_owner]
    #[endpoint(initSupplyMintBurnEsdtSafe)]
    fn init_supply_mint_burn_esdt_safe(
        &self,
        token_id: TokenIdentifier,
        mint_amount: BigUint,
        burn_amount: BigUint,
    ) {
        let esdt_safe_addr = self.esdt_safe_address().get();

        self.tx()
            .to(esdt_safe_addr)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .init_supply_mint_burn(token_id, mint_amount, burn_amount)
            .sync_call();
    }

    #[only_owner]
    #[endpoint(changeFeeEstimatorContractAddress)]
    fn change_fee_estimator_contract_address(&self, new_address: ManagedAddress) {
        let esdt_safe_addr = self.esdt_safe_address().get();

        self.tx()
            .to(esdt_safe_addr)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .set_fee_estimator_contract_address(new_address)
            .sync_call();
    }

    /// Sets the gas limit being used for Ethereum transactions
    /// This is used in the EsdtSafe contract to determine the fee amount
    ///
    /// fee_amount = eth_gas_limit * price_per_gas_unit
    ///
    /// where price_per_gas_unit is queried from the aggregator (fee estimator SC)
    #[only_owner]
    #[endpoint(changeMultiversXToEthGasLimit)]
    fn change_multiversx_to_eth_gas_limit(&self, new_gas_limit: BigUint) {
        let esdt_safe_addr = self.esdt_safe_address().get();

        self.tx()
            .to(esdt_safe_addr)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .set_eth_tx_gas_limit(new_gas_limit)
            .sync_call();
    }

    /// Default price being used if the aggregator lacks a mapping for this token
    /// or the aggregator address is not set
    #[only_owner]
    #[endpoint(changeDefaultPricePerGasUnit)]
    fn change_default_price_per_gas_unit(&self, token_id: TokenIdentifier, new_value: BigUint) {
        let esdt_safe_addr = self.esdt_safe_address().get();

        self.tx()
            .to(esdt_safe_addr)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .set_default_price_per_gas_unit(token_id, new_value)
            .sync_call();
    }

    /// Token ticker being used when querying the aggregator for GWEI prices
    #[only_owner]
    #[endpoint(changeTokenTicker)]
    fn change_token_ticker(&self, token_id: TokenIdentifier, new_ticker: ManagedBuffer) {
        let esdt_safe_addr = self.esdt_safe_address().get();

        self.tx()
            .to(esdt_safe_addr)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .set_token_ticker(token_id, new_ticker)
            .sync_call();
    }

    #[only_owner]
    #[endpoint(esdtSafeAddTokenToWhitelist)]
    fn esdt_safe_add_token_to_whitelist(
        &self,
        token_id: &TokenIdentifier,
        ticker: ManagedBuffer,
        mint_burn_allowed: bool,
        is_native_token: bool,
        total_balance: &BigUint,
        mint_balance: &BigUint,
        burn_balance: &BigUint,
        opt_default_price_per_gas_unit: OptionalValue<BigUint>,
    ) {
        let esdt_safe_addr = self.esdt_safe_address().get();

        self.tx()
            .to(esdt_safe_addr)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .add_token_to_whitelist(
                token_id,
                ticker,
                mint_burn_allowed,
                is_native_token,
                total_balance,
                mint_balance,
                burn_balance,
                opt_default_price_per_gas_unit,
            )
            .sync_call();
    }

    #[only_owner]
    #[endpoint(setMultiTransferOnEsdtSafe)]
    fn set_multi_transfer_on_esdt_safe(&self) {
        let multi_transfer_esdt_address = self.multi_transfer_esdt_address().get();
        let esdt_safe_addr = self.esdt_safe_address().get();
        self.tx()
            .to(esdt_safe_addr)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .set_multi_transfer_contract_address(OptionalValue::Some(multi_transfer_esdt_address))
            .sync_call();
    }

    #[only_owner]
    #[endpoint(setEsdtSafeOnMultiTransfer)]
    fn set_esdt_safe_on_multi_transfer(&self) {
        let esdt_safe_address = self.esdt_safe_address().get();
        let multi_transfer_esdt_addr = self.multi_transfer_esdt_address().get();
        self.tx()
            .to(multi_transfer_esdt_addr)
            .typed(multi_transfer_esdt_proxy::MultiTransferEsdtProxy)
            .set_esdt_safe_contract_address(OptionalValue::Some(esdt_safe_address))
            .sync_call();
    }

    #[only_owner]
    #[endpoint(esdtSafeRemoveTokenFromWhitelist)]
    fn esdt_safe_remove_token_from_whitelist(&self, token_id: TokenIdentifier) {
        let esdt_safe_addr = self.esdt_safe_address().get();
        self.tx()
            .to(esdt_safe_addr)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .remove_token_from_whitelist(token_id)
            .sync_call();
    }

    /// Sets maximum batch size for the EsdtSafe SC.
    /// If a batch reaches this amount of transactions, it is considered full,
    /// and a new incoming transaction will be put into a new batch.
    #[only_owner]
    #[endpoint(esdtSafeSetMaxTxBatchSize)]
    fn esdt_safe_set_max_tx_batch_size(&self, new_max_tx_batch_size: usize) {
        let esdt_safe_addr = self.esdt_safe_address().get();
        self.tx()
            .to(esdt_safe_addr)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .set_max_tx_batch_size(new_max_tx_batch_size)
            .sync_call();
    }

    /// Sets the maximum block duration in which an EsdtSafe batch accepts transactions
    /// For a batch to be considered "full", it has to either reach `maxTxBatchSize` transactions,
    /// or have txBatchBlockDuration blocks pass since the first tx was added in the batch
    #[only_owner]
    #[endpoint(esdtSafeSetMaxTxBatchBlockDuration)]
    fn esdt_safe_set_max_tx_batch_block_duration(&self, new_max_tx_batch_block_duration: u64) {
        let esdt_safe_addr = self.esdt_safe_address().get();

        self.tx()
            .to(esdt_safe_addr)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .set_max_tx_batch_block_duration(new_max_tx_batch_block_duration)
            .sync_call();
    }

    /// Sets the maximum bridged amount for the token for the MultiversX -> Ethereum direction.
    /// Any attempt to transfer over this amount will be rejected.
    #[only_owner]
    #[endpoint(esdtSafeSetMaxBridgedAmountForToken)]
    fn esdt_safe_set_max_bridged_amount_for_token(
        &self,
        token_id: TokenIdentifier,
        max_amount: BigUint,
    ) {
        let esdt_safe_addr = self.esdt_safe_address().get();

        self.tx()
            .to(esdt_safe_addr)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .set_max_bridged_amount(token_id, max_amount)
            .sync_call();
    }

    /// Same as the function above, but for Ethereum -> MultiversX transactions.
    #[only_owner]
    #[endpoint(multiTransferEsdtSetMaxBridgedAmountForToken)]
    fn multi_transfer_esdt_set_max_bridged_amount_for_token(
        &self,
        token_id: TokenIdentifier,
        max_amount: BigUint,
    ) {
        let multi_transfer_esdt_addr = self.multi_transfer_esdt_address().get();
        self.tx()
            .to(multi_transfer_esdt_addr)
            .typed(multi_transfer_esdt_proxy::MultiTransferEsdtProxy)
            .set_max_bridged_amount(token_id, max_amount)
            .sync_call();
    }

    /// Any failed Ethereum -> MultiversX transactions are added into so-called "refund batches"
    /// This configures the size of a batch.
    #[only_owner]
    #[endpoint(multiTransferEsdtSetMaxRefundTxBatchSize)]
    fn multi_transfer_esdt_set_max_refund_tx_batch_size(&self, new_max_tx_batch_size: usize) {
        let multi_transfer_esdt_addr = self.multi_transfer_esdt_address().get();

        self.tx()
            .to(multi_transfer_esdt_addr)
            .typed(multi_transfer_esdt_proxy::MultiTransferEsdtProxy)
            .set_max_tx_batch_size(new_max_tx_batch_size)
            .sync_call();
    }

    /// Max block duration for refund batches. Default is "infinite" (u64::MAX)
    /// and only max batch size matters
    #[only_owner]
    #[endpoint(multiTransferEsdtSetMaxRefundTxBatchBlockDuration)]
    fn multi_transfer_esdt_set_max_refund_tx_batch_block_duration(
        &self,
        new_max_tx_batch_block_duration: u64,
    ) {
        let multi_transfer_esdt_addr = self.multi_transfer_esdt_address().get();

        self.tx()
            .to(multi_transfer_esdt_addr)
            .typed(multi_transfer_esdt_proxy::MultiTransferEsdtProxy)
            .set_max_tx_batch_block_duration(new_max_tx_batch_block_duration)
            .sync_call();
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
        let multi_transfer_esdt_addr = self.multi_transfer_esdt_address().get();

        self.tx()
            .to(multi_transfer_esdt_addr)
            .typed(multi_transfer_esdt_proxy::MultiTransferEsdtProxy)
            .set_wrapping_contract_address(opt_wrapping_contract_address)
            .sync_call();
    }

    // Pause/Unpause endpoints

    #[only_owner]
    #[endpoint(pauseAllChildContracts)]
    fn pause_all_child_contracts(&self) {
        self.pause_endpoint();
        self.pause_esdt_safe();
        self.pause_bridge_proxy();
        self.pause_bridged_tokens_wrapper();
        self.pause_multi_transfer_esdt();
    }

    #[only_owner]
    #[endpoint(unpauseAllChildContracts)]
    fn unpause_all_child_contracts(&self) {
        self.unpause_endpoint();
        self.unpause_esdt_safe();
        self.unpause_bridge_proxy();
        self.unpause_bridged_tokens_wrapper();
        self.unpause_multi_transfer_esdt();
    }

    #[only_owner]
    #[endpoint(pauseEsdtSafe)]
    fn pause_esdt_safe(&self) {
        let esdt_safe_addr = self.esdt_safe_address().get();

        self.tx()
            .to(esdt_safe_addr)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .pause_endpoint()
            .sync_call();

        self.pause_esdt_safe_event();
    }

    #[only_owner]
    #[endpoint(unpauseEsdtSafe)]
    fn unpause_esdt_safe(&self) {
        let esdt_safe_addr = self.esdt_safe_address().get();

        self.tx()
            .to(esdt_safe_addr)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .unpause_endpoint()
            .sync_call();
        self.unpause_esdt_safe_event();
    }

    #[only_owner]
    #[endpoint(pauseBridgeProxy)]
    fn pause_bridge_proxy(&self) {
        let proxy_addr = self.bridge_proxy_address().get();

        self.tx()
            .to(proxy_addr)
            .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
            .pause_endpoint()
            .sync_call();

        self.pause_bridge_proxy_event();
    }

    #[only_owner]
    #[endpoint(unpauseBridgeProxy)]
    fn unpause_bridge_proxy(&self) {
        let proxy_addr = self.bridge_proxy_address().get();

        self.tx()
            .to(proxy_addr)
            .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
            .unpause_endpoint()
            .sync_call();

        self.unpause_bridge_proxy_event();
    }

    #[only_owner]
    #[endpoint(pauseBridgedTokensWrapper)]
    fn pause_bridged_tokens_wrapper(&self) {
        let proxy_addr = self.bridged_tokens_wrapper_address().get();

        self.tx()
            .to(proxy_addr)
            .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
            .pause_endpoint()
            .sync_call();

        self.pause_bridged_tokens_wrapper_event();
    }

    #[only_owner]
    #[endpoint(unpauseBridgedTokensWrapper)]
    fn unpause_bridged_tokens_wrapper(&self) {
        let proxy_addr = self.bridged_tokens_wrapper_address().get();

        self.tx()
            .to(proxy_addr)
            .typed(bridged_tokens_wrapper_proxy::BridgedTokensWrapperProxy)
            .unpause_endpoint()
            .sync_call();

        self.unpause_bridged_tokens_wrapper_event();
    }

    #[only_owner]
    #[endpoint(pauseMultiTransferEsdt)]
    fn pause_multi_transfer_esdt(&self) {
        let proxy_addr = self.multi_transfer_esdt_address().get();

        self.tx()
            .to(proxy_addr)
            .typed(multi_transfer_esdt_proxy::MultiTransferEsdtProxy)
            .pause_endpoint()
            .sync_call();

        self.pause_multi_transfer_esdt_event();
    }

    #[only_owner]
    #[endpoint(unpauseMultiTransferEsdt)]
    fn unpause_multi_transfer_esdt(&self) {
        let proxy_addr = self.multi_transfer_esdt_address().get();

        self.tx()
            .to(proxy_addr)
            .typed(multi_transfer_esdt_proxy::MultiTransferEsdtProxy)
            .unpause_endpoint()
            .sync_call();

        self.unpause_multi_transfer_esdt_event();
    }
}
