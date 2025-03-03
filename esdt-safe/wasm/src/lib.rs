// Code generated by the multiversx-sc build system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Upgrade:                              1
// Endpoints:                           46
// Async Callback (empty):               1
// Total number of exported functions:  49

#![no_std]

multiversx_sc_wasm_adapter::allocator!();
multiversx_sc_wasm_adapter::panic_handler!();

multiversx_sc_wasm_adapter::endpoints! {
    esdt_safe
    (
        init => init
        upgrade => upgrade
        setTransactionBatchStatus => set_transaction_batch_status
        addRefundBatch => add_refund_batch
        addRefundBatchForFailedTx => add_refund_batch_for_failed_tx
        createTransaction => create_transaction
        createRefundTransaction => create_refund_transaction
        claimRefund => claim_refund
        withdrawRefundFeesForEthereum => withdraw_refund_fees_for_ethereum
        withdrawTransactionFees => withdraw_transaction_fees
        addToBatchEndpoint => add_to_batch_endpoint
        computeTotalAmmountsFromIndex => compute_total_amounts_from_index
        getRefundAmounts => get_refund_amounts
        getTotalRefundAmounts => get_total_refund_amounts
        getRefundFeesForEthereum => get_refund_fees_for_ethereum
        getTransactionFees => get_transaction_fees
        setEthTxGasLimit => set_eth_tx_gas_limit
        setDefaultPricePerGasUnit => set_default_price_per_gas_unit
        setTokenTicker => set_token_ticker
        calculateRequiredFee => calculate_required_fee
        getDefaultPricePerGasUnit => default_price_per_gas_unit
        getEthTxGasLimit => eth_tx_gas_limit
        distributeFees => distribute_fees
        addTokenToWhitelist => add_token_to_whitelist
        removeTokenFromWhitelist => remove_token_from_whitelist
        getTokens => get_tokens
        initSupply => init_supply
        initSupplyMintBurn => init_supply_mint_burn
        getAllKnownTokens => token_whitelist
        isNativeToken => native_token
        isMintBurnToken => mint_burn_token
        getAccumulatedTransactionFees => accumulated_transaction_fees
        getTotalBalances => total_balances
        getMintBalances => mint_balances
        getBurnBalances => burn_balances
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
