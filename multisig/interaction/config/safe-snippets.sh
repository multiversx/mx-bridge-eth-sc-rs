deploySafe() {
    CHECK_VARIABLES SAFE_WASM MULTI_TRANSFER AGGREGATOR
    
    mxpy --verbose contract deploy --bytecode=${SAFE_WASM} --recall-nonce --pem=${ALICE} \
    --gas-limit=150000000 \
    --arguments ${AGGREGATOR} ${MULTI_TRANSFER} 1 \
    --send --outfile="deploy-safe-testnet.interaction.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return

    TRANSACTION=$(mxpy data parse --file="./deploy-safe-testnet.interaction.json" --expression="data['emittedTransactionHash']")
    ADDRESS=$(mxpy data parse --file="./deploy-safe-testnet.interaction.json" --expression="data['contractAddress']")

    mxpy data store --key=address-testnet-safe --value=${ADDRESS}
    mxpy data store --key=deployTransaction-testnet --value=${TRANSACTION}

    echo ""
    echo "Safe contract address: ${ADDRESS}"
    update-config SAFE ${ADDRESS}
}   

setLocalRolesEsdtSafe() {
    CHECK_VARIABLES ESDT_SYSTEM_SC_ADDRESS CHAIN_SPECIFIC_TOKEN SAFE

    mxpy --verbose contract call ${ESDT_SYSTEM_SC_ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=60000000 --function="setSpecialRole" \
    --arguments str:${CHAIN_SPECIFIC_TOKEN} ${SAFE} str:ESDTRoleLocalBurn str:ESDTRoleLocalMint \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

unsetLocalRolesEsdtSafe() {
    CHECK_VARIABLES ESDT_SYSTEM_SC_ADDRESS CHAIN_SPECIFIC_TOKEN SAFE

    mxpy --verbose contract call ${ESDT_SYSTEM_SC_ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=60000000 --function="unSetSpecialRole" \
    --arguments str:${CHAIN_SPECIFIC_TOKEN} ${SAFE} str:ESDTRoleLocalBurn str:ESDTRoleLocalMint \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

setBridgedTokensWrapperOnEsdtSafe() {
    CHECK_VARIABLES SAFE BRIDGED_TOKENS_WRAPPER

    mxpy --verbose contract call ${SAFE} --recall-nonce --pem=${ALICE} \
    --gas-limit=60000000 --function="setBridgedTokensWrapperAddress" \
    --arguments ${BRIDGED_TOKENS_WRAPPER} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

setSCProxyOnEsdtSafe() {
    CHECK_VARIABLES SAFE BRIDGE_PROXY

    mxpy --verbose contract call ${SAFE} --recall-nonce --pem=${ALICE} \
    --gas-limit=60000000 --function="setBridgeProxyContractAddress" \
    --arguments ${BRIDGE_PROXY} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

deploySafeForUpgrade() {
    CHECK_VARIABLES SAFE_WASM MULTI_TRANSFER AGGREGATOR BRIDGE_PROXY

    mxpy --verbose contract deploy --bytecode=${SAFE_WASM} --recall-nonce --pem=${ALICE} \
    --gas-limit=150000000 \
    --arguments ${AGGREGATOR} ${MULTI_TRANSFER} 1 \
    --send --outfile="deploy-safe-upgrade.interaction.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return

    TRANSACTION=$(mxpy data parse --file="./deploy-safe-upgrade.interaction.json" --expression="data['emittedTransactionHash']")
    ADDRESS=$(mxpy data parse --file="./deploy-safe-upgrade.interaction.json" --expression="data['contractAddress']")

    echo ""
    echo "New safe contract address: ${ADDRESS}"
}

upgradeSafeContract() {
    local NEW_SAFE_ADDR=$(mxpy data parse --file="./deploy-safe-upgrade.interaction.json" --expression="data['contractAddress']")

    mxpy --verbose contract call ${MULTISIG} --recall-nonce --pem=${ALICE} \
    --gas-limit=400000000 --function="upgradeChildContractFromSource" \
    --arguments ${SAFE} ${NEW_SAFE_ADDR} 0x00 \
    ${AGGREGATOR} ${MULTI_TRANSFER} ${BRIDGE_PROXY} 1 \
    --send --outfile="upgrade-safe-child-sc.json" --proxy=${PROXY} --chain=${CHAIN_ID}
}
