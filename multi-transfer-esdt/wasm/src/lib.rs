////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

#![no_std]

elrond_wasm_node::wasm_endpoints! {
    multi_transfer_esdt
    (
        init
        addTokenToWhitelist
        batchTransferEsdtToken
        calculateRequiredFee
        distributeFees
        getAllKnownTokens
        getBatch
        getCurrentTxBatch
        getDefaultPricePerGasUnit
        getEthTxGasLimit
        getFeeEstimatorContractAddress
        getFirstBatchId
        getLastBatchId
        removeTokenFromWhitelist
        setDefaultPricePerGasUnit
        setEthTxGasLimit
        setFeeEstimatorContractAddress
        setMaxTxBatchBlockDuration
        setMaxTxBatchSize
        setTokenTicker
    )
}

elrond_wasm_node::wasm_empty_callback! {}
