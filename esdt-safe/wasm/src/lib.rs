////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

#![no_std]

elrond_wasm_node::wasm_endpoints! {
    esdt_safe
    (
        init
        addTokenToWhitelist
        calculateRequiredFee
        claimRefund
        createTransaction
        distributeFees
        getAllKnownTokens
        getCurrentTxBatch
        getDefaultPricePerGasUnit
        getEthTxGasLimit
        getFeeEstimatorContractAddress
        getFirstBatch
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
