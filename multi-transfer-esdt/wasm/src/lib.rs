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
        getAllKnownTokens
        getAndClearFirstRefundBatch
        getBatch
        getBatchStatus
        getCurrentTxBatch
        getFirstBatchAnyStatus
        getFirstBatchId
        getLastBatchId
        removeTokenFromWhitelist
        setMaxTxBatchBlockDuration
        setMaxTxBatchSize
    )
}

elrond_wasm_node::wasm_empty_callback! {}
