deploySafe() {
    erdpy --verbose contract deploy --bytecode=${SAFE_WASM} --recall-nonce --pem=${ALICE} \
    --gas-limit=150000000 \
    --arguments ${AGGREGATOR_ADDRESS} ${ESDT_SAFE_ETH_TX_GAS_LIMIT} \
    --send --outfile="deploy-safe-testnet.interaction.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return

    TRANSACTION=$(erdpy data parse --file="./deploy-safe-testnet.interaction.json" --expression="data['emittedTransactionHash']")
    ADDRESS=$(erdpy data parse --file="./deploy-safe-testnet.interaction.json" --expression="data['contractAddress']")

    erdpy data store --key=address-testnet-safe --value=${ADDRESS}
    erdpy data store --key=deployTransaction-testnet --value=${TRANSACTION}

    echo ""
    echo "Safe contract address: ${ADDRESS}"
}   

setLocalRolesEsdtSafe() {
    erdpy --verbose contract call ${ESDT_SYSTEM_SC_ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=60000000 --function="setSpecialRole" \
    --arguments ${CHAINSPECIFIC_TOKEN} ${SAFE_ADDRESS} str:ESDTRoleLocalBurn \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}