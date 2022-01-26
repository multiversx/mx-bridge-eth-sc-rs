////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

#![no_std]

elrond_wasm_node::wasm_endpoints! {
    multi_transfer_esdt
    (
        init
        batchTransferEsdtToken
        getAndClearFirstRefundBatch
        getBatch
        getBatchStatus
        getCurrentTxBatch
        getFirstBatchAnyStatus
        getFirstBatchId
        getLastBatchId
        setMaxTxBatchBlockDuration
        setMaxTxBatchSize
    )
}

elrond_wasm_node::wasm_empty_callback! {}
