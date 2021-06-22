elrond_wasm::imports!();

use crate::action::Action;
use crate::action::ActionFullInfo;
use crate::user_role::UserRole;

#[elrond_wasm_derive::module]
pub trait UtilModule: crate::storage::StorageModule {
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
        let user_id = self.user_mapper().get_user_id(user);
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

    #[view(getAllStakedRelayers)]
    fn get_all_staked_relayers(&self) -> MultiResultVec<Address> {
        let mut relayers = self.get_all_board_members().into_vec();

        relayers.retain(|relayer| self.has_enough_stake(relayer));

        relayers.into()
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

    fn has_enough_stake(&self, board_member_address: &Address) -> bool {
        let required_stake = self.required_stake_amount().get();
        let amount_staked = self.amount_staked(board_member_address).get();

        amount_staked >= required_stake
    }
}
