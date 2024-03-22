#TODO: check & updates upgrade snippets
deploySafeForUpgrade() {
    CHECK_VARIABLES SAFE_WASM AGGREGATOR

    mxpy --verbose contract deploy --bytecode=${SAFE_WASM} --recall-nonce --pem=${ALICE} \
    --gas-limit=150000000 \
    --arguments ${AGGREGATOR} 1 \
    --send --outfile="deploy-safe-upgrade.interaction.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return
}

deployMultiTransferForUpgrade() {
    CHECK_VARIABLES MULTI_TRANSFER_WASM

    mxpy --verbose contract deploy --bytecode=${MULTI_TRANSFER_WASM} --recall-nonce --pem=${ALICE} \
    --gas-limit=100000000 --metadata-payable \
    --send --outfile="deploy-multitransfer-upgrade.interaction.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return
}

deployBridgeProxyForUpgrade() {
    CHECK_VARIABLES PROXY_WASM MULTI_TRANSFER

    mxpy --verbose contract deploy --bytecode=${PROXY_WASM} --recall-nonce --pem=${ALICE} \
    --gas-limit=200000000 \
    --arguments ${MULTI_TRANSFER} \
    --send --outfile="deploy-proxy-upgrade.interaction.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return
}

upgradeSafe() {
    CHECK_VARIABLES MULTISIG SAFE AGGREGATOR
    ADDRESS=$(mxpy data parse --file="./deploy-safe-upgrade.interaction.json" --expression="data['contractAddress']")

    mxpy --verbose contract call ${MULTISIG} --recall-nonce --pem=${ALICE} \
    --gas-limit=400000000 --function="upgradeChildContractFromSource" \
    --arguments ${SAFE} ${ADDRESS} 0 ${AGGREGATOR} 1 \
    --send --outfile="upgrade-safe-child-sc-spam.json" --proxy=${PROXY} --chain=${CHAIN_ID}
}

upgradeMultiTransfer() {
    CHECK_VARIABLES MULTISIG MULTI_TRANSFER
    ADDRESS=$(mxpy data parse --file="./deploy-multitransfer-upgrade.interaction.json" --expression="data['contractAddress']")

    mxpy --verbose contract call ${MULTISIG} --recall-nonce --pem=${ALICE} \
    --gas-limit=400000000 --function="upgradeChildContractFromSource" \
    --arguments ${MULTI_TRANSFER} ${ADDRESS} 0 \
    --send --outfile="upgrade-multitransfer-child-sc-spam.json" --proxy=${PROXY} --chain=${CHAIN_ID}
}

upgradeBridgeProxy() {
    CHECK_VARIABLES MULTISIG BRIDGE_PROXY
    ADDRESS=$(mxpy data parse --file="./deploy-proxy-upgrade.interaction.json" --expression="data['contractAddress']")

    mxpy --verbose contract call ${MULTISIG} --recall-nonce --pem=${ALICE} \
    --gas-limit=400000000 --function="upgradeChildContractFromSource" \
    --arguments ${BRIDGE_PROXY} ${ADDRESS} 0 \
    --send --outfile="upgrade-proxy-child-sc-spam.json" --proxy=${PROXY} --chain=${CHAIN_ID}
}

upgradeMultisig() {
    CHECK_VARIABLES MULTISIG MULTISIG_WASM

    mxpy --verbose contract upgrade ${MULTISIG} --bytecode=${MULTISIG_WASM} --recall-nonce --pem=${ALICE} \
    --gas-limit=200000000 \
    --send --outfile="upgrade-multisig.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return
}