////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

#![no_std]

elrond_wasm_node::wasm_endpoints! {
    esdt_safe
    (
        init
        addRefundBatch
        addTokenToWhitelist
        calculateRequiredFee
        claimRefund
        createTransaction
        distributeFees
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
        getRefundAmounts
        removeTokenFromWhitelist
        setDefaultPricePerGasUnit
        setEthTxGasLimit
        setFeeEstimatorContractAddress
        setMaxTxBatchBlockDuration
        setMaxTxBatchSize
        setTokenTicker
        setTransactionBatchStatus
    )
}

elrond_wasm_node::wasm_empty_callback! {}
