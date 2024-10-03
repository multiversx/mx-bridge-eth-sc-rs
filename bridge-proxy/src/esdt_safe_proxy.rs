// Code generated by the multiversx-sc proxy generator. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

#![allow(dead_code)]
#![allow(clippy::all)]

use multiversx_sc::proxy_imports::*;

pub struct EsdtSafeProxy;

impl<Env, From, To, Gas> TxProxyTrait<Env, From, To, Gas> for EsdtSafeProxy
where
    Env: TxEnv,
    From: TxFrom<Env>,
    To: TxTo<Env>,
    Gas: TxGas<Env>,
{
    type TxProxyMethods = EsdtSafeProxyMethods<Env, From, To, Gas>;

    fn proxy_methods(self, tx: Tx<Env, From, To, (), Gas, (), ()>) -> Self::TxProxyMethods {
        EsdtSafeProxyMethods { wrapped_tx: tx }
    }
}

pub struct EsdtSafeProxyMethods<Env, From, To, Gas>
where
    Env: TxEnv,
    From: TxFrom<Env>,
    To: TxTo<Env>,
    Gas: TxGas<Env>,
{
    wrapped_tx: Tx<Env, From, To, (), Gas, (), ()>,
}

#[rustfmt::skip]
impl<Env, From, Gas> EsdtSafeProxyMethods<Env, From, (), Gas>
where
    Env: TxEnv,
    Env::Api: VMApi,
    From: TxFrom<Env>,
    Gas: TxGas<Env>,
{
    /// fee_estimator_contract_address - The address of a Price Aggregator contract, 
    /// which will get the price of token A in token B 
    ///  
    /// eth_tx_gas_limit - The gas limit that will be used for transactions on the ETH side. 
    /// Will be used to compute the fees for the transfer 
    pub fn init<
        Arg0: ProxyArg<ManagedAddress<Env::Api>>,
        Arg1: ProxyArg<ManagedAddress<Env::Api>>,
        Arg2: ProxyArg<BigUint<Env::Api>>,
    >(
        self,
        fee_estimator_contract_address: Arg0,
        multi_transfer_contract_address: Arg1,
        eth_tx_gas_limit: Arg2,
    ) -> TxTypedDeploy<Env, From, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_deploy()
            .argument(&fee_estimator_contract_address)
            .argument(&multi_transfer_contract_address)
            .argument(&eth_tx_gas_limit)
            .original_result()
    }
}

#[rustfmt::skip]
impl<Env, From, To, Gas> EsdtSafeProxyMethods<Env, From, To, Gas>
where
    Env: TxEnv,
    Env::Api: VMApi,
    From: TxFrom<Env>,
    To: TxTo<Env>,
    Gas: TxGas<Env>,
{
    pub fn upgrade<
        Arg0: ProxyArg<ManagedAddress<Env::Api>>,
        Arg1: ProxyArg<ManagedAddress<Env::Api>>,
        Arg2: ProxyArg<BigUint<Env::Api>>,
    >(
        self,
        fee_estimator_contract_address: Arg0,
        multi_transfer_contract_address: Arg1,
        eth_tx_gas_limit: Arg2,
    ) -> TxTypedUpgrade<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_upgrade()
            .argument(&fee_estimator_contract_address)
            .argument(&multi_transfer_contract_address)
            .argument(&eth_tx_gas_limit)
            .original_result()
    }
}

#[rustfmt::skip]
impl<Env, From, To, Gas> EsdtSafeProxyMethods<Env, From, To, Gas>
where
    Env: TxEnv,
    Env::Api: VMApi,
    From: TxFrom<Env>,
    To: TxTo<Env>,
    Gas: TxGas<Env>,
{
    /// Sets the statuses for the transactions, after they were executed on the Ethereum side. 
    ///  
    /// Only TransactionStatus::Executed (3) and TransactionStatus::Rejected (4) values are allowed. 
    /// Number of provided statuses must be equal to number of transactions in the batch. 
    pub fn set_transaction_batch_status<
        Arg0: ProxyArg<u64>,
        Arg1: ProxyArg<MultiValueEncoded<Env::Api, transaction::transaction_status::TransactionStatus>>,
    >(
        self,
        batch_id: Arg0,
        tx_statuses: Arg1,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("setTransactionBatchStatus")
            .argument(&batch_id)
            .argument(&tx_statuses)
            .original_result()
    }

    /// Converts failed Ethereum -> MultiversX transactions to MultiversX -> Ethereum transaction. 
    /// This is done every now and then to refund the tokens. 
    ///  
    /// As with normal MultiversX -> Ethereum transactions, a part of the tokens will be 
    /// subtracted to pay for the fees 
    pub fn add_refund_batch<
        Arg0: ProxyArg<ManagedVec<Env::Api, transaction::Transaction<Env::Api>>>,
    >(
        self,
        refund_transactions: Arg0,
    ) -> TxTypedCall<Env, From, To, (), Gas, ()> {
        self.wrapped_tx
            .raw_call("addRefundBatch")
            .argument(&refund_transactions)
            .original_result()
    }

    /// Create an MultiversX -> Ethereum transaction. Only fungible tokens are accepted. 
    ///  
    /// Every transfer will have a part of the tokens subtracted as fees. 
    /// The fee amount depends on the global eth_tx_gas_limit 
    /// and the current GWEI price, respective to the bridged token 
    ///  
    /// fee_amount = price_per_gas_unit * eth_tx_gas_limit 
    pub fn create_transaction<
        Arg0: ProxyArg<eth_address::EthAddress<Env::Api>>,
        Arg1: ProxyArg<ManagedAddress<Env::Api>>,
    >(
        self,
        to: Arg0,
        refunding_address: Arg1,
    ) -> TxTypedCall<Env, From, To, (), Gas, ()> {
        self.wrapped_tx
            .raw_call("createTransaction")
            .argument(&to)
            .argument(&refunding_address)
            .original_result()
    }

    pub fn create_transaction_sc_call<
        Arg0: ProxyArg<eth_address::EthAddress<Env::Api>>,
        Arg1: ProxyArg<ManagedBuffer<Env::Api>>,
    >(
        self,
        to: Arg0,
        data: Arg1,
    ) -> TxTypedCall<Env, From, To, (), Gas, ()> {
        self.wrapped_tx
            .raw_call("createTransactionSCCall")
            .argument(&to)
            .argument(&data)
            .original_result()
    }

    /// Claim funds for failed MultiversX -> Ethereum transactions. 
    /// These are not sent automatically to prevent the contract getting stuck. 
    /// For example, if the receiver is a SC, a frozen account, etc. 
    pub fn claim_refund<
        Arg0: ProxyArg<TokenIdentifier<Env::Api>>,
    >(
        self,
        token_id: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, EsdtTokenPayment<Env::Api>> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("claimRefund")
            .argument(&token_id)
            .original_result()
    }

    pub fn init_supply<
        Arg0: ProxyArg<TokenIdentifier<Env::Api>>,
        Arg1: ProxyArg<BigUint<Env::Api>>,
    >(
        self,
        token_id: Arg0,
        amount: Arg1,
    ) -> TxTypedCall<Env, From, To, (), Gas, ()> {
        self.wrapped_tx
            .raw_call("initSupply")
            .argument(&token_id)
            .argument(&amount)
            .original_result()
    }

    pub fn compute_total_amounts_from_index<
        Arg0: ProxyArg<u64>,
        Arg1: ProxyArg<u64>,
    >(
        self,
        startIndex: Arg0,
        endIndex: Arg1,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ManagedVec<Env::Api, EsdtTokenPayment<Env::Api>>> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("computeTotalAmmountsFromIndex")
            .argument(&startIndex)
            .argument(&endIndex)
            .original_result()
    }

    /// Query function that lists all refund amounts for a user. 
    /// Useful for knowing which token IDs to pass to the claimRefund endpoint. 
    pub fn get_refund_amounts<
        Arg0: ProxyArg<ManagedAddress<Env::Api>>,
    >(
        self,
        address: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, MultiValueEncoded<Env::Api, MultiValue2<TokenIdentifier<Env::Api>, BigUint<Env::Api>>>> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("getRefundAmounts")
            .argument(&address)
            .original_result()
    }

    pub fn getTotalRefundAmounts(
        self,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, MultiValueEncoded<Env::Api, MultiValue2<TokenIdentifier<Env::Api>, BigUint<Env::Api>>>> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("getTotalRefundAmounts")
            .original_result()
    }

    pub fn set_fee_estimator_contract_address<
        Arg0: ProxyArg<ManagedAddress<Env::Api>>,
    >(
        self,
        new_address: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("setFeeEstimatorContractAddress")
            .argument(&new_address)
            .original_result()
    }

    pub fn set_eth_tx_gas_limit<
        Arg0: ProxyArg<BigUint<Env::Api>>,
    >(
        self,
        new_limit: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("setEthTxGasLimit")
            .argument(&new_limit)
            .original_result()
    }

    /// Default price being used if the aggregator lacks a mapping for this token 
    /// or the aggregator address is not set 
    pub fn set_default_price_per_gas_unit<
        Arg0: ProxyArg<TokenIdentifier<Env::Api>>,
        Arg1: ProxyArg<BigUint<Env::Api>>,
    >(
        self,
        token_id: Arg0,
        default_price_per_gas_unit: Arg1,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("setDefaultPricePerGasUnit")
            .argument(&token_id)
            .argument(&default_price_per_gas_unit)
            .original_result()
    }

    /// Token ticker being used when querying the aggregator for GWEI prices 
    pub fn set_token_ticker<
        Arg0: ProxyArg<TokenIdentifier<Env::Api>>,
        Arg1: ProxyArg<ManagedBuffer<Env::Api>>,
    >(
        self,
        token_id: Arg0,
        ticker: Arg1,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("setTokenTicker")
            .argument(&token_id)
            .argument(&ticker)
            .original_result()
    }

    /// Returns the fee for the given token ID (the fee amount is in the given token) 
    pub fn calculate_required_fee<
        Arg0: ProxyArg<TokenIdentifier<Env::Api>>,
    >(
        self,
        token_id: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, BigUint<Env::Api>> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("calculateRequiredFee")
            .argument(&token_id)
            .original_result()
    }

    pub fn fee_estimator_contract_address(
        self,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ManagedAddress<Env::Api>> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("getFeeEstimatorContractAddress")
            .original_result()
    }

    pub fn default_price_per_gas_unit<
        Arg0: ProxyArg<TokenIdentifier<Env::Api>>,
    >(
        self,
        token_id: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, BigUint<Env::Api>> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("getDefaultPricePerGasUnit")
            .argument(&token_id)
            .original_result()
    }

    pub fn eth_tx_gas_limit(
        self,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, BigUint<Env::Api>> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("getEthTxGasLimit")
            .original_result()
    }

    /// Distributes the accumulated fees to the given addresses. 
    /// Expected arguments are pairs of (address, percentage), 
    /// where percentages must add up to the PERCENTAGE_TOTAL constant 
    pub fn distribute_fees<
        Arg0: ProxyArg<ManagedVec<Env::Api, token_module::AddressPercentagePair<Env::Api>>>,
    >(
        self,
        address_percentage_pairs: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("distributeFees")
            .argument(&address_percentage_pairs)
            .original_result()
    }

    pub fn add_token_to_whitelist<
        Arg0: ProxyArg<TokenIdentifier<Env::Api>>,
        Arg1: ProxyArg<ManagedBuffer<Env::Api>>,
        Arg2: ProxyArg<bool>,
        Arg3: ProxyArg<bool>,
        Arg4: ProxyArg<OptionalValue<BigUint<Env::Api>>>,
    >(
        self,
        token_id: Arg0,
        ticker: Arg1,
        mint_burn_token: Arg2,
        native_token: Arg3,
        opt_default_price_per_gas_unit: Arg4,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("addTokenToWhitelist")
            .argument(&token_id)
            .argument(&ticker)
            .argument(&mint_burn_token)
            .argument(&native_token)
            .argument(&opt_default_price_per_gas_unit)
            .original_result()
    }

    pub fn remove_token_from_whitelist<
        Arg0: ProxyArg<TokenIdentifier<Env::Api>>,
    >(
        self,
        token_id: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("removeTokenFromWhitelist")
            .argument(&token_id)
            .original_result()
    }

    pub fn get_tokens<
        Arg0: ProxyArg<TokenIdentifier<Env::Api>>,
        Arg1: ProxyArg<BigUint<Env::Api>>,
    >(
        self,
        token_id: Arg0,
        amount: Arg1,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, bool> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("getTokens")
            .argument(&token_id)
            .argument(&amount)
            .original_result()
    }

    pub fn set_multi_transfer_contract_address<
        Arg0: ProxyArg<OptionalValue<ManagedAddress<Env::Api>>>,
    >(
        self,
        opt_new_address: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("setMultiTransferContractAddress")
            .argument(&opt_new_address)
            .original_result()
    }

    pub fn token_whitelist(
        self,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, MultiValueEncoded<Env::Api, TokenIdentifier<Env::Api>>> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("getAllKnownTokens")
            .original_result()
    }

    pub fn native_token<
        Arg0: ProxyArg<TokenIdentifier<Env::Api>>,
    >(
        self,
        token: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, bool> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("isNativeToken")
            .argument(&token)
            .original_result()
    }

    pub fn mint_burn_token<
        Arg0: ProxyArg<TokenIdentifier<Env::Api>>,
    >(
        self,
        token: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, bool> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("isMintBurnToken")
            .argument(&token)
            .original_result()
    }

    pub fn multi_transfer_contract_address(
        self,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ManagedAddress<Env::Api>> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("getMultiTransferContractAddress")
            .original_result()
    }

    pub fn accumulated_transaction_fees<
        Arg0: ProxyArg<TokenIdentifier<Env::Api>>,
    >(
        self,
        token_id: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, BigUint<Env::Api>> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("getAccumulatedTransactionFees")
            .argument(&token_id)
            .original_result()
    }

    pub fn total_balances<
        Arg0: ProxyArg<TokenIdentifier<Env::Api>>,
    >(
        self,
        token_id: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, BigUint<Env::Api>> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("getTotalBalances")
            .argument(&token_id)
            .original_result()
    }

    pub fn mint_balances<
        Arg0: ProxyArg<TokenIdentifier<Env::Api>>,
    >(
        self,
        token_id: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, BigUint<Env::Api>> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("getMintBalances")
            .argument(&token_id)
            .original_result()
    }

    pub fn burn_balances<
        Arg0: ProxyArg<TokenIdentifier<Env::Api>>,
    >(
        self,
        token_id: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, BigUint<Env::Api>> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("getBurnBalances")
            .argument(&token_id)
            .original_result()
    }

    pub fn set_max_tx_batch_size<
        Arg0: ProxyArg<usize>,
    >(
        self,
        new_max_tx_batch_size: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("setMaxTxBatchSize")
            .argument(&new_max_tx_batch_size)
            .original_result()
    }

    pub fn set_max_tx_batch_block_duration<
        Arg0: ProxyArg<u64>,
    >(
        self,
        new_max_tx_batch_block_duration: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("setMaxTxBatchBlockDuration")
            .argument(&new_max_tx_batch_block_duration)
            .original_result()
    }

    pub fn get_current_tx_batch(
        self,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, OptionalValue<MultiValue2<u64, MultiValueEncoded<Env::Api, MultiValue6<u64, u64, ManagedBuffer<Env::Api>, ManagedBuffer<Env::Api>, TokenIdentifier<Env::Api>, BigUint<Env::Api>>>>>> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("getCurrentTxBatch")
            .original_result()
    }

    pub fn get_first_batch_any_status(
        self,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, OptionalValue<MultiValue2<u64, MultiValueEncoded<Env::Api, MultiValue6<u64, u64, ManagedBuffer<Env::Api>, ManagedBuffer<Env::Api>, TokenIdentifier<Env::Api>, BigUint<Env::Api>>>>>> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("getFirstBatchAnyStatus")
            .original_result()
    }

    pub fn get_batch<
        Arg0: ProxyArg<u64>,
    >(
        self,
        batch_id: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, OptionalValue<MultiValue2<u64, MultiValueEncoded<Env::Api, MultiValue6<u64, u64, ManagedBuffer<Env::Api>, ManagedBuffer<Env::Api>, TokenIdentifier<Env::Api>, BigUint<Env::Api>>>>>> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("getBatch")
            .argument(&batch_id)
            .original_result()
    }

    pub fn get_batch_status<
        Arg0: ProxyArg<u64>,
    >(
        self,
        batch_id: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, tx_batch_module::batch_status::BatchStatus<Env::Api>> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("getBatchStatus")
            .argument(&batch_id)
            .original_result()
    }

    pub fn first_batch_id(
        self,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, u64> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("getFirstBatchId")
            .original_result()
    }

    pub fn last_batch_id(
        self,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, u64> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("getLastBatchId")
            .original_result()
    }

    pub fn set_max_bridged_amount<
        Arg0: ProxyArg<TokenIdentifier<Env::Api>>,
        Arg1: ProxyArg<BigUint<Env::Api>>,
    >(
        self,
        token_id: Arg0,
        max_amount: Arg1,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("setMaxBridgedAmount")
            .argument(&token_id)
            .argument(&max_amount)
            .original_result()
    }

    pub fn max_bridged_amount<
        Arg0: ProxyArg<TokenIdentifier<Env::Api>>,
    >(
        self,
        token_id: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, BigUint<Env::Api>> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("getMaxBridgedAmount")
            .argument(&token_id)
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
