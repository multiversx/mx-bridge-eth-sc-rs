multiversx_sc::imports!();

use crate::{action::Action, user_role::UserRole};
use transaction::{transaction_status::TransactionStatus, EthTxAsMultiValue, TxBatchSplitInFields};

use tx_batch_module::ProxyTrait as _;

/// Note: Additional queries can be found in the Storage module
#[multiversx_sc::module]
pub trait QueriesModule: crate::storage::StorageModule + crate::util::UtilModule {
    /// Returns the current EsdtSafe batch.
    ///
    /// First result is the batch ID, then pairs of 6 results, representing transactions
    /// split by fields:
    ///
    /// Block Nonce, Tx Nonce, Sender Address, Receiver Address, Token ID, Amount
    #[view(getCurrentTxBatch)]
    fn get_current_tx_batch(&self) -> OptionalValue<TxBatchSplitInFields<Self::Api>> {
        self.get_esdt_safe_proxy_instance()
            .get_current_tx_batch()
            .execute_on_dest_context()
    }

    /// Returns a batch of failed Ethereum -> Elrond transactions.
    /// The result format is the same as getCurrentTxBatch
    #[view(getCurrentRefundBatch)]
    fn get_current_refund_batch(&self) -> OptionalValue<TxBatchSplitInFields<Self::Api>> {
        self.get_multi_transfer_esdt_proxy_instance()
            .get_first_batch_any_status()
            .execute_on_dest_context()
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

    /// Used for Ethereum -> Elrond batches.
    /// If the mapping was made, it means that the transfer action was proposed in the past.
    /// To check if it was executed as well, use the wasActionExecuted view
    #[view(wasTransferActionProposed)]
    fn was_transfer_action_proposed(
        &self,
        eth_batch_id: u64,
        transfers: MultiValueEncoded<EthTxAsMultiValue<Self::Api>>,
    ) -> bool {
        let action_id = self.get_action_id_for_transfer_batch(eth_batch_id, transfers);

        self.is_valid_action_id(action_id)
    }

    /// Used for Ethereum -> Elrond batches.
    /// If `wasActionExecuted` returns true, then this can be used to get the action ID.
    /// Will return 0 if the transfers were not proposed
    #[view(getActionIdForTransferBatch)]
    fn get_action_id_for_transfer_batch(
        &self,
        eth_batch_id: u64,
        transfers: MultiValueEncoded<EthTxAsMultiValue<Self::Api>>,
    ) -> usize {
        let transfers_as_struct = self.transfers_multi_value_to_eth_tx_vec(transfers);
        let batch_hash = self.hash_eth_tx_batch(&transfers_as_struct);

        self.batch_id_to_action_id_mapping(eth_batch_id)
            .get(&batch_hash)
            .unwrap_or(0)
    }

    /// Used for Elrond -> Ethereum batches.
    /// Returns "true" if an action was already proposed for the given batch,
    /// with these exact transaction statuses, in this exact order
    #[view(wasSetCurrentTransactionBatchStatusActionProposed)]
    fn was_set_current_transaction_batch_status_action_proposed(
        &self,
        esdt_safe_batch_id: u64,
        expected_tx_batch_status: MultiValueEncoded<TransactionStatus>,
    ) -> bool {
        self.is_valid_action_id(self.get_action_id_for_set_current_transaction_batch_status(
            esdt_safe_batch_id,
            expected_tx_batch_status,
        ))
    }

    /// If `wasSetCurrentTransactionBatchStatusActionProposed` return true,
    /// this can be used to get the action ID.
    /// Will return 0 if the set status action was not proposed
    #[view(getActionIdForSetCurrentTransactionBatchStatus)]
    fn get_action_id_for_set_current_transaction_batch_status(
        &self,
        esdt_safe_batch_id: u64,
        expected_tx_batch_status: MultiValueEncoded<TransactionStatus>,
    ) -> usize {
        self.action_id_for_set_current_transaction_batch_status(esdt_safe_batch_id)
            .get(&expected_tx_batch_status.to_vec())
            .unwrap_or(0)
    }

    /// Returns `true` (`1`) if the user has signed the action.
    /// Does not check whether or not the user is still a board member and the signature valid.
    #[view]
    fn signed(&self, user: ManagedAddress, action_id: usize) -> bool {
        let user_id = self.user_mapper().get_user_id(&user);
        if user_id == 0 {
            false
        } else {
            self.action_signer_ids(action_id).contains(&user_id)
        }
    }

    /// Indicates user rights.
    /// `0` = no rights,
    /// `1` = can propose. Can also sign if they have enough stake.
    #[view(userRole)]
    fn user_role(&self, user: &ManagedAddress) -> UserRole {
        self.get_user_role(user)
    }

    /// Lists all board members
    #[view(getAllBoardMembers)]
    fn get_all_board_members(&self) -> MultiValueEncoded<ManagedAddress> {
        self.get_all_users_with_role(UserRole::BoardMember)
    }

    /// Lists all board members that staked the correct amount.
    /// A board member with not enough stake can propose, but cannot sign.
    #[view(getAllStakedRelayers)]
    fn get_all_staked_relayers(&self) -> MultiValueEncoded<ManagedAddress> {
        let relayers = self.get_all_board_members().to_vec();
        let mut staked_relayers = ManagedVec::new();

        for relayer in &relayers {
            if self.has_enough_stake(&relayer) {
                staked_relayers.push(relayer);
            }
        }

        staked_relayers.into()
    }

    /// Gets the number of signatures for the action with the given ID
    #[view(getActionSignerCount)]
    fn get_action_signer_count(&self, action_id: usize) -> usize {
        self.action_signer_ids(action_id).len()
    }

    /// It is possible for board members to lose their role.
    /// They are not automatically removed from all actions when doing so,
    /// therefore the contract needs to re-check every time when actions are performed.
    /// This function is used to validate the signers before performing an action.
    /// It also makes it easy to check before performing an action.
    #[view(getActionValidSignerCount)]
    fn get_action_valid_signer_count(&self, action_id: usize) -> usize {
        self.action_signer_ids(action_id)
            .iter()
            .filter(|signer_id| {
                let signer_role = self.user_id_to_role(*signer_id).get();
                let signer_address = self
                    .user_mapper()
                    .get_user_address(*signer_id)
                    .unwrap_or_default();

                signer_role.is_board_member() && self.has_enough_stake(&signer_address)
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
    fn get_action_data(&self, action_id: usize) -> Action<Self::Api> {
        self.action_mapper().get(action_id)
    }
}
