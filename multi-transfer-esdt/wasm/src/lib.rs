// Code generated by the multiversx-sc build system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Upgrade:                              1
// Endpoints:                           13
// Async Callback (empty):               1
// Promise callbacks:                    1
// Total number of exported functions:  17

#![no_std]

multiversx_sc_wasm_adapter::allocator!();
multiversx_sc_wasm_adapter::panic_handler!();

multiversx_sc_wasm_adapter::endpoints! {
    multi_transfer_esdt
    (
        init => init
        upgrade => upgrade
        batchTransferEsdtToken => batch_transfer_esdt_token
        moveRefundBatchToSafe => move_refund_batch_to_safe
        addUnprocessedRefundTxToBatch => add_unprocessed_refund_tx_to_batch
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
        transfer_callback => transfer_callback
    )
}

multiversx_sc_wasm_adapter::async_callback_empty! {}
