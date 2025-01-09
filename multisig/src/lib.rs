#![no_std]
#![allow(clippy::too_many_arguments)]

mod action;
mod events;
mod multisig_general;
mod queries;
mod setup;
mod storage;
mod user_role;
mod util;

use sc_proxies::{esdt_safe_proxy, multi_transfer_esdt_proxy};

use action::Action;
use token_module::{AddressPercentagePair, INVALID_PERCENTAGE_SUM_OVER_ERR_MSG, PERCENTAGE_TOTAL};
use transaction::transaction_status::TransactionStatus;
use transaction::TxBatchSplitInFields;
use transaction::*;
use user_role::UserRole;

use multiversx_sc::imports::*;

const MAX_ACTIONS_INTER: usize = 10;

/// Multi-signature smart contract implementation.
/// Acts like a wallet that needs multiple signers for any action performed.
#[multiversx_sc::contract]
pub trait Multisig:
    multisig_general::MultisigGeneralModule
    + events::EventsModule
    + setup::SetupModule
    + storage::StorageModule
    + util::UtilModule
    + queries::QueriesModule
    + multiversx_sc_modules::pause::PauseModule
{
    /// EsdtSafe and MultiTransferEsdt are expected to be deployed and configured separately,
    /// and then having their ownership changed to this Multisig SC.
    #[init]
    fn init(
        &self,
        esdt_safe_sc_address: ManagedAddress,
        multi_transfer_sc_address: ManagedAddress,
        proxy_sc_address: ManagedAddress,
        bridged_tokens_wrapper_sc_address: ManagedAddress,
        price_aggregator_sc_address: ManagedAddress,
        required_stake: BigUint,
        slash_amount: BigUint,
        quorum: usize,
        board: MultiValueEncoded<ManagedAddress>,
    ) {
        let mut duplicates = false;
        let board_len = board.len();
        self.user_mapper()
            .get_or_create_users(board.into_iter(), |user_id, new_user| {
                if !new_user {
                    duplicates = true;
                }
                self.user_id_to_role(user_id).set(UserRole::BoardMember);
            });
        require!(!duplicates, "duplicate board member");

        self.num_board_members()
            .update(|nr_board_members| *nr_board_members += board_len);
        self.change_quorum(quorum);

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

        require!(
            self.blockchain().is_smart_contract(&proxy_sc_address),
            "Proxy address is not a Smart Contract address"
        );
        self.proxy_address().set(&proxy_sc_address);

        require!(
            self.blockchain()
                .is_smart_contract(&bridged_tokens_wrapper_sc_address),
            "Bridged Tokens Wrapper address is not a Smart Contract address"
        );
        self.bridged_tokens_wrapper_address()
            .set(&bridged_tokens_wrapper_sc_address);

        require!(
            self.blockchain()
                .is_smart_contract(&price_aggregator_sc_address),
            "Price Aggregator address is not a Smart Contract address"
        );
        self.fee_estimator_address()
            .set(&price_aggregator_sc_address);

        self.set_paused(true);
    }

    #[upgrade]
    fn upgrade(
        &self,
        esdt_safe_sc_address: ManagedAddress,
        multi_transfer_sc_address: ManagedAddress,
        proxy_sc_address: ManagedAddress,
        bridged_tokens_wrapper_sc_address: ManagedAddress,
        price_aggregator_sc_address: ManagedAddress,
    ) {
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

        require!(
            self.blockchain().is_smart_contract(&proxy_sc_address),
            "Proxy address is not a Smart Contract address"
        );
        self.proxy_address().set(&proxy_sc_address);

        require!(
            self.blockchain()
                .is_smart_contract(&bridged_tokens_wrapper_sc_address),
            "Bridged Tokens Wrapper address is not a Smart Contract address"
        );
        self.bridged_tokens_wrapper_address()
            .set(&bridged_tokens_wrapper_sc_address);

        require!(
            self.blockchain()
                .is_smart_contract(&price_aggregator_sc_address),
            "Price Aggregator address is not a Smart Contract address"
        );
        self.fee_estimator_address()
            .set(&price_aggregator_sc_address);

        self.set_paused(true);
    }

    /// Distributes the accumulated fees to the given addresses.
    /// Expected arguments are pairs of (address, percentage),
    /// where percentages must add up to the PERCENTAGE_TOTAL constant
    #[only_owner]
    #[endpoint(distributeFeesFromChildContracts)]
    fn distribute_fees_from_child_contracts(
        &self,
        dest_address_percentage_pairs: MultiValueEncoded<MultiValue2<ManagedAddress, u32>>,
    ) {
        let mut args = ManagedVec::new();
        let mut total_percentage = 0u64;

        for pair in dest_address_percentage_pairs {
            let (dest_address, percentage) = pair.into_tuple();

            require!(
                !self.blockchain().is_smart_contract(&dest_address),
                "Cannot transfer to smart contract dest_address"
            );

            total_percentage += percentage as u64;
            args.push(AddressPercentagePair {
                address: dest_address,
                percentage,
            });
        }

        require!(
            total_percentage == PERCENTAGE_TOTAL as u64,
            INVALID_PERCENTAGE_SUM_OVER_ERR_MSG
        );
        let esdt_safe_addr = self.esdt_safe_address().get();
        let opt_tokens_to_distribute: OptionalValue<MultiValueEncoded<TokenIdentifier<Self::Api>>> =
            OptionalValue::None;
        self.tx()
            .to(esdt_safe_addr)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .distribute_fees(args, opt_tokens_to_distribute)
            .sync_call();
    }

    /// Board members have to stake a certain amount of EGLD
    /// before being allowed to sign actions
    #[payable("EGLD")]
    #[endpoint]
    fn stake(&self, #[payment] payment: BigUint) {
        let caller = self.blockchain().get_caller();
        let caller_role = self.user_role(&caller);
        require!(
            caller_role == UserRole::BoardMember,
            "Only board members can stake"
        );

        self.amount_staked(&caller)
            .update(|amount_staked| *amount_staked += payment);
    }

    #[endpoint]
    fn unstake(&self, amount: BigUint) {
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
        self.tx().to(ToCaller).egld(&amount).transfer();
    }

    // ESDT Safe SC calls

    /// After a batch is processed on the Ethereum side,
    /// the EsdtSafe expects a list of statuses of said transactions (success or failure).
    ///
    /// This endpoint proposes an action to set the statuses to a certain list of values.
    /// Nothing is changed in the EsdtSafe contract until the action is signed and executed.
    #[endpoint(proposeEsdtSafeSetCurrentTransactionBatchStatus)]
    fn propose_esdt_safe_set_current_transaction_batch_status(
        &self,
        esdt_safe_batch_id: u64,
        tx_batch_status: MultiValueEncoded<TransactionStatus>,
    ) -> usize {
        let esdt_safe_addr = self.esdt_safe_address().get();
        let call_result: OptionalValue<TxBatchSplitInFields<Self::Api>> = self
            .tx()
            .to(esdt_safe_addr)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .get_current_tx_batch()
            .returns(ReturnsResult)
            .sync_call();

        let (current_batch_id, current_batch_transactions) = match call_result {
            OptionalValue::Some(batch) => batch.into_tuple(),
            OptionalValue::None => sc_panic!("Current batch is empty"),
        };
        let statuses_vec = tx_batch_status.to_vec();

        require!(
            self.action_id_for_set_current_transaction_batch_status(esdt_safe_batch_id)
                .get(&statuses_vec)
                .is_none(),
            "Action already proposed"
        );

        let current_batch_len = current_batch_transactions.len();
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
        });

        self.action_id_for_set_current_transaction_batch_status(esdt_safe_batch_id)
            .insert(statuses_vec, action_id);

        action_id
    }

    // Multi-transfer ESDT SC calls

    /// Proposes a batch of Ethereum -> MultiversX transfers.
    /// Transactions have to be separated by fields, in the following order:
    /// Sender Address, Destination Address, Token ID, Amount, Tx Nonce
    #[endpoint(proposeMultiTransferEsdtBatch)]
    fn propose_multi_transfer_esdt_batch(
        &self,
        eth_batch_id: u64,
        transfers: MultiValueEncoded<EthTxAsMultiValue<Self::Api>>,
    ) -> usize {
        let next_eth_batch_id = self.last_executed_eth_batch_id().get() + 1;
        require!(
            eth_batch_id == next_eth_batch_id,
            "Can only propose for next batch ID"
        );

        let transfers_as_eth_tx = self.transfers_multi_value_to_eth_tx_vec(transfers);
        self.require_valid_eth_tx_ids(&transfers_as_eth_tx);

        let batch_hash = self.hash_eth_tx_batch(&transfers_as_eth_tx);
        require!(
            self.batch_id_to_action_id_mapping(eth_batch_id)
                .get(&batch_hash)
                .is_none(),
            "This batch was already proposed"
        );

        let action_id = self.propose_action(Action::BatchTransferEsdtToken {
            eth_batch_id,
            transfers: transfers_as_eth_tx,
        });

        self.batch_id_to_action_id_mapping(eth_batch_id)
            .insert(batch_hash, action_id);

        action_id
    }

    /// Failed Ethereum -> MultiversX transactions are saved in the MultiTransfer SC
    /// as "refund transactions", and stored in batches, using the same mechanism as EsdtSafe.
    ///
    /// This function moves the first refund batch into the EsdtSafe SC,
    /// converting the transactions into MultiversX -> Ethereum transactions
    /// and adding them into EsdtSafe batches
    #[only_owner]
    #[endpoint(moveRefundBatchToSafeFromChildContract)]
    fn move_refund_batch_to_safe_from_child_contract(&self) {
        let multi_transfer_esdt_addr = self.multi_transfer_esdt_address().get();
        self.tx()
            .to(multi_transfer_esdt_addr)
            .typed(multi_transfer_esdt_proxy::MultiTransferEsdtProxy)
            .move_refund_batch_to_safe()
            .sync_call();

        self.move_refund_batch_to_safe_event();
    }

    #[only_owner]
    #[payable("*")]
    #[endpoint(initSupplyFromChildContract)]
    fn init_supply_from_child_contract(&self, token_id: TokenIdentifier, amount: BigUint) {
        let (payment_token, payment_amount) = self.call_value().single_fungible_esdt();
        let esdt_safe_addr = self.esdt_safe_address().get();

        self.tx()
            .to(esdt_safe_addr)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .init_supply(token_id, amount)
            .payment((payment_token.clone(), 0, payment_amount.clone()))
            .sync_call();
    }

    #[only_owner]
    #[endpoint(addUnprocessedRefundTxToBatch)]
    fn add_unprocessed_refund_tx_to_batch(&self, tx_id: u64) {
        let multi_transfer_esdt_addr = self.multi_transfer_esdt_address().get();
        self.tx()
            .to(multi_transfer_esdt_addr)
            .typed(multi_transfer_esdt_proxy::MultiTransferEsdtProxy)
            .add_unprocessed_refund_tx_to_batch(tx_id)
            .sync_call();

        self.add_unprocessed_refund_tx_to_batch_event(tx_id);
    }

    #[only_owner]
    #[endpoint(withdrawRefundFeesForEthereum)]
    fn withdraw_refund_fees_for_ethereum(&self, token_id: TokenIdentifier) {
        let esdt_safe_addr = self.esdt_safe_address().get();
        let multisig_owner = self.blockchain().get_owner_address();

        self.tx()
            .to(esdt_safe_addr)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .withdraw_refund_fees_for_ethereum(token_id, multisig_owner)
            .sync_call();
    }

    #[only_owner]
    #[endpoint(withdrawTransactionFees)]
    fn withdraw_transaction_fees(&self, token_id: TokenIdentifier) {
        let esdt_safe_addr = self.esdt_safe_address().get();
        let multisig_owner = self.blockchain().get_owner_address();

        self.tx()
            .to(esdt_safe_addr)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .withdraw_transaction_fees(token_id, multisig_owner)
            .sync_call();
    }

    #[only_owner]
    #[endpoint(withdrawSlashedAmount)]
    fn withdraw_slashed_amount(&self) {
        let slashed_tokens_amount_mapper = self.slashed_tokens_amount();
        let slashed_amount = slashed_tokens_amount_mapper.get();
        self.tx().to(ToCaller).egld(&slashed_amount).transfer();
        slashed_tokens_amount_mapper.clear();
    }

    /// Proposers and board members use this to launch signed actions.
    #[endpoint(performAction)]
    fn perform_action_endpoint(&self, action_id: usize) {
        require!(
            !self.executed_actions().contains(&action_id),
            "Action was already executed"
        );

        let caller_address = self.blockchain().get_caller();
        let caller_role = self.get_user_role(&caller_address);
        require!(
            caller_role.is_board_member(),
            "only board members can perform actions"
        );
        require!(
            self.quorum_reached(action_id),
            "quorum has not been reached"
        );
        require!(self.not_paused(), "No actions may be executed while paused");

        self.perform_action(action_id);
        self.executed_actions().insert(action_id);
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
                    for act_id in action_ids_mapper.values().take(MAX_ACTIONS_INTER) {
                        self.clear_action(act_id);
                    }
                }

                action_ids_mapper.clear();
                let esdt_safe_addr = self.esdt_safe_address().get();
                self.tx()
                    .to(esdt_safe_addr)
                    .typed(esdt_safe_proxy::EsdtSafeProxy)
                    .set_transaction_batch_status(
                        esdt_safe_batch_id,
                        MultiValueEncoded::from(tx_batch_status),
                    )
                    .sync_call();
            }
            Action::BatchTransferEsdtToken {
                eth_batch_id,
                transfers,
            } => {
                let mut action_ids_mapper = self.batch_id_to_action_id_mapping(eth_batch_id);

                // if there's only one proposed action,
                // the action was already cleared at the beginning of this function
                if action_ids_mapper.len() > 1 {
                    for act_id in action_ids_mapper.values().take(MAX_ACTIONS_INTER) {
                        self.clear_action(act_id);
                    }
                }

                action_ids_mapper.clear();
                self.last_executed_eth_batch_id().update(|id| *id += 1);

                let last_tx_index = transfers.len() - 1;
                let last_tx = transfers.get(last_tx_index);
                self.last_executed_eth_tx_id().set(last_tx.tx_nonce);

                let multi_transfer_esdt_addr = self.multi_transfer_esdt_address().get();
                let transfers_multi: MultiValueEncoded<Self::Api, EthTransaction<Self::Api>> =
                    transfers.clone().into();

                self.tx()
                    .to(multi_transfer_esdt_addr)
                    .typed(multi_transfer_esdt_proxy::MultiTransferEsdtProxy)
                    .batch_transfer_esdt_token(eth_batch_id, transfers_multi)
                    .sync_call();
            }
        }
    }

    #[endpoint(clearActionsForBatchId)]
    fn clear_actions_for_batch_id(&self, eth_batch_id: u64) {
        let last_executed_eth_batch_id = self.last_executed_eth_batch_id().get();
        require!(
            eth_batch_id < last_executed_eth_batch_id,
            "Batch needs to be already executed"
        );

        let action_ids_mapper = self.batch_id_to_action_id_mapping(eth_batch_id);

        for act_id in action_ids_mapper.values().take(MAX_ACTIONS_INTER) {
            self.clear_action(act_id);
        }
    }
}
