////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

#![no_std]

elrond_wasm_node::wasm_endpoints! {
    multi_transfer_esdt
    (
        batchTransferEsdtToken
        getAndClearFirstRefundBatch
        getBatch
        getBatchStatus
        getCurrentTxBatch
        getFirstBatchAnyStatus
        getFirstBatchId
        getLastBatchId
        getMaxBridgedAmount
        getWrappingContractAddress
        setMaxBridgedAmount
        setMaxTxBatchBlockDuration
        setMaxTxBatchSize
        setWrappingContractAddress
    )
}

elrond_wasm_node::wasm_empty_callback! {}
