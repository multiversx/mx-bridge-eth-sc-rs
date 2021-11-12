#![no_std]

elrond_wasm::imports!();

pub mod egld_esdt_swap_proxy {
    elrond_wasm::imports!();

    #[elrond_wasm::proxy]
    pub trait EgldEsdtSwap {
        #[init]
        fn init(&self, wrapped_egld_token_id: TokenIdentifier);
    }
}

pub mod esdt_safe_proxy {
    use transaction::{esdt_safe_batch::EsdtSafeTxBatchSplitInFields, TransactionStatus};

    elrond_wasm::imports!();

    #[elrond_wasm::proxy]
    pub trait EsdtSafe {
        #[init]
        fn init(&self, fee_estimator_contract_address: ManagedAddress, eth_tx_gas_limit: BigUint);

        #[endpoint(setFeeEstimatorContractAddress)]
        fn set_fee_estimator_contract_address(&self, new_address: ManagedAddress);

        #[endpoint(setEthTxGasLimit)]
        fn set_eth_tx_gas_limit(&self, new_limit: BigUint);

        #[endpoint(setDefaultPricePerGasUnit)]
        fn set_default_price_per_gas_unit(
            &self,
            token_id: TokenIdentifier,
            default_gwei_price: BigUint,
        );

        #[endpoint(setTokenTicker)]
        fn set_token_ticker(&self, token_id: TokenIdentifier, ticker: ManagedBuffer);

        #[endpoint(addTokenToWhitelist)]
        fn add_token_to_whitelist(
            &self,
            token_id: TokenIdentifier,
            ticker: ManagedBuffer,
            #[var_args] opt_default_price_per_gas_unit: OptionalArg<BigUint>,
        );

        #[endpoint(removeTokenFromWhitelist)]
        fn remove_token_from_whitelist(&self, token_id: TokenIdentifier);

        #[endpoint(setMaxTxBatchSize)]
        fn set_max_tx_batch_size(&self, new_max_tx_batch_size: usize);

        #[endpoint(setMaxTxBatchBlockDuration)]
        fn set_max_tx_batch_block_duration(&self, new_max_tx_batch_block_duration: u64);

        #[endpoint(distributeFees)]
        fn distribute_fees(&self, address_percentage_pairs: Vec<(ManagedAddress, u64)>);

        #[view(getCurrentTxBatch)]
        fn get_current_tx_batch(&self) -> OptionalResult<EsdtSafeTxBatchSplitInFields<Self::Api>>;

        #[endpoint(setTransactionBatchStatus)]
        fn set_transaction_batch_status(
            &self,
            batch_id: u64,
            #[var_args] tx_statuses: VarArgs<TransactionStatus>,
        );
    }
}

pub mod multi_transfer_esdt_proxy {
    use transaction::{SingleTransferTuple, TransactionStatus};

    elrond_wasm::imports!();

    #[elrond_wasm::proxy]
    pub trait MultiTransferEsdt {
        #[init]
        fn init(&self, fee_estimator_contract_address: ManagedAddress, eth_tx_gas_limit: BigUint);

        #[endpoint(setFeeEstimatorContractAddress)]
        fn set_fee_estimator_contract_address(&self, new_address: ManagedAddress);

        #[endpoint(setEthTxGasLimit)]
        fn set_eth_tx_gas_limit(&self, new_limit: BigUint);

        #[endpoint(setDefaultPricePerGasUnit)]
        fn set_default_price_per_gas_unit(
            &self,
            token_id: TokenIdentifier,
            default_gwei_price: BigUint,
        );

        #[endpoint(setTokenTicker)]
        fn set_token_ticker(&self, token_id: TokenIdentifier, ticker: ManagedBuffer);

        #[endpoint(addTokenToWhitelist)]
        fn add_token_to_whitelist(
            &self,
            token_id: TokenIdentifier,
            ticker: ManagedBuffer,
            #[var_args] opt_default_price_per_gas_unit: OptionalArg<BigUint>,
        );

        #[endpoint(removeTokenFromWhitelist)]
        fn remove_token_from_whitelist(&self, token_id: TokenIdentifier);

        #[endpoint(distributeFees)]
        fn distribute_fees(&self, address_percentage_pairs: Vec<(ManagedAddress, u64)>);

        #[endpoint(batchTransferEsdtToken)]
        fn batch_transfer_esdt_token(
            &self,
            #[var_args] transfers: VarArgs<SingleTransferTuple<Self::Api>>,
        ) -> MultiResultVec<TransactionStatus>;
    }
}
