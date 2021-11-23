////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

#![no_std]

elrond_wasm_node::wasm_endpoints! {
    multisig
    (
        init
        addBoardMember
        addMapping
        changeDefaultPricePerGasUnit
        changeElrondToEthGasLimit
        changeEthToElrondGasLimit
        changeFeeEstimatorContractAddress
        changeQuorum
        changeTokenTicker
        clearMapping
        distributeFeesFromChildContracts
        esdtSafeAddTokenToWhitelist
        esdtSafeRemoveTokenFromWhitelist
        esdtSafeSetMaxTxBatchBlockDuration
        esdtSafeSetMaxTxBatchSize
        getActionData
        getActionIdForSetCurrentTransactionBatchStatus
        getActionIdForTransferBatch
        getActionLastIndex
        getActionSignerCount
        getActionValidSignerCount
        getAllBoardMembers
        getAllStakedRelayers
        getAmountStaked
        getCurrentRefundBatch
        getCurrentTxBatch
        getErc20AddressForTokenId
        getEsdtSafeAddress
        getMultiTransferEsdtAddress
        getNumBoardMembers
        getQuorum
        getRequiredStakeAmount
        getSlashAmount
        getSlashedTokensAmount
        getTokenIdForErc20Address
        isPaused
        multiTransferEsdtRemoveTokenFromWhitelist
        multiTransferEsdtaddTokenToWhitelist
        pause
        performAction
        proposeEsdtSafeSetCurrentTransactionBatchStatus
        proposeMultiTransferEsdtBatch
        quorumReached
        removeUser
        sign
        signed
        slashBoardMember
        stake
        unpause
        unstake
        upgradeChildContractFromSource
        userRole
        wasActionExecuted
        wasSetCurrentTransactionBatchStatusActionProposed
        wasTransferActionProposed
    )
}

elrond_wasm_node::wasm_empty_callback! {}
