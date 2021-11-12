elrond_wasm::imports!();

use transaction::SingleTransferTuple;

use crate::action::Action;
use crate::user_role::UserRole;

#[elrond_wasm::module]
pub trait UtilModule: crate::storage::StorageModule {
    /// Returns `true` (`1`) if the user has signed the action.
    /// Does not check whether or not the user is still a board member and the signature valid.
    #[view]
    fn signed(&self, user: ManagedAddress, action_id: usize) -> bool {
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
    fn user_role(&self, user: &ManagedAddress) -> UserRole {
        let user_id = self.user_mapper().get_user_id(user);
        if user_id == 0 {
            UserRole::None
        } else {
            self.get_user_id_to_role(user_id)
        }
    }

    /// Lists all users that can sign actions.
    #[view(getAllBoardMembers)]
    fn get_all_board_members(&self) -> MultiResultVec<ManagedAddress> {
        self.get_all_users_with_role(UserRole::BoardMember)
    }

    #[view(getAllStakedRelayers)]
    fn get_all_staked_relayers(&self) -> MultiResultVec<ManagedAddress> {
        let mut relayers = self.get_all_board_members().into_vec();
        relayers.retain(|relayer| self.has_enough_stake(relayer));

        relayers.into()
    }

    /// Lists all proposers that are not board members.
    #[view(getAllProposers)]
    fn get_all_proposers(&self) -> MultiResultVec<ManagedAddress> {
        self.get_all_users_with_role(UserRole::Proposer)
    }

    /// Gets addresses of all users who signed an action.
    /// Does not check if those users are still board members or not,
    /// so the result may contain invalid signers.
    #[view(getActionSigners)]
    fn get_action_signers(&self, action_id: usize) -> Vec<ManagedAddress> {
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
    fn get_action_data(&self, action_id: usize) -> Action<BigUint> {
        self.action_mapper().get(action_id)
    }

    fn get_all_users_with_role(&self, role: UserRole) -> MultiResultVec<ManagedAddress> {
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

    fn has_enough_stake(&self, board_member_address: &ManagedAddress) -> bool {
        let required_stake = self.required_stake_amount().get();
        let amount_staked = self.amount_staked(board_member_address).get();

        amount_staked >= required_stake
    }

    fn transfers_multiarg_to_tuples_vec(
        &self,
        transfers: MultiArgVec<MultiArg3<ManagedAddress, TokenIdentifier, BigUint>>,
    ) -> Vec<SingleTransferTuple<BigUint>> {
        let mut transfers_as_tuples = Vec::new();
        for transfer in transfers.into_vec() {
            transfers_as_tuples.push(transfer.into_tuple());
        }

        transfers_as_tuples
    }
}
