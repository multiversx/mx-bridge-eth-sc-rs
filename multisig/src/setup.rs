use multiversx_sc::imports::*;

use eth_address::EthAddress;
use sc_proxies::{bridge_proxy_contract_proxy, esdt_safe_proxy, multi_transfer_esdt_proxy};

const MAX_BOARD_MEMBERS: usize = 40;

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
        init_args: MultiValueEncoded<ManagedBuffer>,
    ) {
        let metadata = CodeMetadata::UPGRADEABLE;

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

        // remove slashed amount from user stake amount self
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
            new_quorum <= MAX_BOARD_MEMBERS && new_quorum > 1,
            "Quorum size not appropriate"
        );
        let total_users = self.user_mapper().get_user_count();
        let mut board_member_with_valid_stake: usize = 0;

        for user_id in 1..total_users + 1 {
            let user_role = self.user_id_to_role(user_id).get();

            if user_role.is_board_member() {
                if let Some(board_member_addr) = self.user_mapper().get_user_address(user_id) {
                    let amount_staked = self.amount_staked(&board_member_addr).get();
                    let required_stake_amount = self.required_stake_amount().get();
                    if amount_staked >= required_stake_amount {
                        board_member_with_valid_stake += 1;
                    }
                }
            }
        }

        require!(
            new_quorum <= board_member_with_valid_stake,
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
    #[endpoint(pauseEsdtSafe)]
    fn pause_esdt_safe(&self) {
        let esdt_safe_addr = self.esdt_safe_address().get();

        self.tx()
            .to(esdt_safe_addr)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .pause_endpoint()
            .sync_call();

        self.pause_bridge_proxy_event();
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
        self.unpause_bridge_proxy_event();
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
    #[endpoint(pauseProxy)]
    fn pause_proxy(&self) {
        let proxy_addr = self.proxy_address().get();

        self.tx()
            .to(proxy_addr)
            .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
            .pause_endpoint()
            .sync_call();

        self.pause_bridge_proxy_event();
    }

    #[only_owner]
    #[endpoint(unpauseProxy)]
    fn unpause_proxy(&self) {
        let proxy_addr = self.proxy_address().get();

        self.tx()
            .to(proxy_addr)
            .typed(bridge_proxy_contract_proxy::BridgeProxyContractProxy)
            .unpause_endpoint()
            .sync_call();

        self.unpause_bridge_proxy_event();
    }

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
}
