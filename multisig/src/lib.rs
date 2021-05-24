#![no_std]
#![allow(non_snake_case)]

mod action;
mod user_role;

use action::{Action, ActionFullInfo};
use eth_address::*;
use transaction::*;
use user_role::UserRole;

elrond_wasm::imports!();

/// Multi-signature smart contract implementation.
/// Acts like a wallet that needs multiple signers for any action performed.
#[elrond_wasm_derive::contract]
pub trait Multisig {
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

    // endpoints - owner-only

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
        let gas_per_deploy = self.blockchain().get_gas_left() / 4;

        arg_buffer_token_whitelist.push_argument_bytes(wrapped_egld_token_id.as_esdt_identifier());
        for token in token_whitelist.into_vec() {
            arg_buffer_token_whitelist.push_argument_bytes(token.as_esdt_identifier());
        }

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

    /// Add ESDT Safe to Ethereum Fee Prepay whitelist
    /// Can't be done in the previous step, as the contracts only exist after execution has finished
    #[endpoint(finishSetup)]
    fn finish_setup(&self) -> SCResult<()> {
        self.require_caller_owner()?;
        self.require_ethereum_fee_prepay_deployed()?;
        self.require_esdt_safe_deployed()?;

        let ethereum_fee_prepay_address = self.ethereum_fee_prepay_address().get();
        let esdt_safe_address = self.esdt_safe_address().get();

        self.ethereum_fee_prepay_proxy(ethereum_fee_prepay_address)
            .add_to_whitelist(esdt_safe_address)
            .execute_on_dest_context(self.blockchain().get_gas_left());

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

    #[endpoint(proposeSlashUser)]
    fn propose_slash_user(&self, user_address: Address) -> SCResult<usize> {
        self.require_caller_owner()?;
        require!(
            self.user_role(&user_address) == UserRole::BoardMember,
            "can only slash board members"
        );

        self.propose_action(Action::SlashUser(user_address))
    }

    // endpoints

    #[payable("EGLD")]
    #[endpoint]
    fn stake(&self, #[payment] payment: Self::BigUint) -> SCResult<()> {
        let caller = self.blockchain().get_caller();
        let caller_role = self.user_role(&caller);
        require!(
            caller_role == UserRole::BoardMember,
            "Only board members can stake"
        );

        let mut amount_staked = self.amount_staked(&caller).get();
        amount_staked += payment;
        self.amount_staked(&caller).set(&amount_staked);

        Ok(())
    }

    #[endpoint]
    fn unstake(&self, amount: Self::BigUint) -> SCResult<()> {
        let caller = self.blockchain().get_caller();
        let amount_staked = self.amount_staked(&caller).get();
        require!(
            amount <= amount_staked,
            "can't unstake more than amount staked"
        );

        let remaining_stake = &amount_staked - &amount;
        if self.user_role(&caller) == UserRole::BoardMember {
            let required_stake_amount = self.required_stake_amount().get();
            require!(
                remaining_stake >= required_stake_amount,
                "can't unstake, must keep minimum amount as insurance"
            );
        }

        self.amount_staked(&caller).set(&remaining_stake);
        self.send().direct_egld(&caller, &amount, &[]);

        Ok(())
    }

    /// Used by board members to sign actions.
    #[endpoint]
    fn sign(&self, action_id: usize) -> SCResult<()> {
        require!(
            !self.action_mapper().item_is_empty_unchecked(action_id),
            "action does not exist"
        );

        let caller_address = self.blockchain().get_caller();
        let caller_id = self.user_mapper().get_user_id(&caller_address);
        let caller_role = self.get_user_id_to_role(caller_id);
        require!(caller_role.can_sign(), "only board members can sign");
        require!(self.has_enough_stake(&caller_address), "not enough stake");

        self.action_signer_ids(action_id).update(|signer_ids| {
            if !signer_ids.contains(&caller_id) {
                signer_ids.push(caller_id);
            }
        });

        Ok(())
    }

    /// Board members can withdraw their signatures if they no longer desire for the action to be executed.
    /// Actions that are left with no valid signatures can be then deleted to free up storage.
    #[endpoint]
    fn unsign(&self, action_id: usize) -> SCResult<()> {
        require!(
            !self.action_mapper().item_is_empty_unchecked(action_id),
            "action does not exist"
        );

        let caller_address = self.blockchain().get_caller();
        let caller_id = self.user_mapper().get_user_id(&caller_address);
        let caller_role = self.get_user_id_to_role(caller_id);
        require!(caller_role.can_sign(), "only board members can un-sign");
        require!(self.has_enough_stake(&caller_address), "not enough stake");

        self.action_signer_ids(action_id).update(|signer_ids| {
            if let Some(signer_pos) = signer_ids
                .iter()
                .position(|&signer_id| signer_id == caller_id)
            {
                // since we don't care about the order,
                // it is ok to call swap_remove, which is O(1)
                signer_ids.swap_remove(signer_pos);
            }
        });

        Ok(())
    }

    /// Initiates board member addition process.
    /// Can also be used to promote a proposer to board member.
    #[endpoint(proposeAddBoardMember)]
    fn propose_add_board_member(&self, board_member_address: Address) -> SCResult<usize> {
        self.propose_action(Action::AddBoardMember(board_member_address))
    }

    /// Initiates proposer addition process..
    /// Can also be used to demote a board member to proposer.
    #[endpoint(proposeAddProposer)]
    fn propose_add_proposer(&self, proposer_address: Address) -> SCResult<usize> {
        self.propose_action(Action::AddProposer(proposer_address))
    }

    /// Removes user regardless of whether it is a board member or proposer.
    #[endpoint(proposeRemoveUser)]
    fn propose_remove_user(&self, user_address: Address) -> SCResult<usize> {
        self.propose_action(Action::RemoveUser(user_address))
    }

    #[endpoint(proposeChangeQuorum)]
    fn propose_change_quorum(&self, new_quorum: usize) -> SCResult<usize> {
        self.propose_action(Action::ChangeQuorum(new_quorum))
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

    // ESDT Safe SC calls

    #[endpoint(esdtSafeAddTokenToWhitelist)]
    fn esdt_safe_add_token_to_whitelist(&self, token_id: TokenIdentifier) -> SCResult<()> {
        self.require_caller_owner()?;
        self.require_esdt_safe_deployed()?;

        self.esdt_safe_proxy(self.esdt_safe_address().get())
            .add_token_to_whitelist(token_id)
            .execute_on_dest_context(self.blockchain().get_gas_left());

        Ok(())
    }

    #[endpoint(esdtSafeRemoveTokenFromWhitelist)]
    fn esdt_safe_remove_token_from_whitelist(&self, token_id: TokenIdentifier) -> SCResult<()> {
        self.require_caller_owner()?;
        self.require_esdt_safe_deployed()?;

        self.esdt_safe_proxy(self.esdt_safe_address().get())
            .remove_token_from_whitelist(token_id)
            .execute_on_dest_context(self.blockchain().get_gas_left());

        Ok(())
    }

    #[endpoint(getNextPendingTransaction)]
    fn get_next_pending_transaction(&self) -> SCResult<TxAsMultiResult<Self::BigUint>> {
        self.require_esdt_safe_deployed()?;
        require!(
            self.current_tx().is_empty(),
            "Must execute and set status for current tx first"
        );

        match self
            .esdt_safe_proxy(self.esdt_safe_address().get())
            .get_next_pending_transaction()
            .execute_on_dest_context(self.blockchain().get_gas_left())
        {
            OptionalArg::Some(multi_result) => {
                self.current_tx()
                    .set(&Transaction::from(multi_result.clone()));

                Ok(multi_result)
            }
            OptionalArg::None => sc_error!("No transactions left to execute"),
        }
    }

    #[endpoint(proposeEsdtSafeSetCurrentTransactionStatus)]
    fn propose_esdt_safe_set_current_transaction_status(
        &self,
        relayer_reward_address: Address,
        transaction_status: TransactionStatus,
    ) -> SCResult<usize> {
        self.require_esdt_safe_deployed()?;
        require!(
            !self.current_tx().is_empty(),
            "There is no transaction to set status for"
        );
        require!(
            self.action_id_for_set_current_transaction_status().get() == 0,
            "Set status action already proposed"
        );

        let action_id = self.propose_action(Action::SetCurrentTransactionStatus {
            relayer_reward_address,
            tx_status: transaction_status,
        })?;

        self.action_id_for_set_current_transaction_status()
            .set(&action_id);

        Ok(action_id)
    }

    // Multi-transfer ESDT SC calls

    #[endpoint(multiTransferEsdtaddTokenToWhitelist)]
    fn multi_transfer_esdt_add_token_to_whitelist(
        &self,
        token_id: TokenIdentifier,
    ) -> SCResult<()> {
        self.require_caller_owner()?;
        self.require_multi_transfer_esdt_deployed()?;

        self.multi_transfer_esdt_proxy(self.multi_transfer_esdt_address().get())
            .add_token_to_whitelist(token_id)
            .execute_on_dest_context(self.blockchain().get_gas_left());

        Ok(())
    }

    #[endpoint(proposeMultiTransferEsdtTransferEsdtToken)]
    fn propose_multi_transfer_esdt_transfer_esdt_token(
        &self,
        eth_tx_nonce: u64,
        to: Address,
        token_id: TokenIdentifier,
        amount: Self::BigUint,
    ) -> SCResult<usize> {
        self.require_multi_transfer_esdt_deployed()?;
        require!(
            self.eth_tx_nonce_to_action_id_mapping(eth_tx_nonce)
                .is_empty(),
            "Tx was already proposed"
        );

        let action_id = self.propose_action(Action::TransferEsdtToken {
            to,
            token_id,
            amount,
        })?;

        self.eth_tx_nonce_to_action_id_mapping(eth_tx_nonce)
            .set(&action_id);

        Ok(action_id)
    }

    /// Proposers and board members use this to launch signed actions.
    #[endpoint(performAction)]
    fn perform_action_endpoint(&self, action_id: usize) -> SCResult<()> {
        let caller_address = self.blockchain().get_caller();
        let caller_id = self.user_mapper().get_user_id(&caller_address);
        let caller_role = self.get_user_id_to_role(caller_id);
        require!(
            caller_role.can_perform_action(),
            "only board members and proposers can perform actions"
        );
        require!(
            self.quorum_reached(action_id),
            "quorum has not been reached"
        );

        self.perform_action(action_id)
    }

    /// Clears storage pertaining to an action that is no longer supposed to be executed.
    /// Any signatures that the action received must first be removed, via `unsign`.
    /// Otherwise this endpoint would be prone to abuse.
    #[endpoint(discardAction)]
    fn discard_action(&self, action_id: usize) -> SCResult<()> {
        let caller_address = self.blockchain().get_caller();
        let caller_id = self.user_mapper().get_user_id(&caller_address);
        let caller_role = self.get_user_id_to_role(caller_id);
        require!(
            caller_role.can_discard_action(),
            "only board members and proposers can discard actions"
        );
        require!(
            self.get_action_valid_signer_count(action_id) == 0,
            "cannot discard action with valid signatures"
        );

        self.clear_action(action_id);
        Ok(())
    }

    // views

    /// Iterates through all actions and retrieves those that are still pending.
    /// Serialized full action data:
    /// - the action id
    /// - the serialized action data
    /// - (number of signers followed by) list of signer addresses.
    #[view(getPendingActionFullInfo)]
    fn get_pending_action_full_info(&self) -> MultiResultVec<ActionFullInfo<Self::BigUint>> {
        let mut result = Vec::new();
        let action_last_index = self.get_action_last_index();
        let action_mapper = self.action_mapper();
        for action_id in 1..=action_last_index {
            let action_data = action_mapper.get(action_id);
            if action_data.is_pending() {
                result.push(ActionFullInfo {
                    action_id,
                    action_data,
                    signers: self.get_action_signers(action_id),
                });
            }
        }
        result.into()
    }

    /// Returns `true` (`1`) if the user has signed the action.
    /// Does not check whether or not the user is still a board member and the signature valid.
    #[view]
    fn signed(&self, user: Address, action_id: usize) -> bool {
        let user_id = self.user_mapper().get_user_id(&user);
        if user_id == 0 {
            false
        } else {
            let signer_ids = self.action_signer_ids(action_id).get();
            signer_ids.contains(&user_id)
        }
    }

    /// Indicates user rights.
    /// `0` = no rights,
    /// `1` = can propose, but not sign,
    /// `2` = can propose and sign.
    #[view(userRole)]
    fn user_role(&self, user: &Address) -> UserRole {
        let user_id = self.user_mapper().get_user_id(&user);
        if user_id == 0 {
            UserRole::None
        } else {
            self.get_user_id_to_role(user_id)
        }
    }

    /// Lists all users that can sign actions.
    #[view(getAllBoardMembers)]
    fn get_all_board_members(&self) -> MultiResultVec<Address> {
        self.get_all_users_with_role(UserRole::BoardMember)
    }

    /// Lists all proposers that are not board members.
    #[view(getAllProposers)]
    fn get_all_proposers(&self) -> MultiResultVec<Address> {
        self.get_all_users_with_role(UserRole::Proposer)
    }

    /// Gets addresses of all users who signed an action.
    /// Does not check if those users are still board members or not,
    /// so the result may contain invalid signers.
    #[view(getActionSigners)]
    fn get_action_signers(&self, action_id: usize) -> Vec<Address> {
        self.action_signer_ids(action_id)
            .get()
            .iter()
            .map(|signer_id| self.user_mapper().get_user_address_unchecked(*signer_id))
            .collect()
    }

    /// Gets addresses of all users who signed an action and are still board members.
    /// All these signatures are currently valid.
    #[view(getActionSignerCount)]
    fn get_action_signer_count(&self, action_id: usize) -> usize {
        self.action_signer_ids(action_id).get().len()
    }

    /// It is possible for board members to lose their role.
    /// They are not automatically removed from all actions when doing so,
    /// therefore the contract needs to re-check every time when actions are performed.
    /// This function is used to validate the signers before performing an action.
    /// It also makes it easy to check before performing an action.
    #[view(getActionValidSignerCount)]
    fn get_action_valid_signer_count(&self, action_id: usize) -> usize {
        let signer_ids = self.action_signer_ids(action_id).get();
        signer_ids
            .iter()
            .filter(|signer_id| {
                let signer_role = self.get_user_id_to_role(**signer_id);
                signer_role.can_sign()
            })
            .count()
    }

    /// Returns `true` (`1`) if `getActionValidSignerCount >= getQuorum`.
    #[view(quorumReached)]
    fn quorum_reached(&self, action_id: usize) -> bool {
        let quorum = self.quorum().get();
        let valid_signers_count = self.get_action_valid_signer_count(action_id);
        valid_signers_count >= quorum
    }

    /// The index of the last proposed action.
    /// 0 means that no action was ever proposed yet.
    #[view(getActionLastIndex)]
    fn get_action_last_index(&self) -> usize {
        self.action_mapper().len()
    }

    /// Serialized action data of an action with index.
    #[view(getActionData)]
    fn get_action_data(&self, action_id: usize) -> Action<Self::BigUint> {
        self.action_mapper().get(action_id)
    }

    #[view(getCurrentTx)]
    fn get_current_tx(&self) -> OptionalResult<TxAsMultiResult<Self::BigUint>> {
        if !self.current_tx().is_empty() {
            OptionalResult::Some(self.current_tx().get().into_multiresult())
        } else {
            OptionalResult::None
        }
    }

    #[view(isValidActionId)]
    fn is_valid_action_id(&self, action_id: usize) -> bool {
        let min_id = 1;
        let max_id = self.action_mapper().len();

        action_id >= min_id && action_id <= max_id
    }

    /// Actions are cleared after execution, so an empty entry means the action was executed already
    /// Returns "false" if the action ID is invalid
    #[view(wasActionExecuted)]
    fn was_action_executed(&self, action_id: usize) -> bool {
        if self.is_valid_action_id(action_id) {
            self.action_mapper().item_is_empty(action_id)
        } else {
            false
        }
    }

    /// If the mapping was made, it means that the transfer action was proposed in the past
    /// To check if it was executed as well, use the wasActionExecuted view
    #[view(wasTransferActionProposed)]
    fn was_transfer_action_proposed(&self, eth_tx_nonce: u64) -> bool {
        let action_id = self.eth_tx_nonce_to_action_id_mapping(eth_tx_nonce).get();

        self.is_valid_action_id(action_id)
    }

    #[view(wasSetCurrentTransactionStatusActionProposed)]
    fn was_set_current_transaction_status_action_proposed(
        &self,
        expected_tx_status: TransactionStatus,
    ) -> bool {
        let action_id = self.action_id_for_set_current_transaction_status().get();

        if self.is_valid_action_id(action_id) {
            let action = self.action_mapper().get(action_id);

            match action {
                Action::SetCurrentTransactionStatus {
                    relayer_reward_address: _,
                    tx_status,
                } => tx_status == expected_tx_status,
                _ => false,
            }
        } else {
            false
        }
    }

    // private

    fn propose_action(&self, action: Action<Self::BigUint>) -> SCResult<usize> {
        let caller_address = self.blockchain().get_caller();
        let caller_id = self.user_mapper().get_user_id(&caller_address);
        let caller_role = self.get_user_id_to_role(caller_id);
        require!(
            caller_role.can_propose(),
            "only board members and proposers can propose"
        );

        let action_id = self.action_mapper().push(&action);
        if caller_role.can_sign() {
            // also sign
            // since the action is newly created, the caller can be the only signer
            self.action_signer_ids(action_id).set(&[caller_id].to_vec());
        }

        Ok(action_id)
    }

    fn get_all_users_with_role(&self, role: UserRole) -> MultiResultVec<Address> {
        let mut result = Vec::new();
        let num_users = self.user_mapper().get_user_count();
        for user_id in 1..=num_users {
            if self.get_user_id_to_role(user_id) == role {
                if let Some(address) = self.user_mapper().get_user_address(user_id) {
                    result.push(address);
                }
            }
        }
        result.into()
    }

    /// Can be used to:
    /// - create new user (board member / proposer)
    /// - remove user (board member / proposer)
    /// - reactivate removed user
    /// - convert between board member and proposer
    /// Will keep the board size and proposer count in sync.
    fn change_user_role(&self, user_address: Address, new_role: UserRole) {
        let user_id = self.user_mapper().get_or_create_user(&user_address);
        let old_role = if user_id == 0 {
            UserRole::None
        } else {
            self.get_user_id_to_role(user_id)
        };
        self.set_user_id_to_role(user_id, new_role);

        // update board size
        #[allow(clippy::collapsible_else_if)]
        if old_role == UserRole::BoardMember {
            if new_role != UserRole::BoardMember {
                self.num_board_members().update(|value| *value -= 1);
            }
        } else {
            if new_role == UserRole::BoardMember {
                self.num_board_members().update(|value| *value += 1);
            }
        }

        // update num_proposers
        #[allow(clippy::collapsible_else_if)]
        if old_role == UserRole::Proposer {
            if new_role != UserRole::Proposer {
                self.num_proposers().update(|value| *value -= 1);
            }
        } else {
            if new_role == UserRole::Proposer {
                self.num_proposers().update(|value| *value += 1);
            }
        }
    }

    fn perform_action(&self, action_id: usize) -> SCResult<()> {
        let action = self.action_mapper().get(action_id);

        if self.pause_status().get() {
            require!(
                action.is_slash_user(),
                "Only slash user action may be performed while paused"
            );
        }

        // clean up storage
        // happens before actual execution, because the match provides the return on each branch
        // syntax aside, the async_call_raw kills contract execution so cleanup cannot happen afterwards
        self.clear_action(action_id);

        match action {
            Action::Nothing => {}
            Action::AddBoardMember(board_member_address) => {
                self.change_user_role(board_member_address, UserRole::BoardMember);
            }
            Action::AddProposer(proposer_address) => {
                self.change_user_role(proposer_address, UserRole::Proposer);

                // validation required for the scenario when a board member becomes a proposer
                require!(
                    self.quorum().get() <= self.num_board_members().get(),
                    "quorum cannot exceed board size"
                );
            }
            Action::RemoveUser(user_address) => {
                self.change_user_role(user_address, UserRole::None);
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
            }
            Action::SlashUser(user_address) => {
                self.change_user_role(user_address.clone(), UserRole::None);
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

                // remove slashed amount from user stake amount
                let mut user_stake = self.amount_staked(&user_address).get();
                user_stake -= &slash_amount;
                self.amount_staked(&user_address).set(&user_stake);

                // add it to total slashed amount pool
                let mut total_slashed_amount = self.slashed_tokens_amount().get();
                total_slashed_amount += &slash_amount;
                self.slashed_tokens_amount().set(&total_slashed_amount);
            }
            Action::ChangeQuorum(new_quorum) => {
                require!(
                    new_quorum <= self.num_board_members().get(),
                    "quorum cannot exceed board size"
                );
                self.quorum().set(&new_quorum);
            }
            Action::SetCurrentTransactionStatus {
                relayer_reward_address,
                tx_status,
            } => {
                let current_tx = self.current_tx().get();

                self.current_tx().clear();
                self.action_id_for_set_current_transaction_status().clear();

                self.esdt_safe_proxy(self.esdt_safe_address().get())
                    .set_transaction_status(
                        relayer_reward_address,
                        current_tx.from,
                        current_tx.nonce,
                        tx_status,
                    )
                    .execute_on_dest_context(self.blockchain().get_gas_left());
            }
            Action::TransferEsdtToken {
                to,
                token_id,
                amount,
            } => {
                self.multi_transfer_esdt_proxy(self.multi_transfer_esdt_address().get())
                    .transfer_esdt_token(to, token_id, amount)
                    .execute_on_dest_context(self.blockchain().get_gas_left());
            }
        }

        Ok(())
    }

    fn clear_action(&self, action_id: usize) {
        self.action_mapper().clear_entry_unchecked(action_id);
        self.action_signer_ids(action_id).clear();
    }

    fn has_enough_stake(&self, board_member_address: &Address) -> bool {
        let required_stake = self.required_stake_amount().get();
        let amount_staked = self.amount_staked(board_member_address).get();

        amount_staked >= required_stake
    }

    fn require_caller_owner(&self) -> SCResult<()> {
        only_owner!(self, "Only owner may call this function");
        Ok(())
    }

    fn require_esdt_safe_deployed(&self) -> SCResult<()> {
        require!(
            !self.esdt_safe_address().is_empty(),
            "ESDT Safe SC has to be deployed first"
        );
        Ok(())
    }

    fn require_multi_transfer_esdt_deployed(&self) -> SCResult<()> {
        require!(
            !self.multi_transfer_esdt_address().is_empty(),
            "Multi-transfer ESDT SC has to be deployed first"
        );
        Ok(())
    }

    fn require_ethereum_fee_prepay_deployed(&self) -> SCResult<()> {
        require!(
            !self.ethereum_fee_prepay_address().is_empty(),
            "Ethereum Fee Prepay SC has to be deployed first"
        );
        Ok(())
    }

    // proxies

    #[proxy]
    fn egld_esdt_swap_proxy(&self, sc_address: Address) -> egld_esdt_swap::Proxy<Self::SendApi>;

    #[proxy]
    fn esdt_safe_proxy(&self, sc_address: Address) -> esdt_safe::Proxy<Self::SendApi>;

    #[proxy]
    fn multi_transfer_esdt_proxy(
        &self,
        sc_address: Address,
    ) -> multi_transfer_esdt::Proxy<Self::SendApi>;

    #[proxy]
    fn ethereum_fee_prepay_proxy(
        &self,
        sc_address: Address,
    ) -> ethereum_fee_prepay::Proxy<Self::SendApi>;

    // storage

    /// Minimum number of signatures needed to perform any action.
    #[view(getQuorum)]
    #[storage_mapper("quorum")]
    fn quorum(&self) -> SingleValueMapper<Self::Storage, usize>;

    #[storage_mapper("user")]
    fn user_mapper(&self) -> UserMapper<Self::Storage>;

    #[storage_get("user_role")]
    fn get_user_id_to_role(&self, user_id: usize) -> UserRole;

    #[storage_set("user_role")]
    fn set_user_id_to_role(&self, user_id: usize, user_role: UserRole);

    /// Denormalized board member count.
    /// It is kept in sync with the user list by the contract.
    #[view(getNumBoardMembers)]
    #[storage_mapper("num_board_members")]
    fn num_board_members(&self) -> SingleValueMapper<Self::Storage, usize>;

    /// Denormalized proposer count.
    /// It is kept in sync with the user list by the contract.
    #[view(getNumProposers)]
    #[storage_mapper("num_proposers")]
    fn num_proposers(&self) -> SingleValueMapper<Self::Storage, usize>;

    #[storage_mapper("action_data")]
    fn action_mapper(&self) -> VecMapper<Self::Storage, Action<Self::BigUint>>;

    #[storage_mapper("action_signer_ids")]
    fn action_signer_ids(&self, action_id: usize) -> SingleValueMapper<Self::Storage, Vec<usize>>;

    /// The required amount to stake for accepting relayer position
    #[view(getRequiredStakeAmount)]
    #[storage_mapper("requiredStakeAmount")]
    fn required_stake_amount(&self) -> SingleValueMapper<Self::Storage, Self::BigUint>;

    /// Staked amount by each board member.
    #[view(getAmountStaked)]
    #[storage_mapper("amountStaked")]
    fn amount_staked(
        &self,
        board_member_address: &Address,
    ) -> SingleValueMapper<Self::Storage, Self::BigUint>;

    /// Amount of stake slashed if a relayer is misbehaving
    #[view(getSlashAmount)]
    #[storage_mapper("slashAmount")]
    fn slash_amount(&self) -> SingleValueMapper<Self::Storage, Self::BigUint>;

    /// Total slashed tokens accumulated
    #[view(getSlashedTokensAmount)]
    #[storage_mapper("slashedTokensAmount")]
    fn slashed_tokens_amount(&self) -> SingleValueMapper<Self::Storage, Self::BigUint>;

    #[view(isPaused)]
    #[storage_mapper("pauseStatus")]
    fn pause_status(&self) -> SingleValueMapper<Self::Storage, bool>;

    #[storage_mapper("currentTx")]
    fn current_tx(&self) -> SingleValueMapper<Self::Storage, Transaction<Self::BigUint>>;

    #[view(getActionIdForEthTxNonce)]
    #[storage_mapper("ethTxNonceToActionIdMapping")]
    fn eth_tx_nonce_to_action_id_mapping(
        &self,
        eth_tx_nonce: u64,
    ) -> SingleValueMapper<Self::Storage, usize>;

    #[view(getActionIdForSetCurrentTransactionStatus)]
    #[storage_mapper("actionIdForSetCurrentTransactionStatus")]
    fn action_id_for_set_current_transaction_status(
        &self,
    ) -> SingleValueMapper<Self::Storage, usize>;

    /// Mapping between ERC20 Ethereum address and Elrond ESDT Token Identifiers

    #[view(getErc20AddressForTokenId)]
    #[storage_mapper("erc20AddressForTokenId")]
    fn erc20_address_for_token_id(
        &self,
        token_id: &TokenIdentifier,
    ) -> SingleValueMapper<Self::Storage, EthAddress>;

    #[view(getTokenIdForErc20Address)]
    #[storage_mapper("tokenIdForErc20Address")]
    fn token_id_for_erc20_address(
        &self,
        erc20_address: &EthAddress,
    ) -> SingleValueMapper<Self::Storage, TokenIdentifier>;

    // SC addresses

    #[view(getEgldEsdtSwapAddress)]
    #[storage_mapper("egldEsdtSwapAddress")]
    fn egld_esdt_swap_address(&self) -> SingleValueMapper<Self::Storage, Address>;

    #[view(getEsdtSafeAddress)]
    #[storage_mapper("esdtSafeAddress")]
    fn esdt_safe_address(&self) -> SingleValueMapper<Self::Storage, Address>;

    #[view(getMultiTransferEsdtAddress)]
    #[storage_mapper("multiTransferEsdtAddress")]
    fn multi_transfer_esdt_address(&self) -> SingleValueMapper<Self::Storage, Address>;

    #[view(getEthereumFeePrepayAddress)]
    #[storage_mapper("ethereumFeePrepayAddress")]
    fn ethereum_fee_prepay_address(&self) -> SingleValueMapper<Self::Storage, Address>;
}
