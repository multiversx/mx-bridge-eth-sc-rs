# Initial Setup

The bridge suite is comprised of multiple smart contracts. First and foremost, you'll need to setup an `Aggregator` smart contract, which will be used to get the approximate cost of transactions on Ethereum at a certain point in time. More details on this in the workflows description, but basically, fees are currently very high on the Ethereum blockchain, so we can't expect the relayers to handle the costs.  

Next, we're going to setup the main "controller" contract, which will be a multisig-style SC. You can find more details about this type of smart contract [here](https://github.com/ElrondNetwork/elrond-wasm-rs/blob/master/contracts/examples/multisig/README.md)  

Basically, we will have a certain number of board members (in this case we will call them "relayers") which will validate transactions and take the appropriate actions. As this is a multisig contract, at least a certain number of members must agree to the action, otherwise it cannot be performed.  

Once the multisig contract is deployed and setup properly, the owner must call the `deployChildContracts` to deploy `EgldEsdtSwap`, `EsdtSafe`, `MultiTransferEsdt` and `EthereumFeePrepay` contracts.  The endpoint looks like this:  

```
    #[endpoint(deployChildContracts)]
    fn deploy_child_contracts(
        &self,
        egld_esdt_swap_code: BoxedBytes,
        multi_transfer_esdt_code: BoxedBytes,
        ethereum_fee_prepay_code: BoxedBytes,
        esdt_safe_code: BoxedBytes,
        aggregator_address: Address,
        #[var_args] esdt_safe_token_whitelist: VarArgs<TokenIdentifier>
```

The `_code` arguments are the compiled wasm bytecode of the respective contracts. `aggregator_address` is the aggregator described in the intro (has to be deployed already, preferably by the same account as the multisig one), and optionally you can add any number of tokens to the initial whitelist, each as a different argument.  

To complete the setup, there is one more step that has to be performed: We have to add the EsdtSafe to EthereumFeePrepay whitelist. This is done through the `finishSetup` endpoint (which is owner-only). No arguments required for this one.  

```
    #[endpoint(finishSetup)]
    fn finish_setup(&self)
```

Once those are setup, the individual contracts will need a bit more setup themselves. These steps will be done through the multisig propose-sign-perform action flow.  

# EgldEsdtSwap

The EgldEsdtSwap SC requires wrapped eGLD token to be issued and to have the local-mint role set. As these have to be done through calls to a system SC, calls will be cross-shard, so we can only do this setup in two separate steps.  

Issuing is done through the `issueWrappedEgld` endpoint. 

```
    #[payable("EGLD")]
    #[endpoint(issueWrappedEgld)]
    fn issue_wrapped_egld(
        &self,
        token_display_name: BoxedBytes,
        token_ticker: BoxedBytes,
        initial_supply: BigUint,
        #[payment] issue_cost: BigUint,
    )
```

To get the assigned identifier, query the `getWrappedEgldTokenIdentifier` view function.  

At the time of writing this document, the issue cost is 5 eGLD. This might change in the future, so we didn't enforce the exact payment in the contract. The issue will fail if the payment value is wrong.  

Once the issue is complete, we have to set the local mint role for the newly created token. This makes it so the contract doesn't have to wait for the owner to mint more tokens when it runs out, and it can simply mint more of them locally, similar to the ERC20-style contracts on Ethereum. Setting roles is done by calling `setLocalMintRole` with no arguments.  

# MultiTransferEsdt

The contract is made to be as generic as possible (i.e. handle an "infinite" number of tokens), but in this case, all we need to do is very similar to the `EgldEsdtSwap`: We have to issue a token (which we'll call `WrappedEth`, with `WETH` as token ticker) and setup local mint role for it. As we can't store native ETH tokens on Elrond, we have "wrap" them in an ESDT token.  

Issuing is done by calling `issueEsdtToken`:
```
    #[payable("EGLD")]
    #[endpoint(issueEsdtToken)]
    fn issue_esdt_token_endpoint(
        &self,
        token_display_name: BoxedBytes,
        token_ticker: BoxedBytes,
        initial_supply: BigUint,
        #[payment] issue_cost: BigUint,
    )
```

Now we have to set local roles for this token through the `setLocalMintRole` endpoint. As this SC is made to handle multiple types of tokens, we also have to specify the token identifier when calling the function.

```
    #[endpoint(setLocalMintRole)]
    fn set_local_mint_role(
        &self,
        token_id: TokenIdentifier,
        #[var_args] opt_address: OptionalArg<Address>,
    )
```

In this case, we will only provide the token identifier. Optional argument defaults to contract's own address, which is what we want in this case.

Since we have no way of knowing the identifier, we query the handy `getLastIssuedToken` view function to get it. If the result is "EGLD", then the issue failed and you'll have to re-check the arguments you provided for token issue, then try issuing again.  

# EsdtSafe

The only additional setup this contract requires is adding `WrappedEgld` and `WrappedEth` to the token whitelist, which is done by calling the following endpoint twice, once for each token:

```
    #[endpoint(addTokenToWhitelist)]
    fn add_token_to_whitelist(&self, token_id: TokenIdentifier)
```

Remember, `EgldEsdtSwap` and `MultiTransferEsdt` both have handy view functions to provide you with the relevant token identifiers.  

## Ethereum Fee Prepay

This contract requires no additional setup.  

Setup is now complete! Now let's discuss the use-cases, workflows and more, in the [readme](../README.md) document.
