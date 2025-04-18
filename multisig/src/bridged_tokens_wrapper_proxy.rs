// Code generated by the multiversx-sc proxy generator. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

#![allow(dead_code)]
#![allow(clippy::all)]

use multiversx_sc::proxy_imports::*;

pub struct BridgedTokensWrapperProxy;

impl<Env, From, To, Gas> TxProxyTrait<Env, From, To, Gas> for BridgedTokensWrapperProxy
where
    Env: TxEnv,
    From: TxFrom<Env>,
    To: TxTo<Env>,
    Gas: TxGas<Env>,
{
    type TxProxyMethods = BridgedTokensWrapperProxyMethods<Env, From, To, Gas>;

    fn proxy_methods(self, tx: Tx<Env, From, To, (), Gas, (), ()>) -> Self::TxProxyMethods {
        BridgedTokensWrapperProxyMethods { wrapped_tx: tx }
    }
}

pub struct BridgedTokensWrapperProxyMethods<Env, From, To, Gas>
where
    Env: TxEnv,
    From: TxFrom<Env>,
    To: TxTo<Env>,
    Gas: TxGas<Env>,
{
    wrapped_tx: Tx<Env, From, To, (), Gas, (), ()>,
}

#[rustfmt::skip]
impl<Env, From, Gas> BridgedTokensWrapperProxyMethods<Env, From, (), Gas>
where
    Env: TxEnv,
    Env::Api: VMApi,
    From: TxFrom<Env>,
    Gas: TxGas<Env>,
{
    pub fn init(
        self,
    ) -> TxTypedDeploy<Env, From, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_deploy()
            .original_result()
    }
}

#[rustfmt::skip]
impl<Env, From, To, Gas> BridgedTokensWrapperProxyMethods<Env, From, To, Gas>
where
    Env: TxEnv,
    Env::Api: VMApi,
    From: TxFrom<Env>,
    To: TxTo<Env>,
    Gas: TxGas<Env>,
{
    pub fn upgrade(
        self,
    ) -> TxTypedUpgrade<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_upgrade()
            .original_result()
    }
}

#[rustfmt::skip]
impl<Env, From, To, Gas> BridgedTokensWrapperProxyMethods<Env, From, To, Gas>
where
    Env: TxEnv,
    Env::Api: VMApi,
    From: TxFrom<Env>,
    To: TxTo<Env>,
    Gas: TxGas<Env>,
{
    pub fn add_wrapped_token<
        Arg0: ProxyArg<TokenIdentifier<Env::Api>>,
        Arg1: ProxyArg<u32>,
    >(
        self,
        universal_bridged_token_ids: Arg0,
        num_decimals: Arg1,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("addWrappedToken")
            .argument(&universal_bridged_token_ids)
            .argument(&num_decimals)
            .original_result()
    }

    pub fn update_wrapped_token<
        Arg0: ProxyArg<TokenIdentifier<Env::Api>>,
        Arg1: ProxyArg<u32>,
    >(
        self,
        universal_bridged_token_ids: Arg0,
        num_decimals: Arg1,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("updateWrappedToken")
            .argument(&universal_bridged_token_ids)
            .argument(&num_decimals)
            .original_result()
    }

    pub fn remove_wrapped_token<
        Arg0: ProxyArg<TokenIdentifier<Env::Api>>,
    >(
        self,
        universal_bridged_token_ids: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("removeWrappedToken")
            .argument(&universal_bridged_token_ids)
            .original_result()
    }

    pub fn whitelist_token<
        Arg0: ProxyArg<TokenIdentifier<Env::Api>>,
        Arg1: ProxyArg<u32>,
        Arg2: ProxyArg<TokenIdentifier<Env::Api>>,
    >(
        self,
        chain_specific_token_id: Arg0,
        chain_specific_token_decimals: Arg1,
        universal_bridged_token_ids: Arg2,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("whitelistToken")
            .argument(&chain_specific_token_id)
            .argument(&chain_specific_token_decimals)
            .argument(&universal_bridged_token_ids)
            .original_result()
    }

    pub fn update_whitelisted_token<
        Arg0: ProxyArg<TokenIdentifier<Env::Api>>,
        Arg1: ProxyArg<u32>,
    >(
        self,
        chain_specific_token_id: Arg0,
        chain_specific_token_decimals: Arg1,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("updateWhitelistedToken")
            .argument(&chain_specific_token_id)
            .argument(&chain_specific_token_decimals)
            .original_result()
    }

    pub fn blacklist_token<
        Arg0: ProxyArg<TokenIdentifier<Env::Api>>,
    >(
        self,
        chain_specific_token_id: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("blacklistToken")
            .argument(&chain_specific_token_id)
            .original_result()
    }

    pub fn deposit_liquidity(
        self,
    ) -> TxTypedCall<Env, From, To, (), Gas, ()> {
        self.wrapped_tx
            .raw_call("depositLiquidity")
            .original_result()
    }

    /// Will wrap what it can, and send back the rest unchanged 
    pub fn wrap_tokens(
        self,
    ) -> TxTypedCall<Env, From, To, (), Gas, ManagedVec<Env::Api, EsdtTokenPayment<Env::Api>>> {
        self.wrapped_tx
            .raw_call("wrapTokens")
            .original_result()
    }

    pub fn unwrap_token<
        Arg0: ProxyArg<TokenIdentifier<Env::Api>>,
    >(
        self,
        requested_token: Arg0,
    ) -> TxTypedCall<Env, From, To, (), Gas, ()> {
        self.wrapped_tx
            .raw_call("unwrapToken")
            .argument(&requested_token)
            .original_result()
    }

    pub fn unwrap_token_create_transaction<
        Arg0: ProxyArg<TokenIdentifier<Env::Api>>,
        Arg1: ProxyArg<ManagedAddress<Env::Api>>,
        Arg2: ProxyArg<eth_address::EthAddress<Env::Api>>,
    >(
        self,
        requested_token: Arg0,
        safe_address: Arg1,
        to: Arg2,
    ) -> TxTypedCall<Env, From, To, (), Gas, ()> {
        self.wrapped_tx
            .raw_call("unwrapTokenCreateTransaction")
            .argument(&requested_token)
            .argument(&safe_address)
            .argument(&to)
            .original_result()
    }

    pub fn universal_bridged_token_ids(
        self,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, MultiValueEncoded<Env::Api, TokenIdentifier<Env::Api>>> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("getUniversalBridgedTokenIds")
            .original_result()
    }

    pub fn token_liquidity<
        Arg0: ProxyArg<TokenIdentifier<Env::Api>>,
    >(
        self,
        token: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, BigUint<Env::Api>> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("getTokenLiquidity")
            .argument(&token)
            .original_result()
    }

    pub fn chain_specific_to_universal_mapping<
        Arg0: ProxyArg<TokenIdentifier<Env::Api>>,
    >(
        self,
        token: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, TokenIdentifier<Env::Api>> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("getChainSpecificToUniversalMapping")
            .argument(&token)
            .original_result()
    }

    pub fn chain_specific_token_ids<
        Arg0: ProxyArg<TokenIdentifier<Env::Api>>,
    >(
        self,
        universal_token_id: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, MultiValueEncoded<Env::Api, TokenIdentifier<Env::Api>>> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("getchainSpecificTokenIds")
            .argument(&universal_token_id)
            .original_result()
    }

    pub fn pause_endpoint(
        self,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("pause")
            .original_result()
    }

    pub fn unpause_endpoint(
        self,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("unpause")
            .original_result()
    }

    pub fn paused_status(
        self,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, bool> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("isPaused")
            .original_result()
    }
}
