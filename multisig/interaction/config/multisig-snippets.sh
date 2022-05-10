deployMultisig() {
    erdpy --verbose contract deploy --bytecode=${MULTISIG_WASM} --recall-nonce --pem=${ALICE} \
    --gas-limit=200000000 \
    --arguments ${SAFE} ${MULTI_TRANSFER} \
    ${RELAYER_REQUIRED_STAKE} ${SLASH_AMOUNT} ${QUORUM} \
    ${RELAYER_ADDR_0} ${RELAYER_ADDR_1} ${RELAYER_ADDR_2} ${RELAYER_ADDR_3} \
    ${RELAYER_ADDR_4} ${RELAYER_ADDR_5} ${RELAYER_ADDR_6} ${RELAYER_ADDR_7} \
    ${RELAYER_ADDR_8} ${RELAYER_ADDR_9} \
    --send --outfile="deploy-testnet.interaction.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return

    TRANSACTION=$(erdpy data parse --file="./deploy-testnet.interaction.json" --expression="data['emitted_tx']['hash']")
    ADDRESS=$(erdpy data parse --file="./deploy-testnet.interaction.json" --expression="data['contractAddress']")

    erdpy data store --key=address-testnet-multisig --value=${ADDRESS}
    erdpy data store --key=deployTransaction-testnet --value=${TRANSACTION}

    echo ""
    echo "Multisig contract address: ${ADDRESS}"
}

changeChildContractsOwnership() {
    erdpy --verbose contract call ${SAFE} --recall-nonce --pem=${ALICE} \
    --gas-limit=10000000 --function="ChangeOwnerAddress" \
    --arguments ${MULTISIG} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}

    sleep 10

    erdpy --verbose contract call ${MULTI_TRANSFER} --recall-nonce --pem=${ALICE} \
    --gas-limit=10000000 --function="ChangeOwnerAddress" \
    --arguments ${MULTISIG} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

addMapping() {
    erdpy --verbose contract call ${MULTISIG} --recall-nonce --pem=${ALICE} \
    --gas-limit=40000000 --function="addMapping" \
    --arguments ${ERC20_TOKEN} str:${CHAIN_SPECIFIC_TOKEN} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

addTokenToWhitelist() {
    erdpy --verbose contract call ${MULTISIG} --recall-nonce --pem=${ALICE} \
    --gas-limit=60000000 --function="esdtSafeAddTokenToWhitelist" \
    --arguments str:${CHAIN_SPECIFIC_TOKEN} str:${CHAIN_SPECIFIC_TOKEN_TICKER} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

esdtSafeSetMaxTxBatchSize() {
    erdpy --verbose contract call ${MULTISIG} --recall-nonce --pem=${ALICE} \
    --gas-limit=30000000 --function="esdtSafeSetMaxTxBatchSize" --arguments ${MAX_TX_PER_BATCH} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

esdtSafeSetMaxTxBatchBlockDuration() {
    erdpy --verbose contract call ${MULTISIG} --recall-nonce --pem=${ALICE} \
    --gas-limit=30000000 --function="esdtSafeSetMaxTxBatchBlockDuration" \
    --arguments ${MAX_TX_BLOCK_DURATION_PER_BATCH} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

clearMapping() {
     erdpy --verbose contract call ${MULTISIG} --recall-nonce --pem=${ALICE} \
    --gas-limit=40000000 --function="clearMapping" \
    --arguments ${ERC20_TOKEN} str:${CHAIN_SPECIFIC_TOKEN} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

changeQuorum() {
    erdpy --verbose contract call ${MULTISIG} --recall-nonce --pem=${ALICE} \
    --gas-limit=40000000 --function="changeQuorum" \
    --arguments ${QUORUM} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

pause() {
    erdpy --verbose contract call ${MULTISIG} --recall-nonce --pem=${ALICE} \
    --gas-limit=40000000 --function="pause" \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

pauseEsdtSafe() {
    erdpy --verbose contract call ${MULTISIG} --recall-nonce --pem=${ALICE} \
    --gas-limit=40000000 --function="pauseEsdtSafe" \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

unpause() {
    erdpy --verbose contract call ${MULTISIG} --recall-nonce --pem=${ALICE} \
    --gas-limit=40000000 --function="unpause" \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

unpauseEsdtSafe() {
    erdpy --verbose contract call ${MULTISIG} --recall-nonce --pem=${ALICE} \
    --gas-limit=40000000 --function="unpauseEsdtSafe" \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}
