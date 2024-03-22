deployAggregator() {
    CHECK_VARIABLES AGGREGATOR_WASM CHAIN_SPECIFIC_TOKEN ALICE_ADDRESS AGGREGATOR_REQUIRED_STAKE

    MIN_STAKE=$(echo "$AGGREGATOR_REQUIRED_STAKE*10^18" | bc)
    mxpy --verbose contract deploy --bytecode=${AGGREGATOR_WASM} --recall-nonce --pem=${ALICE} \
    --gas-limit=100000000 --arguments str:EGLD ${MIN_STAKE} ${SLASH_AMOUNT} 1 1 ${ALICE_ADDRESS} \
    --send --outfile=deploy-price-agregator-testnet.interaction.json --proxy=${PROXY} --chain=${CHAIN_ID} || return

    TRANSACTION=$(mxpy data parse --file="./deploy-price-agregator-testnet.interaction.json" --expression="data['emittedTransactionHash']")
    ADDRESS=$(mxpy data parse --file="./deploy-price-agregator-testnet.interaction.json" --expression="data['contractAddress']")

    mxpy data store --key=address-testnet-safe --value=${ADDRESS}
    mxpy data store --key=deployTransaction-testnet --value=${TRANSACTION}

    echo ""
    echo "Price agregator: ${ADDRESS}"
    update-config AGGREGATOR ${ADDRESS}
}

stakeAggregator() {
    CHECK_VARIABLES AGGREGATOR AGGREGATOR_REQUIRED_STAKE

    MIN_STAKE=$(echo "$AGGREGATOR_REQUIRED_STAKE*10^18" | bc)
    mxpy --verbose contract call ${AGGREGATOR} --recall-nonce --pem=${ALICE} \
    --value=${MIN_STAKE} \
    --gas-limit=10000000 --function="stake" \
    --send --proxy=${PROXY} --chain=${CHAIN_ID} || return
}

setPairDecimals() {
    CHECK_VARIABLES AGGREGATOR CHAIN_SPECIFIC_TOKEN

    mxpy --verbose contract call ${AGGREGATOR} --recall-nonce --pem=${ALICE} \
    --gas-limit=10000000 --function="setPairDecimals" \
    --arguments str:GWEI str:${CHAIN_SPECIFIC_TOKEN_TICKER} 0 \
    --send --proxy=${PROXY} --chain=${CHAIN_ID} || return
} 

submitAggregatorBatch() {
    CHECK_VARIABLES AGGREGATOR CHAIN_SPECIFIC_TOKEN FEE_AMOUNT NR_DECIMALS_CHAIN_SPECIFIC

    FEE=$(echo "scale=0; $FEE_AMOUNT*10^$NR_DECIMALS_CHAIN_SPECIFIC/1" | bc)
    TIMESTAMP=$(date +%s)
    mxpy --verbose contract call ${AGGREGATOR} --recall-nonce --pem=${ALICE} \
    --gas-limit=15000000 --function="submitBatch" \
    --arguments str:GWEI str:${CHAIN_SPECIFIC_TOKEN_TICKER} ${TIMESTAMP} ${FEE} 0 \
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
