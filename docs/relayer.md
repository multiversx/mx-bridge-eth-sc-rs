# Relayer Documentation

## Introduction

In this document, you will find all the possible actions, scenarios and what is expected of you as a relayer. We will be using the terms `relayer` and `board-member` interchangeably, but they mean the same thing in our case.  

We're assuming all the smart contracts are already setup as described [here](setup.md).    

For details about multisig smart contract design, check [here](https://github.com/ElrondNetwork/elrond-wasm-rs/blob/master/contracts/examples/multisig/README.md).  

If you're interested in a more abstract explanation, check the [readme](../README.md).  

As a relayer, you will interact directly only with the multisig-staking smart contract, except for claiming transaction fees for transactions you executed (claiming will be done through the `EthereumFeePrepay` SC).  

## Prerequisites for being a relayer

The first and most important prerequisite is being recognized as a board member by the multisig smart contract. There are two ways this can happen:
- Owner adds you as board member when deploying the contract
- The current board members vote to add you as a board member alongside them

But that is only the first step. You will not be able to perform any board-member exclusive action until you've staked a certain amount of eGLD in the multisig contract. Once staked, you cannot unstake until your role has been revoked.  This can also happen in two ways:
- You (or another board member) propose to remove you as board memmber, in which case you will then be able to unstake your full stake
- The owner proposes to "slash" your stake, you lose your board member role and part of your stake and can unstake the rest.  

Stake "slashing" will only happen if you're actively being malicious. So play nice!  

## Types of actions

Within the multisig contract, there will be two main types of possible actions:
- general multisig actions (i.e. add/remove member)
- interaction with one of the 4 "child" contracts

The interaction actions can be identified by their specific endpoint name. They usually start with `propose` + the name of the contract + action name. Take `proposeEgldEsdtSwapWrappedEgldIssue` as an example. `propose` + name of the contract (`EgldEsdtSwap`) + action name (`WrappedEgldIssue`).  

Some of the interaction actions can only be used once and are meant as setup actions. The action described above is a good example of that. Since they might fail, we leave them as repeatable (trying to issued wrapped eGLD a second time will fail anyway, so there's no risk of having multiple tokens for the same purpose).  

You can find more about the one-time only actions in the [setup document](setup.md).  

The list of repeatable actions is as follows, grouped by contract. We'll only write the name of the action, not the whole endpoint name. For details like what arguments are required, check the implementation.

- `EgldEsdtSwap`
    - MintWrappedEgld
- `EsdtSafe`
    - AddTokenToWhitelist
    - RemoveTokenFromWhitelist
    - **GetNextPendingTransaction**
    - **SetCurrentTransactionStatus**
- `MultiTransferEsdt`
    - IssueEsdtToken
    - SetLocalMintRole
    - MintEsdtToken
    - **TransferEsdtToken**
- `EthereumFeePrepay`
    - **PayFee**

The endpoints in bold are the ones you'll be interacting with the most. Next we're going to describe the steps needed to perform each kind of transaction: Elrond -> Ethereum and Ethereum -> Elrond.  

## Elrond -> Ethereum transaction

For this kind of transaction, we'll be using the `EsdtSafe` and `EthereumFeePrepay` contracts. The user will first have to pay the transaction fees through the `EthereumFeePrepay` SC, and then to submit the transaction through the `EsdtSafe` SC. To protect against transaction flood, the fee is locked at the time of saving the transaction.  

After the user played their part, the multisig SC will have to propose a `GetNextPendingTransaction` action. Upon execution, it will query the `EsdtSafe` contract and, as the name suggests, get a pending transaction (if any). The transaction data (nonce, sender, receiver, token_id and amount) will be stored inside the multisig SC for further processing. Any one of the relayers will be able to get the transaction and execute it on the Ethereum side.  

Once the transaction has been executed, a `SetCurrentTransactionStatus` action will be proposed, which, upon execution, will set the status to "Executed" if it was executed successfully, or "Rejected" if it failed for any reason. Success will burn the tokens on the Elrond side, while a failure will return the tokens to the user.  

In both cases, the relayer which executed the transaction on the Ethereum side will be able to propose a `PayFee` action. When executed, the cost of transaction fees will be returned to the relayer, in the form of a deposit in the `EthereumFeePrepay` SC. The relayer will be able to withdraw this eGLD at any time.  

And that's about it for Elrond -> Ethereum transactions. The only thing you'll have to figure out yourself is how to decide which relayer executes the transaction and the steps required on the Ethereum side.  

## Ethereum -> Elrond transaction

In this case the process is very simple, as most of the processing will happen on the Ethereum side. Once all that is complete, all the relayers have to do is propose a `TransferEsdtToken` to the `MultiTransferEsdt` SC, with the appropriate receiver, token identifier and amount. When this action is executed, the user will receive their tokens.  

## Conclusion

And that sums up pretty much all the high-level information you'll need to know as a relayer. Through this bridge we hope to be one step closer to bringing all the blockchains together, instead of each being as a lone island.
