deployAggregator() {
    CHECK_VARIABLES AGGREGATOR_WASM CHAIN_SPECIFIC_TOKEN ALICE_ADDRESS

    mxpy --verbose contract deploy --bytecode=${AGGREGATOR_WASM} --recall-nonce --pem=${ALICE} \
    --gas-limit=100000000 --arguments 1 0 ${ALICE_ADDRESS} \
    --send --outfile=deploy-price-agregator-testnet.interaction.json --proxy=${PROXY} --chain=${CHAIN_ID} || return

    TRANSACTION=$(mxpy data parse --file="./deploy-price-agregator-testnet.interaction.json" --expression="data['emittedTransactionHash']")
    ADDRESS=$(mxpy data parse --file="./deploy-price-agregator-testnet.interaction.json" --expression="data['contractAddress']")

    mxpy data store --key=address-testnet-safe --value=${ADDRESS}
    mxpy data store --key=deployTransaction-testnet --value=${TRANSACTION}

    echo ""
    echo "Price agregator: ${ADDRESS}"
}

submitAggregatorBatch() {
    CHECK_VARIABLES AGGREGATOR CHAIN_SPECIFIC_TOKEN FEE_AMOUNT NR_DECIMALS_CHAIN_SPECIFIC

    FEE=$(echo "scale=0; $FEE_AMOUNT*10^$NR_DECIMALS_CHAIN_SPECIFIC/1" | bc)

    mxpy --verbose contract call ${AGGREGATOR} --recall-nonce --pem=${ALICE} \
    --gas-limit=15000000 --function="submitBatch" \
    --arguments str:GWEI str:${CHAIN_SPECIFIC_TOKEN_TICKER} ${FEE} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID} || return
}

pauseAggregator() {
    CHECK_VARIABLES AGGREGATOR

    mxpy --verbose contract call ${AGGREGATOR} --recall-nonce --pem=${ALICE} \
    --gas-limit=5000000 --function="pause" \
    --send --proxy=${PROXY} --chain=${CHAIN_ID} || return
}

unpauseAggregator() {
    CHECK_VARIABLES AGGREGATOR

    mxpy --verbose contract call ${AGGREGATOR} --recall-nonce --pem=${ALICE} \
    --gas-limit=5000000 --function="unpause" \
    --send --proxy=${PROXY} --chain=${CHAIN_ID} || return
}
