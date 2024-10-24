// Code generated by the multiversx-sc proxy generator. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

#![allow(dead_code)]
#![allow(clippy::all)]

use multiversx_sc::proxy_imports::*;

pub struct BridgeProxyContractProxy;

impl<Env, From, To, Gas> TxProxyTrait<Env, From, To, Gas> for BridgeProxyContractProxy
where
    Env: TxEnv,
    From: TxFrom<Env>,
    To: TxTo<Env>,
    Gas: TxGas<Env>,
{
    type TxProxyMethods = BridgeProxyContractProxyMethods<Env, From, To, Gas>;

    fn proxy_methods(self, tx: Tx<Env, From, To, (), Gas, (), ()>) -> Self::TxProxyMethods {
        BridgeProxyContractProxyMethods { wrapped_tx: tx }
    }
}

pub struct BridgeProxyContractProxyMethods<Env, From, To, Gas>
where
    Env: TxEnv,
    From: TxFrom<Env>,
    To: TxTo<Env>,
    Gas: TxGas<Env>,
{
    wrapped_tx: Tx<Env, From, To, (), Gas, (), ()>,
}

#[rustfmt::skip]
impl<Env, From, Gas> BridgeProxyContractProxyMethods<Env, From, (), Gas>
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
impl<Env, From, To, Gas> BridgeProxyContractProxyMethods<Env, From, To, Gas>
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
impl<Env, From, To, Gas> BridgeProxyContractProxyMethods<Env, From, To, Gas>
where
    Env: TxEnv,
    Env::Api: VMApi,
    From: TxFrom<Env>,
    To: TxTo<Env>,
    Gas: TxGas<Env>,
{
    pub fn deposit<
        Arg0: ProxyArg<transaction::EthTransaction<Env::Api>>,
        Arg1: ProxyArg<u64>,
    >(
        self,
        eth_tx: Arg0,
        batch_id: Arg1,
    ) -> TxTypedCall<Env, From, To, (), Gas, ()> {
        self.wrapped_tx
            .raw_call("deposit")
            .argument(&eth_tx)
            .argument(&batch_id)
            .original_result()
    }

    pub fn execute<
        Arg0: ProxyArg<usize>,
    >(
        self,
        tx_id: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("execute")
            .argument(&tx_id)
            .original_result()
    }

    pub fn update_lowest_tx_id(
        self,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("updateLowestTxId")
            .original_result()
    }

    pub fn get_pending_transaction_by_id<
        Arg0: ProxyArg<usize>,
    >(
        self,
        tx_id: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, transaction::EthTransaction<Env::Api>> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("getPendingTransactionById")
            .argument(&tx_id)
            .original_result()
    }

    pub fn get_pending_transactions(
        self,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, MultiValueEncoded<Env::Api, MultiValue2<usize, transaction::EthTransaction<Env::Api>>>> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("getPendingTransactions")
            .original_result()
    }

    pub fn lowest_tx_id(
        self,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, usize> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("lowestTxId")
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
