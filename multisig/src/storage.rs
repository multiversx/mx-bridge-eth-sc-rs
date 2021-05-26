elrond_wasm::imports!();

use eth_address::EthAddress;
use transaction::Transaction;

use crate::action::Action;
use crate::user_role::UserRole;

#[elrond_wasm_derive::module]
pub trait StorageModule {
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
