deployAggregator() {
    CHECK_VARIABLES AGGREGATOR_WASM CHAIN_SPECIFIC_TOKEN ORACLE_ADDR_0 ORACLE_ADDR_1 ORACLE_ADDR_2

    STAKE=$(echo "$ORACLE_REQUIRED_STAKE*10^18" | bc)
    mxpy --verbose contract deploy --bytecode=${AGGREGATOR_WASM} --recall-nonce --pem=${ALICE} \
    --gas-limit=100000000 --arguments str:EGLD ${STAKE} 1 2 3 \
    ${ORACLE_ADDR_0} ${ORACLE_ADDR_1} ${ORACLE_ADDR_2} \
    --send --outfile=deploy-price-agregator-testnet.interaction.json --proxy=${PROXY} --chain=${CHAIN_ID} || return

    TRANSACTION=$(mxpy data parse --file="./deploy-price-agregator-testnet.interaction.json" --expression="data['emittedTransactionHash']")
    ADDRESS=$(mxpy data parse --file="./deploy-price-agregator-testnet.interaction.json" --expression="data['contractAddress']")

    mxpy data store --key=address-testnet-safe --value=${ADDRESS}
    mxpy data store --key=deployTransaction-testnet --value=${TRANSACTION}

    echo ""
    echo "Price agregator: ${ADDRESS}"
    update-config AGGREGATOR ${ADDRESS}
}

stakeOracles() {
    CHECK_VARIABLES AGGREGATOR

    STAKE=$(echo "$ORACLE_REQUIRED_STAKE*10^18" | bc)
    echo "---------------------------------------------------------"
    mxpy --verbose contract call ${AGGREGATOR} --recall-nonce --pem=${ORACLE_WALLET0} \
    --gas-limit=35000000 --function="stake" --value=${STAKE} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
    echo "---------------------------------------------------------"
    echo "---------------------------------------------------------"
    mxpy --verbose contract call ${AGGREGATOR} --recall-nonce --pem=${ORACLE_WALLET1} \
    --gas-limit=35000000 --function="stake" --value=${STAKE} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
    echo "---------------------------------------------------------"
    echo "---------------------------------------------------------"
    mxpy --verbose contract call ${AGGREGATOR} --recall-nonce --pem=${ORACLE_WALLET2} \
    --gas-limit=35000000 --function="stake" --value=${STAKE} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
    echo "---------------------------------------------------------"
}

submitAggregatorBatch() {
    CHECK_VARIABLES AGGREGATOR CHAIN_SPECIFIC_TOKEN FEE_AMOUNT NR_DECIMALS_CHAIN_SPECIFIC

    FEE=$(echo "scale=0; $FEE_AMOUNT*10^$NR_DECIMALS_CHAIN_SPECIFIC/1" | bc)

    CURRENT_TIME=$(date +%s)
    mxpy --verbose contract call ${AGGREGATOR} --recall-nonce --pem=${ORACLE_WALLET0} \
    --gas-limit=15000000 --function="submitBatch" \
    --arguments str:GWEI str:${CHAIN_SPECIFIC_TOKEN_TICKER} ${CURRENT_TIME} ${FEE} 0 \
    --send --proxy=${PROXY} --chain=${CHAIN_ID} || return

    CURRENT_TIME=$(date +%s)
    mxpy --verbose contract call ${AGGREGATOR} --recall-nonce --pem=${ORACLE_WALLET1} \
    --gas-limit=15000000 --function="submitBatch" \
    --arguments str:GWEI str:${CHAIN_SPECIFIC_TOKEN_TICKER} ${CURRENT_TIME} ${FEE} 0 \
    --send --proxy=${PROXY} --chain=${CHAIN_ID} || return

    CURRENT_TIME=$(date +%s)
    mxpy --verbose contract call ${AGGREGATOR} --recall-nonce --pem=${ORACLE_WALLET2} \
    --gas-limit=15000000 --function="submitBatch" \
    --arguments str:GWEI str:${CHAIN_SPECIFIC_TOKEN_TICKER} ${CURRENT_TIME} ${FEE} 0 \
    --send --proxy=${PROXY} --chain=${CHAIN_ID} || return
}

setPairDecimals() {
    CHECK_VARIABLES AGGREGATOR

    mxpy --verbose contract call ${AGGREGATOR} --recall-nonce --pem=${ALICE} \
        --gas-limit=15000000 --function="setPairDecimals" \
        --arguments str:GWEI str:${CHAIN_SPECIFIC_TOKEN_TICKER} 0 \
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

aggregator-upgrade() {
    CHECK_VARIABLES AGGREGATOR AGGREGATOR_WASM

    mxpy --verbose contract upgrade ${AGGREGATOR} --bytecode=${AGGREGATOR_WASM} --recall-nonce --pem=${ALICE} \
    --gas-limit=100000000 --send \
    --outfile="upgrade-aggregator.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return
}
