# 1. deployAggregator
# 2. Call submitAggregatorBatch to set gas price for eth

deployAggregator() {
    erdpy --verbose contract deploy --bytecode=${AGGREGATOR_WASM} --recall-nonce --pem=${ALICE} \
    --gas-limit=100000000 --arguments str:${CHAINSPECIFIC_TOKEN_TO_BE_WHITELISTED} ${ALICE_ADDRESS} 1 2 0 \
    --send --outfile=deploy-price-agregator-testnet.interaction.json --proxy=${PROXY} --chain=${CHAIN_ID} || return

    TRANSACTION=$(erdpy data parse --file="./deploy-price-agregator-testnet.interaction.json" --expression="data['emittedTransactionHash']")
    ADDRESS=$(erdpy data parse --file="./deploy-price-agregator-testnet.interaction.json" --expression="data['contractAddress']")

    erdpy data store --key=address-testnet-safe --value=${ADDRESS}
    erdpy data store --key=deployTransaction-testnet --value=${TRANSACTION}

    echo ""
    echo "Price agregator: ${ADDRESS}"
}

submitAggregatorBatch() {
    erdpy --verbose contract call ${AGGREGATOR_ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=15000000 --function="submitBatch" \
    --arguments str:GWEI str:${CHAINSPECIFIC_TOKEN_TO_BE_WHITELISTED} ${GAS_PRICE_ON_ETH} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID} || return
}