# Relayer Documentation

## Introduction

In this document, you will find all the possible actions, scenarios and what is expected of you as a relayer. We will be using the terms `relayer` and `board-member` interchangeably, but they mean the same thing in our case.  

We're assuming all the smart contracts are already setup as described [here](setup.md).    

For details about multisig smart contract design, check here: https://github.com/multiversx/mx-sdk-rs/blob/master/contracts/examples/multisig/README.md.  

If you're interested in a more abstract explanation, check the [readme](../README.md).  

As a relayer, you will interact directly only with the multisig-staking smart contract.  

## Prerequisites for being a relayer

The first and most important prerequisite is being recognized as a board member by the multisig smart contract. Only owner may add board members.  

But that is only the first step. You will not be able to perform any board-member exclusive action until you've staked a certain amount of EGLD in the multisig contract. Once staked, you cannot unstake until your role has been revoked.  This can also happen in two ways:
- The owner removes you from the board member list, in which case you will then be able to unstake your full stake
- The owner "slashes" your stake, you lose your board member role and part of your stake and can unstake the rest.  

Stake "slashing" will only happen if you're actively being malicious. So play nice!  

## MultiversX -> Ethereum transaction

For this kind of transaction, we'll be using the `EsdtSafe` contract. The user will have to submit the transaction through the `EsdtSafe` SC.  

After the user played their part, any relayer may call `fetchNextTransactionBatch`. There is no risk of overwriting, as the function will return an error if the current batch was not executed yet. `fetchNextTransactionBatch` will query the `EsdtSafe` contract and, as the name suggests, get the next N pending transactions. 

The batch will not exceed a pre-defined size, and it will also not bundle transactions that have been created too recently. These are configurable constants in the `EsdtSafe` smart contract.  

The transaction data (block nonce, transaction nonce, sender, receiver, token_id and amount) will be stored inside the multisig SC for further processing. Any one of the relayers will be able to get the transactions and execute them on the Ethereum side.  

Note: Transaction nonce is not the account nonce of the sender account, and rather an internal nonce used by the `EsdtSafe` smart contract. It can safely be ignored by relayers.  

Once the transaction has been executed, a `SetCurrentTransactionBatchStatus` action will be proposed (through the `proposeEsdtSafeSetCurrentTransactionBatchStatus` endpoint), which, for each transaction in the batch, will set the status to "Executed" if it was executed successfully, or "Rejected" if it failed for any reason. Success will burn the tokens on the MultiversX side, while a failure will return the tokens to the user. Endpoint signature is as follows:  

```
#[endpoint(proposeEsdtSafeSetCurrentTransactionBatchStatus)]
fn propose_esdt_safe_set_current_transaction_batch_status(
    &self,
    esdt_safe_batch_id: usize,
     tx_batch_status: VarArgs<TransactionStatus>,
) -> usize
```

`esdt_safe_batch_id` is the ID of the current batch. Each batch will have an assigned ID to make sure relayers always sign the expected batch.  

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

The endpoint returns the assigned Action ID. Other relayers can get this ID by using the following view function:  

```
#[view(getActionIdForSetCurrentTransactionBatchStatus)]
fn get_action_id_for_set_current_transaction_batch_status(
    &self,
    esdt_safe_batch_id: usize,
     expected_tx_batch_status: VarArgs<TransactionStatus>,
) -> usize
```

And that's about it for MutiversX -> Ethereum transactions. The only thing you'll have to figure out yourself is how to decide which relayer executes the transaction and the steps required on the Ethereum side.  

## Ethereum -> MutiversX transaction

In this case the process is very simple, as most of the processing will happen on the Ethereum side. Once all that is complete, all the relayers have to do is propose a `BatchTransferEsdtToken` to the `MultiTransferEsdt` SC, with the appropriate receiver, token identifier and amount. When this action is executed, the user will receive their tokens.  

This is done through the `proposeMultiTransferEsdtBatch` endpoint:  

```
#[endpoint(proposeMultiTransferEsdtBatch)]
fn propose_multi_transfer_esdt_batch(
    &self,
    batch_id: u64,
     transfers: MultiValueVec<MultiValue3<ManagedAddress, TokenIdentifier, BigUint>>,
) -> usize {
```

`batch_id` is an id provided by the relayers. It is used internally to know if an action was proposed for that specific batch.  

`transfers` is a list of (Destination, Token ID, Amount) pairs.  

The endpoint returns the assigned Action ID. Other relayers can get this ID by using the following view function:  

```
#[view(getActionIdForTransferBatch)]
fn get_action_id_for_transfer_batch(
    &self,
    batch_id: u64,
     transfers: MultiValueVec<MultiValue3<ManagedAddress, TokenIdentifier, BigUint>>,
) -> usize
```

## Miscellaneous view functions

```
/// Mapping between ERC20 Ethereum address and MutiversX ESDT Token Identifiers
#[view(getErc20AddressForTokenId)]
#[storage_mapper("erc20AddressForTokenId")]
fn erc20_address_for_token_id(
    &self,
    token_id: &TokenIdentifier,
) -> SingleValueMapper<EthAddress>;

#[view(getTokenIdForErc20Address)]
#[storage_mapper("tokenIdForErc20Address")]
fn token_id_for_erc20_address(
    &self,
    erc20_address: &EthAddress,
) -> SingleValueMapper<TokenIdentifier>;
```

Returns the mapping between MutiversX ESDT token identifier and Ethereum ERC20 contract address. Note that if the mapping returns "EGLD", that means "empty" (Internally, the TokenIdentifier is serialized as "empty" for "EGLD", and as such, "empty" storage is deserialized as "EGLD").  

```
#[view(getCurrentTxBatch)]
fn get_current_tx_batch(&self) -> EsdtSafeTxBatchSplitInFields<BigUint>
```

Returns the current transaction batch, each field of each transaction separated by '@'. The result type is defined as follows:

```
pub type EsdtSafeTxBatchSplitInFields<BigUint> = MultiValue2<usize, MultiValueVec<TxAsMultiValue<BigUint>>>;

pub type TxAsMultiValue<BigUint> =
MultiValue6<BlockNonce, TxNonce, ManagedAddress, EthAddress, TokenIdentifier, BigUint>;
```

The first result is the batch ID, followed by pairs of (block nonce, tx nonce, sender address, receiver address, token type, amount), each as a separate result, i.e. delimited by `@`.  

## Conclusion

And that sums up pretty much all the high-level information you'll need to know as a relayer. Through this bridge we hope to be one step closer to bringing all the blockchains together, instead of each being as a lone island.
