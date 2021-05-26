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

After the user played their part, anyone may call `getNextPendingTransaction`. There is no risk of overwriting a transaction, as the function will return an error if the current transaction was not executed yet. `getNextPendingTransaction` will query the `EsdtSafe` contract and, as the name suggests, get a pending transaction (if any). The transaction data (block nonce, transaction nonce, sender, receiver, token_id and amount) will be stored inside the multisig SC for further processing. Any one of the relayers will be able to get the transaction and execute it on the Ethereum side.  

Note: Transaction nonce is not the account nonce of the sender account, and rather an internal nonce used by the `EsdtSafe` smart contract. It can safely be ignored by relayers.  

Once the transaction has been executed, a `SetCurrentTransactionStatus` action will be proposed (through the `proposeEsdtSafeSetCurrentTransactionStatus` endpoint), which will set the status to "Executed" if it was executed successfully, or "Rejected" if it failed for any reason. Success will burn the tokens on the Elrond side, while a failure will return the tokens to the user. Endpoint signature is as follows:  

```
#[endpoint(proposeEsdtSafeSetCurrentTransactionStatus)]
fn propose_esdt_safe_set_current_transaction_status(
    &self,
    relayer_reward_address: Address,
    transaction_status: TransactionStatus,
) -> SCResult<usize> {
```

`relayer_reward_address` is the address of the relayer that processed the transaction on the Ethereum side. The relayer will receive the transaction fee deposited by the user to compensate for the transaction costs on Ethereum.  

`transaction_status` is the status that will be set. `TransactionStatus` is an enum:  

```
pub enum TransactionStatus {
    None = 0,
    Pending = 1,
    InProgress = 2,
    Executed = 3,
    Rejected = 4,
}
```

Since only `Executed` and `Rejected` are valid, the argument will always be 0x03 or 0x04 respectively.  

And that's about it for Elrond -> Ethereum transactions. The only thing you'll have to figure out yourself is how to decide which relayer executes the transaction and the steps required on the Ethereum side.  

## Ethereum -> Elrond transaction

In this case the process is very simple, as most of the processing will happen on the Ethereum side. Once all that is complete, all the relayers have to do is propose a `TransferEsdtToken` to the `MultiTransferEsdt` SC, with the appropriate receiver, token identifier and amount. When this action is executed, the user will receive their tokens.  

This is done through the `proposeMultiTransferEsdtTransferEsdtToken` endpoint:  

```
#[endpoint(proposeMultiTransferEsdtTransferEsdtToken)]
fn propose_multi_transfer_esdt_transfer_esdt_token(
    &self,
    eth_tx_nonce: u64,
    to: Address,
    token_id: TokenIdentifier,
    amount: Self::BigUint,
) -> SCResult<usize>
```

`eth_tx_nonce` is the nonce of the transaction from the Ethereum side. It is used internally to know if an action was proposed for that specific tx.  

## Miscellaneous view functions

```
#[view(getActionIdForEthTxNonce)]
#[storage_mapper("ethTxNonceToActionIdMapping")]
fn eth_tx_nonce_to_action_id_mapping(
    &self,
    eth_tx_nonce: u64,
) -> SingleValueMapper<Self::Storage, usize>;
```

Returns the action id for the transaction with the nonce given as argument. Returns 0 if it does not exist.  

```
#[view(getActionIdForSetCurrentTransactionStatus)]
#[storage_mapper("actionIdForSetCurrentTransactionStatus")]
fn action_id_for_set_current_transaction_status(
    &self,
) -> SingleValueMapper<Self::Storage, usize>;
```

Returns the action id for the `SetCurrentTransactionStatus` action, 0 if it does not exist.  

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
#[view(getCurrentTx)]
fn get_current_tx(&self) -> OptionalResult<TxAsMultiResult<Self::BigUint>>
```

Returns the current transaction, each field separated by '@'. The result type is defined as follows:

```
pub type TxAsMultiResult<BigUint> =
MultiResult6<BlockNonce, TxNonce, Address, EthAddress, TokenIdentifier, BigUint>;
```

The fields are, in order: block nonce, tx nonce, sender address, receiver address, token type, amount.  

## Conclusion

And that sums up pretty much all the high-level information you'll need to know as a relayer. Through this bridge we hope to be one step closer to bringing all the blockchains together, instead of each being as a lone island.
