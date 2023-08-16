// Code generated by the multiversx-sc multi-contract system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Endpoints:                           16
// Async Callback (empty):               1
// Total number of exported functions:  18

#![no_std]
#![feature(lang_items)]

multiversx_sc_wasm_adapter::allocator!();
multiversx_sc_wasm_adapter::panic_handler!();

multiversx_sc_wasm_adapter::endpoints! {
    bridged_tokens_wrapper
    (
        init => init
        addWrappedToken => add_wrapped_token
        updateWrappedToken => update_wrapped_token
        removeWrappedToken => remove_wrapped_token
        whitelistToken => whitelist_token
        updateWhitelistedToken => update_whitelisted_token
        blacklistToken => blacklist_token
        depositLiquidity => deposit_liquidity
        wrapTokens => wrap_tokens
        unwrapToken => unwrap_token
        getUniversalBridgedTokenIds => universal_bridged_token_ids
        getTokenLiquidity => token_liquidity
        getChainSpecificToUniversalMapping => chain_specific_to_universal_mapping
        getchainSpecificTokenIds => chain_specific_token_ids
        pause => pause_endpoint
        unpause => unpause_endpoint
        isPaused => paused_status
    )
}

multiversx_sc_wasm_adapter::async_callback_empty! {}
