deployMultisig() {
    CHECK_VARIABLES RELAYER_ADDR_0 RELAYER_ADDR_1 RELAYER_ADDR_2 RELAYER_ADDR_3 \
    RELAYER_ADDR_4 RELAYER_ADDR_5 RELAYER_ADDR_6 RELAYER_ADDR_7 RELAYER_ADDR_8 \
    RELAYER_ADDR_9 SAFE MULTI_TRANSFER RELAYER_REQUIRED_STAKE SLASH_AMOUNT QUORUM MULTISIG_WASM

    MIN_STAKE=$(echo "$RELAYER_REQUIRED_STAKE*10^18" | bc)
    mxpy --verbose contract deploy --bytecode=${MULTISIG_WASM} --recall-nonce --pem=${ALICE} \
    --gas-limit=200000000 \
    --arguments ${SAFE} ${MULTI_TRANSFER} \
    ${MIN_STAKE} ${SLASH_AMOUNT} ${QUORUM} \
    ${RELAYER_ADDR_0} ${RELAYER_ADDR_1} ${RELAYER_ADDR_2} ${RELAYER_ADDR_3} \
    --send --outfile="deploy-testnet.interaction.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return

    TRANSACTION=$(mxpy data parse --file="./deploy-testnet.interaction.json" --expression="data['emitted_tx']['hash']")
    ADDRESS=$(mxpy data parse --file="./deploy-testnet.interaction.json" --expression="data['contractAddress']")

    mxpy data store --key=address-testnet-multisig --value=${ADDRESS}
    mxpy data store --key=deployTransaction-testnet --value=${TRANSACTION}

    echo ""
    echo "Multisig contract address: ${ADDRESS}"
}

changeChildContractsOwnershipSafe() {
    CHECK_VARIABLES SAFE MULTISIG

    mxpy --verbose contract call ${SAFE} --recall-nonce --pem=${ALICE} \
    --gas-limit=10000000 --function="ChangeOwnerAddress" \
    --arguments ${MULTISIG} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

changeChildContractsOwnershipMultiTransfer() {
    CHECK_VARIABLES MULTI_TRANSFER MULTISIG

    mxpy --verbose contract call ${MULTI_TRANSFER} --recall-nonce --pem=${ALICE} \
    --gas-limit=10000000 --function="ChangeOwnerAddress" \
    --arguments ${MULTISIG} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

clearMapping() {
    CHECK_VARIABLES ERC20_TOKEN CHAIN_SPECIFIC_TOKEN MULTISIG

    mxpy --verbose contract call ${MULTISIG} --recall-nonce --pem=${ALICE} \
    --gas-limit=40000000 --function="clearMapping" \
    --arguments ${ERC20_TOKEN} str:${CHAIN_SPECIFIC_TOKEN} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

addMapping() {
    CHECK_VARIABLES ERC20_TOKEN CHAIN_SPECIFIC_TOKEN MULTISIG

    mxpy --verbose contract call ${MULTISIG} --recall-nonce --pem=${ALICE} \
    --gas-limit=40000000 --function="addMapping" \
    --arguments ${ERC20_TOKEN} str:${CHAIN_SPECIFIC_TOKEN} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

addTokenToWhitelist() {
    CHECK_VARIABLES CHAIN_SPECIFIC_TOKEN CHAIN_SPECIFIC_TOKEN_TICKER MULTISIG

    mxpy --verbose contract call ${MULTISIG} --recall-nonce --pem=${ALICE} \
    --gas-limit=60000000 --function="esdtSafeAddTokenToWhitelist" \
    --arguments str:${CHAIN_SPECIFIC_TOKEN} str:${CHAIN_SPECIFIC_TOKEN_TICKER} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

removeTokenFromWhitelist() {
    CHECK_VARIABLES CHAIN_SPECIFIC_TOKEN CHAIN_SPECIFIC_TOKEN_TICKER MULTISIG

    mxpy --verbose contract call ${MULTISIG} --recall-nonce --pem=${ALICE} \
    --gas-limit=60000000 --function="esdtSafeRemoveTokenFromWhitelist" \
    --arguments str:${CHAIN_SPECIFIC_TOKEN} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

esdtSafeSetMaxTxBatchSize() {
    CHECK_VARIABLES MAX_TX_PER_BATCH MULTISIG

    mxpy --verbose contract call ${MULTISIG} --recall-nonce --pem=${ALICE} \
    --gas-limit=30000000 --function="esdtSafeSetMaxTxBatchSize" \
    --arguments ${MAX_TX_PER_BATCH} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

esdtSafeSetMaxTxBatchBlockDuration() {
    CHECK_VARIABLES MAX_TX_BLOCK_DURATION_PER_BATCH MULTISIG

    mxpy --verbose contract call ${MULTISIG} --recall-nonce --pem=${ALICE} \
    --gas-limit=30000000 --function="esdtSafeSetMaxTxBatchBlockDuration" \
    --arguments ${MAX_TX_BLOCK_DURATION_PER_BATCH} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

clearMapping() {
    CHECK_VARIABLES ERC20_TOKEN CHAIN_SPECIFIC_TOKEN MULTISIG

     mxpy --verbose contract call ${MULTISIG} --recall-nonce --pem=${ALICE} \
    --gas-limit=40000000 --function="clearMapping" \
    --arguments ${ERC20_TOKEN} str:${CHAIN_SPECIFIC_TOKEN} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

changeQuorum() {
    CHECK_VARIABLES QUORUM MULTISIG

    mxpy --verbose contract call ${MULTISIG} --recall-nonce --pem=${ALICE} \
    --gas-limit=40000000 --function="changeQuorum" \
    --arguments ${QUORUM} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

pause() {
    CHECK_VARIABLES MULTISIG

    mxpy --verbose contract call ${MULTISIG} --recall-nonce --pem=${ALICE} \
    --gas-limit=40000000 --function="pause" \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

pauseEsdtSafe() {
    CHECK_VARIABLES MULTISIG

    mxpy --verbose contract call ${MULTISIG} --recall-nonce --pem=${ALICE} \
    --gas-limit=40000000 --function="pauseEsdtSafe" \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

unpause() {
    CHECK_VARIABLES MULTISIG

    mxpy --verbose contract call ${MULTISIG} --recall-nonce --pem=${ALICE} \
    --gas-limit=40000000 --function="unpause" \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

unpauseEsdtSafe() {
    CHECK_VARIABLES MULTISIG

    mxpy --verbose contract call ${MULTISIG} --recall-nonce --pem=${ALICE} \
    --gas-limit=40000000 --function="unpauseEsdtSafe" \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

esdtSafeSetMaxBridgedAmountForToken() {
    CHECK_VARIABLES MAX_AMOUNT NR_DECIMALS_CHAIN_SPECIFIC CHAIN_SPECIFIC_TOKEN MULTISIG

    MAX=$(echo "scale=0; $MAX_AMOUNT*10^$NR_DECIMALS_CHAIN_SPECIFIC/1" | bc)
    mxpy --verbose contract call ${MULTISIG} --recall-nonce --pem=${ALICE} \
    --gas-limit=40000000 --function="esdtSafeSetMaxBridgedAmountForToken" \
    --arguments str:${CHAIN_SPECIFIC_TOKEN} ${MAX} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

multiTransferEsdtSetMaxBridgedAmountForToken() {
    CHECK_VARIABLES MAX_AMOUNT NR_DECIMALS_CHAIN_SPECIFIC CHAIN_SPECIFIC_TOKEN MULTISIG

    MAX=$(echo "scale=0; $MAX_AMOUNT*10^$NR_DECIMALS_CHAIN_SPECIFIC/1" | bc)
    mxpy --verbose contract call ${MULTISIG} --recall-nonce --pem=${ALICE} \
    --gas-limit=40000000 --function="multiTransferEsdtSetMaxBridgedAmountForToken" \
    --arguments str:${CHAIN_SPECIFIC_TOKEN} ${MAX} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}
