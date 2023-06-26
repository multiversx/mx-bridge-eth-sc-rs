multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use eth_address::EthAddress;
use transaction::transaction_status::TransactionStatus;

use crate::action::Action;
use crate::user_role::UserRole;

pub type EthBatchHash<M> = ManagedByteArray<M, 32>; // keccak256(ManagedVec<EthTransaction<Self::Api>)

#[multiversx_sc::module]
pub trait StorageModule {
    /// Minimum number of signatures needed to perform any action.
    #[view(getQuorum)]
    #[storage_mapper("quorum")]
    fn quorum(&self) -> SingleValueMapper<usize>;

    #[storage_mapper("user")]
    fn user_mapper(&self) -> UserMapper;

    #[storage_mapper("user_role")]
    fn user_id_to_role(&self, user_id: usize) -> SingleValueMapper<UserRole>;

    /// Denormalized board member count.
    /// It is kept in sync with the user list by the contract.
    #[view(getNumBoardMembers)]
    #[storage_mapper("num_board_members")]
    fn num_board_members(&self) -> SingleValueMapper<usize>;

    #[storage_mapper("action_data")]
    fn action_mapper(&self) -> VecMapper<Action<Self::Api>>;

    #[storage_mapper("action_signer_ids")]
    fn action_signer_ids(&self, action_id: usize) -> UnorderedSetMapper<usize>;

    /// The required amount to stake for accepting relayer position
    #[view(getRequiredStakeAmount)]
    #[storage_mapper("requiredStakeAmount")]
    fn required_stake_amount(&self) -> SingleValueMapper<BigUint>;

    /// Staked amount by each board member.
    #[view(getAmountStaked)]
    #[storage_mapper("amountStaked")]
    fn amount_staked(&self, board_member_address: &ManagedAddress) -> SingleValueMapper<BigUint>;

    /// Amount of stake slashed if a relayer is misbehaving
    #[view(getSlashAmount)]
    #[storage_mapper("slashAmount")]
    fn slash_amount(&self) -> SingleValueMapper<BigUint>;

    /// Total slashed tokens accumulated
    #[view(getSlashedTokensAmount)]
    #[storage_mapper("slashedTokensAmount")]
    fn slashed_tokens_amount(&self) -> SingleValueMapper<BigUint>;

    #[view(getLastExecutedEthBatchId)]
    #[storage_mapper("lastExecutedEthBatchId")]
    fn last_executed_eth_batch_id(&self) -> SingleValueMapper<u64>;

    #[view(getLastExecutedEthTxId)]
    #[storage_mapper("lastExecutedEthTxId")]
    fn last_executed_eth_tx_id(&self) -> SingleValueMapper<u64>;

    #[storage_mapper("batchIdToActionIdMapping")]
    fn batch_id_to_action_id_mapping(
        &self,
        batch_id: u64,
    ) -> MapMapper<EthBatchHash<Self::Api>, usize>;

    #[storage_mapper("actionIdForSetCurrentTransactionBatchStatus")]
    fn action_id_for_set_current_transaction_batch_status(
        &self,
        esdt_safe_batch_id: u64,
    ) -> MapMapper<ManagedVec<TransactionStatus>, usize>;

    /// Mapping between ERC20 Ethereum address and Elrond ESDT Token Identifiers

    #[view(getErc20AddressForTokenId)]
    #[storage_mapper("erc20AddressForTokenId")]
    fn erc20_address_for_token_id(
        &self,
        token_id: &TokenIdentifier,
    ) -> SingleValueMapper<EthAddress<Self::Api>>;

    #[view(getTokenIdForErc20Address)]
    #[storage_mapper("tokenIdForErc20Address")]
    fn token_id_for_erc20_address(
        &self,
        erc20_address: &EthAddress<Self::Api>,
    ) -> SingleValueMapper<TokenIdentifier>;

    // SC addresses

    #[view(getEsdtSafeAddress)]
    #[storage_mapper("esdtSafeAddress")]
    fn esdt_safe_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getMultiTransferEsdtAddress)]
    #[storage_mapper("multiTransferEsdtAddress")]
    fn multi_transfer_esdt_address(&self) -> SingleValueMapper<ManagedAddress>;
}
