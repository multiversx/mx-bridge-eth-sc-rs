////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

#![no_std]

elrond_wasm_node::wasm_endpoints! {
    esdt_safe
    (
        addRefundBatch
        addTokenToWhitelist
        calculateRequiredFee
        claimRefund
        createTransaction
        distributeFees
        getAccumulatedTransactionFees
        getAllKnownTokens
        getBatch
        getBatchStatus
        getCurrentTxBatch
        getDefaultPricePerGasUnit
        getEthTxGasLimit
        getFeeEstimatorContractAddress
        getFirstBatchAnyStatus
        getFirstBatchId
        getLastBatchId
        getMaxBridgedAmount
        getRefundAmounts
        isPaused
        pause
        removeTokenFromWhitelist
        setDefaultPricePerGasUnit
        setEthTxGasLimit
        setFeeEstimatorContractAddress
        setMaxBridgedAmount
        setMaxTxBatchBlockDuration
        setMaxTxBatchSize
        setTokenTicker
        setTransactionBatchStatus
        unpause
    )
}

elrond_wasm_node::wasm_empty_callback! {}
