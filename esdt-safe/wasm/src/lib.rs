// Code generated by the multiversx-sc build system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Endpoints:                           37
// Async Callback (empty):               1
// Total number of exported functions:  39

#![no_std]

// Configuration that works with rustc < 1.73.0.
// TODO: Recommended rustc version: 1.73.0 or newer.
#![feature(lang_items)]

multiversx_sc_wasm_adapter::allocator!();
multiversx_sc_wasm_adapter::panic_handler!();

multiversx_sc_wasm_adapter::endpoints! {
    esdt_safe
    (
        init => init
        upgrade => upgrade
        setTransactionBatchStatus => set_transaction_batch_status
        addRefundBatch => add_refund_batch
        createTransaction => create_transaction
        claimRefund => claim_refund
        getRefundAmounts => get_refund_amounts
        setFeeEstimatorContractAddress => set_fee_estimator_contract_address
        setEthTxGasLimit => set_eth_tx_gas_limit
        setDefaultPricePerGasUnit => set_default_price_per_gas_unit
        setTokenTicker => set_token_ticker
        calculateRequiredFee => calculate_required_fee
        getFeeEstimatorContractAddress => fee_estimator_contract_address
        getDefaultPricePerGasUnit => default_price_per_gas_unit
        getEthTxGasLimit => eth_tx_gas_limit
        distributeFees => distribute_fees
        addTokenToWhitelist => add_token_to_whitelist
        removeTokenFromWhitelist => remove_token_from_whitelist
        mintToken => mint_token
        setMultiTransferContractAddress => set_multi_transfer_contract_address
        getAllKnownTokens => token_whitelist
        isWhitelistedTokenMintBurn => mint_burn_allowed
        getMultiTransferContractAddress => multi_transfer_contract_address
        getAccumulatedTransactionFees => accumulated_transaction_fees
        getAccumulatedBurnedTokens => accumulated_burned_tokens
        setMaxTxBatchSize => set_max_tx_batch_size
        setMaxTxBatchBlockDuration => set_max_tx_batch_block_duration
        getCurrentTxBatch => get_current_tx_batch
        getFirstBatchAnyStatus => get_first_batch_any_status
        getBatch => get_batch
        getBatchStatus => get_batch_status
        getFirstBatchId => first_batch_id
        getLastBatchId => last_batch_id
        setMaxBridgedAmount => set_max_bridged_amount
        getMaxBridgedAmount => max_bridged_amount
        pause => pause_endpoint
        unpause => unpause_endpoint
        isPaused => paused_status
    )
}

multiversx_sc_wasm_adapter::async_callback_empty! {}
