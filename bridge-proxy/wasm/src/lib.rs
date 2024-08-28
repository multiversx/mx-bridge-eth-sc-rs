// Code generated by the multiversx-sc build system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Upgrade:                              1
// Endpoints:                           12
// Async Callback (empty):               1
// Promise callbacks:                    1
// Total number of exported functions:  16

#![no_std]

multiversx_sc_wasm_adapter::allocator!();
multiversx_sc_wasm_adapter::panic_handler!();

multiversx_sc_wasm_adapter::endpoints! {
    bridge_proxy
    (
        init => init
        upgrade => upgrade
        deposit => deposit
        execute => execute
        getPendingTransactionById => get_pending_transaction_by_id
        getPendingTransactions => get_pending_transactions
        setupMultiTransfer => set_multi_transfer_contract_address
        setBridgedTokensWrapper => set_bridged_tokens_wrapper
        getMultiTransferAddress => multi_transfer_address
        getBridgedTokensWrapperAddress => bridged_tokens_wrapper_address
        lowestTxId => lowest_tx_id
        pause => pause_endpoint
        unpause => unpause_endpoint
        isPaused => paused_status
        execution_callback => execution_callback
    )
}

multiversx_sc_wasm_adapter::async_callback_empty! {}
