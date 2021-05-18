# Initial Setup

The bridge suite is comprised of multiple smart contracts. First and foremost, you'll need to setup an `Aggregator` smart contract, which will be used to get the approximate cost of transactions on Ethereum at a certain point in time. More details on this in the workflows description, but basically, fees are currently very high on the Ethereum blockchain, so we can't expect the relayers to handle the costs.  

Additionally, you will have to issue at least two ESDT tokens (suggested parameters in paranthesis):  
- Wrapped EGLD (name: "WrappedEgld", ticker: "WEGLD", initial supply: 1, decimals: 18)
- Wrapped ETH (name: "WrappedEth", ticker: "WETH", initial supply: 1, decimals: 18)

You can find more about how to issue an ESDT token here: https://docs.elrond.com/developers/esdt-tokens/#issuance-of-fungible-esdt-tokens  

Next, we're going to setup the main "controller" contract, which will be a multisig-style SC. You can find more details about this type of smart contract here: https://github.com/ElrondNetwork/elrond-wasm-rs/blob/master/contracts/examples/multisig/README.md  

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
        wrapped_egld_token_id: TokenIdentifier,
        #[var_args] token_whitelist: VarArgs<TokenIdentifier>
```

The `_code` arguments are the compiled wasm bytecode of the respective contracts. `aggregator_address` is the aggregator described in the intro (has to be deployed already). 

`wrapped_egld_token_id` is the token identifier of the previously issued "WrappedEgld" token (Note: identifier format is ticker + '-' + 6 random characters). For WrappedEgld, it might look something like "WEGLD-123456".  

`token_whitelist` is a list of tokens already issued that will be used by the bridge, in our case, that will be only one: The "WrappedEth" token.  

To complete the setup, there is one more step that has to be performed: We have to add the EsdtSafe to EthereumFeePrepay whitelist. This is done through the `finishSetup` endpoint (which is owner-only). No arguments required for this one.  

```
    #[endpoint(finishSetup)]
    fn finish_setup(&self)
```

Once those are setup, the individual contracts will need a bit more setup themselves.   

# EgldEsdtSwap 

Requires having local MINT and local BURN roles for the WEGLD token. More info about how to set local roles can be found here: https://docs.elrond.com/developers/esdt-tokens/#setting-and-unsetting-special-roles

# MultiTransferEsdt

Requires local MINT role set for every token added to the whitelist. In this case, those tokens will be the WEGLD and WETH tokens.  

# EsdtSafe

Requires local BURN role set for every token added to the whitelist. In this case, those tokens will be the WEGLD and WETH tokens.  

# End of Setup 

Setup is now complete! Now let's discuss the use-cases, workflows and more, in the [readme](../README.md) document.
