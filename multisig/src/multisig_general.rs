multiversx_sc::imports!();

use crate::action::Action;
use crate::user_role::UserRole;

#[multiversx_sc::module]
pub trait MultisigGeneralModule:
    crate::util::UtilModule + crate::storage::StorageModule + multiversx_sc_modules::pause::PauseModule
{
    /// Used by board members to sign actions.
    #[endpoint]
    fn sign(&self, action_id: usize) {
        require!(
            !self.action_mapper().item_is_empty_unchecked(action_id),
            "action does not exist"
        );

        let caller_address = self.blockchain().get_caller();
        let caller_id = self.user_mapper().get_user_id(&caller_address);
        let caller_role = self.user_id_to_role(caller_id).get();
        require!(caller_role.is_board_member(), "only board members can sign");
        require!(self.has_enough_stake(&caller_address), "not enough stake");

        let _ = self.action_signer_ids(action_id).insert(caller_id);
    }

    fn propose_action(&self, action: Action<Self::Api>) -> usize {
        let caller_address = self.blockchain().get_caller();
        let caller_id = self.user_mapper().get_user_id(&caller_address);
        let caller_role = self.user_id_to_role(caller_id).get();
        require!(
            caller_role.is_board_member(),
            "only board members can propose"
        );

        require!(self.not_paused(), "No actions may be proposed while paused");

        let action_id = self.action_mapper().push(&action);
        if self.has_enough_stake(&caller_address) {
            let _ = self.action_signer_ids(action_id).insert(caller_id);
        }

        action_id
    }

    fn clear_action(&self, action_id: usize) {
        self.action_mapper().clear_entry_unchecked(action_id);
        self.action_signer_ids(action_id).clear();
    }

    fn add_board_member(&self, user_address: &ManagedAddress) {
        let user_id = self.user_mapper().get_or_create_user(user_address);
        let old_role = self.user_id_to_role(user_id).get();

        if !old_role.is_board_member() {
            self.num_board_members().update(|value| *value += 1);
            self.user_id_to_role(user_id).set(UserRole::BoardMember);
        }
    }

    fn remove_board_member(&self, user_address: &ManagedAddress) {
        let user_id = self.user_mapper().get_user_id(user_address);
        if user_id == 0 {
            return;
        }

        let old_role = self.user_id_to_role(user_id).get();
        if old_role.is_board_member() {
            self.num_board_members().update(|value| *value -= 1);
            self.user_id_to_role(user_id).set(UserRole::None);
        }
    }
}
