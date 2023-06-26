deploySafe() {
    CHECK_VARIABLES SAFE_WASM AGGREGATOR
    
    mxpy --verbose contract deploy --bytecode=${SAFE_WASM} --recall-nonce --pem=${ALICE} \
    --gas-limit=150000000 \
    --arguments ${AGGREGATOR} 1 \
    --send --outfile="deploy-safe-testnet.interaction.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return

    TRANSACTION=$(mxpy data parse --file="./deploy-safe-testnet.interaction.json" --expression="data['emittedTransactionHash']")
    ADDRESS=$(mxpy data parse --file="./deploy-safe-testnet.interaction.json" --expression="data['contractAddress']")

    mxpy data store --key=address-testnet-safe --value=${ADDRESS}
    mxpy data store --key=deployTransaction-testnet --value=${TRANSACTION}

    echo ""
    echo "Safe contract address: ${ADDRESS}"
}   

setLocalRolesEsdtSafe() {
    CHECK_VARIABLES ESDT_SYSTEM_SC_ADDRESS CHAIN_SPECIFIC_TOKEN SAFE

    mxpy --verbose contract call ${ESDT_SYSTEM_SC_ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=60000000 --function="setSpecialRole" \
    --arguments str:${CHAIN_SPECIFIC_TOKEN} ${SAFE} str:ESDTRoleLocalBurn \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

unsetLocalRolesEsdtSafe() {
    CHECK_VARIABLES ESDT_SYSTEM_SC_ADDRESS CHAIN_SPECIFIC_TOKEN SAFE

    mxpy --verbose contract call ${ESDT_SYSTEM_SC_ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=60000000 --function="unSetSpecialRole" \
    --arguments str:${CHAIN_SPECIFIC_TOKEN} ${SAFE} str:ESDTRoleLocalBurn \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}