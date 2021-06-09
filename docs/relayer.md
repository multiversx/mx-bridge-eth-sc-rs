# Relayer Documentation

## Introduction

In this document, you will find all the possible actions, scenarios and what is expected of you as a relayer. We will be using the terms `relayer` and `board-member` interchangeably, but they mean the same thing in our case.  

We're assuming all the smart contracts are already setup as described [here](setup.md).    

For details about multisig smart contract design, check here: https://github.com/ElrondNetwork/elrond-wasm-rs/blob/master/contracts/examples/multisig/README.md.  

If you're interested in a more abstract explanation, check the [readme](../README.md).  

As a relayer, you will interact directly only with the multisig-staking smart contract.  

## Prerequisites for being a relayer

The first and most important prerequisite is being recognized as a board member by the multisig smart contract. There are two ways this can happen:
- Owner adds you as board member when deploying the contract
- The current board members vote to add you as a board member alongside them

But that is only the first step. You will not be able to perform any board-member exclusive action until you've staked a certain amount of eGLD in the multisig contract. Once staked, you cannot unstake until your role has been revoked.  This can also happen in two ways:
- You (or another board member) propose to remove you as board memmber, in which case you will then be able to unstake your full stake
- The owner proposes to "slash" your stake, you lose your board member role and part of your stake and can unstake the rest.  

Stake "slashing" will only happen if you're actively being malicious. So play nice!  

## Elrond -> Ethereum transaction

For this kind of transaction, we'll be using the `EsdtSafe` and `EthereumFeePrepay` contracts. The user will first have to pay the transaction fees through the `EthereumFeePrepay` SC, and then to submit the transaction through the `EsdtSafe` SC. To protect against transaction flood, the fee is locked at the time of saving the transaction.  

After the user played their part, any relayer may call `getNextTransactionBatch`. There is no risk of overwriting, as the function will return an error if the current batch was not executed yet. `getNextTransactionBatch` will query the `EsdtSafe` contract and, as the name suggests, get the next N pending transactions. 

The batch will not exceed a pre-defined size, and it will also not bundle transactions that are too far from each other in terms of their creation date. These are configurable constants in the `EsdtSafe` smart contract.

The transaction data (block nonce, transaction nonce, sender, receiver, token_id and amount) will be stored inside the multisig SC for further processing. Any one of the relayers will be able to get the transactions and execute them on the Ethereum side.  

Note: Transaction nonce is not the account nonce of the sender account, and rather an internal nonce used by the `EsdtSafe` smart contract. It can safely be ignored by relayers.  

Once the transaction has been executed, a `SetCurrentTransactionBatchStatus` action will be proposed (through the `proposeEsdtSafeSetCurrentTransactionBatchStatus` endpoint), which, for each transaction in the batch, will set the status to "Executed" if it was executed successfully, or "Rejected" if it failed for any reason. Success will burn the tokens on the Elrond side, while a failure will return the tokens to the user. Endpoint signature is as follows:  

```
#[endpoint(proposeEsdtSafeSetCurrentTransactionBatchStatus)]
fn propose_esdt_safe_set_current_transaction_batch_status(
    &self,
    relayer_reward_address: Address,
    #[var_args] tx_batch_status: VarArgs<TransactionStatus>,
) -> SCResult<usize> {
```

`relayer_reward_address` is the address of the relayer that processed the transaction on the Ethereum side. The relayer will receive the transaction fee deposited by the user to compensate for the transaction costs on Ethereum.  

`tx_batch_status` is the status that will be set for each transaction. They are expected to be in the same order as the order in which the transactions were returned to the relayers.  

`TransactionStatus` is an enum:  

```
pub enum TransactionStatus {
    None = 0,
    Pending = 1,
    InProgress = 2,
    Executed = 3,
    Rejected = 4,
}
```

Since only `Executed` and `Rejected` are valid, the status arguments will always be 0x03 or 0x04 respectively.  

And that's about it for Elrond -> Ethereum transactions. The only thing you'll have to figure out yourself is how to decide which relayer executes the transaction and the steps required on the Ethereum side.  

## Ethereum -> Elrond transaction

In this case the process is very simple, as most of the processing will happen on the Ethereum side. Once all that is complete, all the relayers have to do is propose a `BatchTransferEsdtToken` to the `MultiTransferEsdt` SC, with the appropriate receiver, token identifier and amount. When this action is executed, the user will receive their tokens.  

This is done through the `proposeMultiTransferEsdtBatch` endpoint:  

```
#[endpoint(proposeMultiTransferEsdtBatch)]
fn propose_multi_transfer_esdt_batch(
    &self,
    batch_id: u64,
    #[var_args] transfers: MultiArgVec<MultiArg3<Address, TokenIdentifier, Self::BigUint>>,
) -> SCResult<usize> {
```

`batch_id` is an id provided by the relayers. It is used internally to know if an action was proposed for that specific batch.  

## Miscellaneous view functions

```
#[view(getActionIdForBatchId)]
#[storage_mapper("batchIdToActionIdMapping")]
fn batch_id_to_action_id_mapping(
    &self,
    batch_id: u64,
) -> SingleValueMapper<Self::Storage, usize>;
```

Returns the action id for the transaction batch with the id provided as argument. Returns 0 if it does not exist.  

```
#[view(getActionIdForSetCurrentTransactionBatchStatus)]
#[storage_mapper("actionIdForSetCurrentTransactionBatchStatus")]
fn action_id_for_set_current_transaction_batch_status(
    &self,
) -> SingleValueMapper<Self::Storage, usize>;
```

Returns the action id for the `SetCurrentTransactionBatchStatus` action, 0 if it does not exist.  

```
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
```

Returns the mapping between Elrond ESDT token identifier and Ethereum ERC20 contract address.  

```
#[view(getCurrentTxBatch)]
    fn get_current_tx_batch(&self) -> MultiResultVec<TxAsMultiResult<Self::BigUint>>
```

Returns the current transaction batch, each field of each transaction separated by '@'. The result type is defined as follows:

```
pub type TxAsMultiResult<BigUint> =
MultiResult6<BlockNonce, TxNonce, Address, EthAddress, TokenIdentifier, BigUint>;
```

The fields are, in order: block nonce, tx nonce, sender address, receiver address, token type, amount.  

## Conclusion

And that sums up pretty much all the high-level information you'll need to know as a relayer. Through this bridge we hope to be one step closer to bringing all the blockchains together, instead of each being as a lone island.
