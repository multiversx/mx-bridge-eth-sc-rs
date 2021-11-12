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
        getDefaultPricePerGasUnit
        getEthTxGasLimit
        getFeeEstimatorContractAddress
        removeTokenFromWhitelist
        setDefaultPricePerGasUnit
        setEthTxGasLimit
        setFeeEstimatorContractAddress
        setTokenTicker
    )
}

elrond_wasm_node::wasm_empty_callback! {}
