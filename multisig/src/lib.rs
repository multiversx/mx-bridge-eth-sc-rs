#![no_std]
#![allow(non_snake_case)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

mod action;
mod user_role;

use action::Action;
use eth_address::EthAddress;
use token_module::AddressPercentagePair;
use transaction::esdt_safe_batch::TxBatchSplitInFields;
use transaction::*;
use user_role::UserRole;

mod multisig_general;
mod setup;
mod storage;
mod util;

use token_module::ProxyTrait as _;
use tx_batch_module::ProxyTrait as _;

pub const PERCENTAGE_TOTAL: u32 = 10_000; // precision of 2 decimals

elrond_wasm::imports!();

/// Multi-signature smart contract implementation.
/// Acts like a wallet that needs multiple signers for any action performed.
#[elrond_wasm::contract]
pub trait Multisig:
    multisig_general::MultisigGeneralModule
    + setup::SetupModule
    + storage::StorageModule
    + util::UtilModule
{
    #[init]
    fn init(
        &self,
        esdt_safe_sc_address: ManagedAddress,
        multi_transfer_sc_address: ManagedAddress,
        required_stake: BigUint,
        slash_amount: BigUint,
        quorum: usize,
        #[var_args] board: ManagedVarArgs<ManagedAddress>,
    ) -> SCResult<()> {
        self.quorum().set(&quorum);

        let mut duplicates = false;
        let board_len = board.len();
        self.user_mapper()
            .get_or_create_users(board.into_iter(), |user_id, new_user| {
                if !new_user {
                    duplicates = true;
                }
                self.user_id_to_role(user_id).set(&UserRole::BoardMember);
            });
        require!(!duplicates, "duplicate board member");

        self.num_board_members()
            .update(|nr_board_members| *nr_board_members += board_len);

        require!(
            slash_amount <= required_stake,
            "slash amount must be less than or equal to required stake"
        );
        self.required_stake_amount().set(&required_stake);
        self.slash_amount().set(&slash_amount);

        require!(
            self.blockchain().is_smart_contract(&esdt_safe_sc_address),
            "Esdt Safe address is not a Smart Contract address"
        );
        self.esdt_safe_address().set(&esdt_safe_sc_address);

        require!(
            self.blockchain()
                .is_smart_contract(&multi_transfer_sc_address),
            "Multi Transfer address is not a Smart Contract address"
        );
        self.multi_transfer_esdt_address()
            .set(&multi_transfer_sc_address);

        Ok(())
    }

    #[only_owner]
    #[endpoint(distributeFeesFromChildContracts)]
    fn distribute_fees_from_child_contracts(
        &self,
        #[var_args] dest_address_percentage_pairs: ManagedVarArgs<MultiArg2<ManagedAddress, u32>>,
    ) -> SCResult<()> {
        let mut args = ManagedVec::new();
        let mut total_percentage = 0;

        for pair in dest_address_percentage_pairs {
            let (dest_address, percentage) = pair.into_tuple();

            require!(
                !self.blockchain().is_smart_contract(&dest_address),
                "Cannot transfer to smart contract dest_address"
            );

            total_percentage += percentage;
            args.push(AddressPercentagePair {
                address: dest_address,
                percentage,
            });
        }

        require!(
            total_percentage == PERCENTAGE_TOTAL,
            "Percentages do not add up to 100%"
        );

        self.get_esdt_safe_proxy_instance()
            .distribute_fees(args.clone())
            .execute_on_dest_context();

        self.get_multi_transfer_esdt_proxy_instance()
            .distribute_fees(args)
            .execute_on_dest_context();

        Ok(())
    }

    #[payable("EGLD")]
    #[endpoint]
    fn stake(&self, #[payment] payment: BigUint) -> SCResult<()> {
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
    fn unstake(&self, amount: BigUint) -> SCResult<()> {
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
        #[var_args] tx_batch_status: ManagedVarArgs<TransactionStatus>,
    ) -> SCResult<()> {
        let call_result = self
            .get_esdt_safe_proxy_instance()
            .get_current_tx_batch()
            .execute_on_dest_context();
        let (current_batch_id, current_batch_transactions) = call_result
            .into_option()
            .ok_or("Current batch is empty")?
            .into_tuple();

        let statuses_vec = tx_batch_status.to_vec();

        require!(
            self.action_id_for_set_current_transaction_batch_status(
                esdt_safe_batch_id,
                &statuses_vec
            )
            .is_empty(),
            "Action already proposed"
        );

        let current_batch_len = current_batch_transactions.len() / TX_MULTIRESULT_NR_FIELDS;
        let status_batch_len = statuses_vec.len();
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
            tx_batch_status: statuses_vec.clone(),
        })?;

        self.action_id_for_set_current_transaction_batch_status(esdt_safe_batch_id, &statuses_vec)
            .set(&action_id);

        Ok(())
    }

    // Multi-transfer ESDT SC calls

    #[endpoint(proposeMultiTransferEsdtBatch)]
    fn propose_multi_transfer_esdt_batch(
        &self,
        eth_batch_id: u64,
        #[var_args] transfers: ManagedVarArgs<
            MultiArg4<EthAddress<Self::Api>, ManagedAddress, TokenIdentifier, BigUint>,
        >,
    ) -> SCResult<()> {
        let current_eth_batch_id = self.current_eth_batch_id().get();
        require!(
            eth_batch_id == current_eth_batch_id,
            "Can only propose for the current batch"
        );

        let transfers_as_tuples = self.transfers_multiarg_to_tuples_vec(transfers);
        require!(
            self.eth_batch_id_to_action_id_mapping(eth_batch_id, &transfers_as_tuples)
                .is_empty(),
            "This batch was already proposed"
        );

        let action_id = self.propose_action(Action::BatchTransferEsdtToken {
            eth_batch_id,
            transfers: transfers_as_tuples.clone(),
        })?;

        self.eth_batch_id_to_action_id_mapping(eth_batch_id, &transfers_as_tuples)
            .set(&action_id);

        Ok(())
    }

    #[only_owner]
    #[endpoint(moveRefundBatchToSafe)]
    fn move_refund_batch_to_safe(&self) {
        let opt_refund_batch_fields: OptionalResult<TxBatchSplitInFields<Self::Api>> = self
            .get_multi_transfer_esdt_proxy_instance()
            .get_and_clear_first_refund_batch()
            .execute_on_dest_context();

        if let OptionalResult::Some(refund_batch_fields) = opt_refund_batch_fields {
            let (_batch_id, all_tx_fields) = refund_batch_fields.into_tuple();
            let mut refund_batch = ManagedVec::new();

            for tx_fields in all_tx_fields {
                refund_batch.push(Transaction::from(tx_fields));
            }

            self.get_esdt_safe_proxy_instance()
                .add_refund_batch(refund_batch)
                .execute_on_dest_context();
        }
    }

    /// Proposers and board members use this to launch signed actions.
    #[endpoint(performAction)]
    fn perform_action_endpoint(&self, action_id: usize) -> SCResult<()> {
        require!(
            !self.action_mapper().item_is_empty(action_id),
            "Action was already executed"
        );

        let caller_address = self.blockchain().get_caller();
        let caller_id = self.user_mapper().get_user_id(&caller_address);
        let caller_role = self.user_id_to_role(caller_id).get();
        require!(
            caller_role.is_board_member(),
            "only board members can perform actions"
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
    fn get_current_tx_batch(&self) -> OptionalResult<TxBatchSplitInFields<Self::Api>> {
        let _ = self
            .get_esdt_safe_proxy_instance()
            .get_current_tx_batch()
            .execute_on_dest_context();

        // result is already returned automatically from the EsdtSafe call,
        // we only keep this signature for correct ABI generation
        OptionalResult::None
    }

    // For failed Ethereum -> Elrond transactions
    #[view(getCurrentRefundBatch)]
    fn get_current_refund_batch(&self) -> OptionalResult<TxBatchSplitInFields<Self::Api>> {
        let _ = self
            .get_multi_transfer_esdt_proxy_instance()
            .get_current_tx_batch()
            .execute_on_dest_context();

        // result is already returned automatically from the MultiTransferEsdt call,
        // we only keep this signature for correct ABI generation
        OptionalResult::None
    }

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
        eth_batch_id: u64,
        #[var_args] transfers: ManagedVarArgs<
            MultiArg4<EthAddress<Self::Api>, ManagedAddress, TokenIdentifier, BigUint>,
        >,
    ) -> bool {
        let transfers_vec = self.transfers_multiarg_to_tuples_vec(transfers);
        let action_id = self
            .eth_batch_id_to_action_id_mapping(eth_batch_id, &transfers_vec)
            .get();

        self.is_valid_action_id(action_id)
    }

    #[view(wasSetCurrentTransactionBatchStatusActionProposed)]
    fn was_set_current_transaction_batch_status_action_proposed(
        &self,
        esdt_safe_batch_id: u64,
        #[var_args] expected_tx_batch_status: ManagedVarArgs<TransactionStatus>,
    ) -> bool {
        let statuses_vec = expected_tx_batch_status.to_vec();
        let action_id = self
            .action_id_for_set_current_transaction_batch_status(esdt_safe_batch_id, &statuses_vec)
            .get();

        self.is_valid_action_id(action_id)
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
                self.action_id_for_set_current_transaction_batch_status(
                    esdt_safe_batch_id,
                    &tx_batch_status,
                )
                .clear();

                self.get_esdt_safe_proxy_instance()
                    .set_transaction_batch_status(esdt_safe_batch_id, tx_batch_status.into())
                    .execute_on_dest_context();
            }
            Action::BatchTransferEsdtToken {
                eth_batch_id,
                transfers,
            } => {
                self.eth_batch_id_to_action_id_mapping(eth_batch_id, &transfers)
                    .clear();
                self.current_eth_batch_id().update(|id| *id += 1);

                self.get_multi_transfer_esdt_proxy_instance()
                    .batch_transfer_esdt_token(transfers.into())
                    .execute_on_dest_context();
            }
            _ => {}
        }
    }

    // proxies

    #[proxy]
    fn esdt_safe_proxy(&self, sc_address: ManagedAddress) -> esdt_safe::Proxy<Self::Api>;

    #[proxy]
    fn multi_transfer_esdt_proxy(
        &self,
        sc_address: ManagedAddress,
    ) -> multi_transfer_esdt::Proxy<Self::Api>;

    fn get_esdt_safe_proxy_instance(&self) -> esdt_safe::Proxy<Self::Api> {
        self.esdt_safe_proxy(self.esdt_safe_address().get())
    }

    fn get_multi_transfer_esdt_proxy_instance(&self) -> multi_transfer_esdt::Proxy<Self::Api> {
        self.multi_transfer_esdt_proxy(self.multi_transfer_esdt_address().get())
    }
}
