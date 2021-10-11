#![no_std]
#![allow(non_snake_case)]
#![allow(clippy::too_many_arguments)]

mod action;
mod user_role;

use token_module::{ProxyTrait as OtherProxyTrait, PERCENTAGE_TOTAL};

use action::Action;
use transaction::esdt_safe_batch::EsdtSafeTxBatchSplitInFields;
use transaction::*;
use user_role::UserRole;

mod multisig_general;
mod setup;
mod storage;
mod util;

elrond_wasm::imports!();

/// Multi-signature smart contract implementation.
/// Acts like a wallet that needs multiple signers for any action performed.
#[elrond_wasm_derive::contract]
pub trait Multisig:
    multisig_general::MultisigGeneralModule
    + setup::SetupModule
    + storage::StorageModule
    + util::UtilModule
{
    #[only_owner]
    #[endpoint(distributeFeesFromChildContracts)]
    fn distribute_fees_from_child_contracts(
        &self,
        #[var_args] dest_address_percentage_pairs: VarArgs<MultiArg2<Address, u64>>,
    ) -> SCResult<()> {
        let mut args = Vec::new();
        let mut total_percentage = 0;

        for pair in dest_address_percentage_pairs.into_vec() {
            let (dest_address, percentage) = pair.into_tuple();

            require!(
                !self.blockchain().is_smart_contract(&dest_address),
                "Cannot transfer to smart contract dest_address"
            );

            total_percentage += percentage;
            args.push((dest_address, percentage));
        }

        require!(
            total_percentage == PERCENTAGE_TOTAL,
            "Percentages do not add up to 100%"
        );

        self.esdt_safe_proxy(self.esdt_safe_address().get())
            .distribute_fees(args.clone())
            .execute_on_dest_context();

        self.multi_transfer_esdt_proxy(self.multi_transfer_esdt_address().get())
            .distribute_fees(args)
            .execute_on_dest_context();

        Ok(())
    }

    #[payable("EGLD")]
    #[endpoint]
    fn stake(&self, #[payment] payment: Self::BigUint) -> SCResult<()> {
        let caller = self.blockchain().get_caller();
        let caller_role = self.user_role(&caller);
        require!(
            caller_role == UserRole::BoardMember,
            "Only board members can stake"
        );

        self.amount_staked(&caller)
            .update(|amount_staked| *amount_staked += payment);

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

    // ESDT Safe SC calls

    #[endpoint(proposeEsdtSafeSetCurrentTransactionBatchStatus)]
    fn propose_esdt_safe_set_current_transaction_batch_status(
        &self,
        esdt_safe_batch_id: u64,
        #[var_args] tx_batch_status: VarArgs<TransactionStatus>,
    ) -> SCResult<usize> {
        let call_result = self
            .esdt_safe_proxy(self.esdt_safe_address().get())
            .get_current_tx_batch()
            .execute_on_dest_context();
        let (current_batch_id, current_batch_transactions) = call_result
            .into_option()
            .ok_or("Current batch is empty")?
            .into_tuple();

        require!(
            self.action_id_for_set_current_transaction_batch_status(esdt_safe_batch_id)
                .get(&tx_batch_status.0)
                == None,
            "Action already proposed"
        );

        let current_batch_len = current_batch_transactions.len();
        let status_batch_len = tx_batch_status.len();
        require!(
            current_batch_len == status_batch_len,
            "Number of statuses provided must be equal to number of transactions in current batch"
        );
        require!(
            esdt_safe_batch_id == current_batch_id,
            "Current EsdtSafe tx batch does not have the provided ID"
        );

        let action_id = self.propose_action(Action::SetCurrentTransactionBatchStatus {
            esdt_safe_batch_id,
            tx_batch_status: tx_batch_status.0.clone(),
        })?;

        self.action_id_for_set_current_transaction_batch_status(esdt_safe_batch_id)
            .insert(tx_batch_status.into_vec(), action_id);

        Ok(action_id)
    }

    // Multi-transfer ESDT SC calls

    #[endpoint(proposeMultiTransferEsdtBatch)]
    fn propose_multi_transfer_esdt_batch(
        &self,
        batch_id: u64,
        #[var_args] transfers: MultiArgVec<MultiArg3<Address, TokenIdentifier, Self::BigUint>>,
    ) -> SCResult<usize> {
        self.require_multi_transfer_esdt_deployed()?;

        let transfers_as_tuples = self.transfers_multiarg_to_tuples_vec(transfers);

        require!(
            self.batch_id_to_action_id_mapping(batch_id)
                .get(&transfers_as_tuples)
                == None,
            "This batch was already proposed"
        );

        let action_id = self.propose_action(Action::BatchTransferEsdtToken {
            batch_id,
            transfers: transfers_as_tuples.clone(),
        })?;

        self.batch_id_to_action_id_mapping(batch_id)
            .insert(transfers_as_tuples, action_id);

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
        require!(
            !self.pause_status().get(),
            "No actions may be executed while paused"
        );

        self.perform_action(action_id);

        Ok(())
    }

    #[view(getCurrentTxBatch)]
    fn get_current_tx_batch(&self) -> OptionalResult<EsdtSafeTxBatchSplitInFields<Self::BigUint>> {
        let _ = self
            .esdt_safe_proxy(self.esdt_safe_address().get())
            .get_current_tx_batch()
            .execute_on_dest_context();

        // result is already returned automatically from the EsdtSafe call,
        // we only keep this signature for correct ABI generation
        OptionalResult::None
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
    fn was_transfer_action_proposed(
        &self,
        batch_id: u64,
        #[var_args] transfers: MultiArgVec<MultiArg3<Address, TokenIdentifier, Self::BigUint>>,
    ) -> bool {
        let action_id = self.get_action_id_for_transfer_batch(batch_id, transfers);

        self.is_valid_action_id(action_id)
    }

    #[view(getActionIdForTransferBatch)]
    fn get_action_id_for_transfer_batch(
        &self,
        batch_id: u64,
        #[var_args] transfers: MultiArgVec<MultiArg3<Address, TokenIdentifier, Self::BigUint>>,
    ) -> usize {
        let transfers_as_tuples = self.transfers_multiarg_to_tuples_vec(transfers);

        self.batch_id_to_action_id_mapping(batch_id)
            .get(&transfers_as_tuples)
            .unwrap_or(0)
    }

    #[view(getStatusesAfterExecution)]
    fn get_statuses_after_execution(&self, batch_id: u64) -> MultiResultVec<TransactionStatus> {
        let (actual_batch_id, statuses) = self.statuses_after_execution().get();
        if batch_id == actual_batch_id {
            statuses.into()
        } else {
            MultiResultVec::new()
        }
    }

    #[view(wasSetCurrentTransactionBatchStatusActionProposed)]
    fn was_set_current_transaction_batch_status_action_proposed(
        &self,
        esdt_safe_batch_id: u64,
        #[var_args] expected_tx_batch_status: VarArgs<TransactionStatus>,
    ) -> bool {
        self.is_valid_action_id(self.get_action_id_for_set_current_transaction_batch_status(
            esdt_safe_batch_id,
            expected_tx_batch_status,
        ))
    }

    #[view(getActionIdForSetCurrentTransactionBatchStatus)]
    fn get_action_id_for_set_current_transaction_batch_status(
        &self,
        esdt_safe_batch_id: u64,
        #[var_args] expected_tx_batch_status: VarArgs<TransactionStatus>,
    ) -> usize {
        self.action_id_for_set_current_transaction_batch_status(esdt_safe_batch_id)
            .get(&expected_tx_batch_status.0)
            .unwrap_or(0)
    }

    // private

    fn perform_action(&self, action_id: usize) {
        let action = self.action_mapper().get(action_id);
        self.clear_action(action_id);

        match action {
            Action::Nothing => {}
            Action::SetCurrentTransactionBatchStatus {
                esdt_safe_batch_id,
                tx_batch_status,
            } => {
                let mut action_ids_mapper =
                    self.action_id_for_set_current_transaction_batch_status(esdt_safe_batch_id);

                // if there's only one proposed action,
                // the action was already cleared at the beginning of this function
                if action_ids_mapper.len() > 1 {
                    for act_id in action_ids_mapper.values() {
                        self.clear_action(act_id);
                    }
                }

                action_ids_mapper.clear();

                self.esdt_safe_proxy(self.esdt_safe_address().get())
                    .set_transaction_batch_status(
                        esdt_safe_batch_id,
                        VarArgs::from(tx_batch_status),
                    )
                    .execute_on_dest_context();
            }
            Action::BatchTransferEsdtToken {
                batch_id,
                transfers,
            } => {
                let mut action_ids_mapper = self.batch_id_to_action_id_mapping(batch_id);

                // if there's only one proposed action,
                // the action was already cleared at the beginning of this function
                if action_ids_mapper.len() > 1 {
                    for act_id in action_ids_mapper.values() {
                        self.clear_action(act_id);
                    }
                }

                action_ids_mapper.clear();

                let transfers_len = transfers.len();
                let statuses = self
                    .multi_transfer_esdt_proxy(self.multi_transfer_esdt_address().get())
                    .batch_transfer_esdt_token(transfers.into())
                    .execute_on_dest_context_custom_range(|_, after| {
                        (after - transfers_len, after)
                    });

                self.statuses_after_execution()
                    .set(&(batch_id, statuses.into_vec()));
            }
            _ => {}
        }
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
}
