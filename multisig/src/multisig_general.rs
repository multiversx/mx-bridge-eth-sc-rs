elrond_wasm::imports!();

use eth_address::EthAddress;
use transaction::TransactionStatus;

use crate::action::Action;
use crate::user_role::UserRole;

#[elrond_wasm::module]
pub trait MultisigGeneralModule: crate::util::UtilModule + crate::storage::StorageModule {
    /// Used by board members to sign actions.
    #[endpoint(signEsdtSafeSetBatchStatus)]
    fn sign_esdt_safe_set_batch_status(
        &self,
        esdt_safe_batch_id: u64,
        #[var_args] expected_tx_batch_status: ManagedVarArgs<TransactionStatus>,
    ) -> SCResult<()> {
        let statuses_vec = expected_tx_batch_status.to_vec();
        let action_id = self
            .action_id_for_set_current_transaction_batch_status(esdt_safe_batch_id, &statuses_vec)
            .get();

        self.caller_sign(action_id)
    }

    #[endpoint(signEthToElrondBatch)]
    fn sign_eth_to_elrond_batch(
        &self,
        eth_batch_id: u64,
        #[var_args] transfers: ManagedVarArgs<
            MultiArg4<EthAddress<Self::Api>, ManagedAddress, TokenIdentifier, BigUint>,
        >,
    ) -> SCResult<()> {
        let transfers_vec = self.transfers_multiarg_to_tuples_vec(transfers);
        let action_id = self
            .eth_batch_id_to_action_id_mapping(eth_batch_id, &transfers_vec)
            .get();

        self.caller_sign(action_id)
    }

    fn caller_sign(&self, action_id: usize) -> SCResult<()> {
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

        Ok(())
    }

    fn propose_action(&self, action: Action<Self::Api>) -> SCResult<usize> {
        let caller_address = self.blockchain().get_caller();
        let caller_id = self.user_mapper().get_user_id(&caller_address);
        let caller_role = self.user_id_to_role(caller_id).get();
        require!(
            caller_role.is_board_member(),
            "only board members can propose"
        );

        require!(
            !self.pause_status().get(),
            "No actions may be proposed while paused"
        );

        let action_id = self.action_mapper().push(&action);
        if self.has_enough_stake(&caller_address) {
            let _ = self.action_signer_ids(action_id).insert(caller_id);
        }

        Ok(action_id)
    }

    fn clear_action(&self, action_id: usize) {
        self.action_mapper().clear_entry_unchecked(action_id);
        self.action_signer_ids(action_id).clear();
    }

    /// Can be used to:
    /// - create new user (board member / proposer)
    /// - remove user (board member / proposer)
    /// - reactivate removed user
    /// - convert between board member and proposer
    /// Will keep the board size and proposer count in sync.
    fn change_user_role(&self, user_address: ManagedAddress, new_role: UserRole) {
        let user_id = self.user_mapper().get_or_create_user(&user_address);
        let old_role = self.user_role(&user_address);

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

        self.user_id_to_role(user_id).set(&new_role);
    }
}
