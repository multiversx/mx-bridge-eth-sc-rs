#TODO: check & updates upgrade snippets
deploySafeForUpgrade() {
    getAggregatorAddressHex

    local ESDT_SAFE_ETH_TX_GAS_LIMIT=20000 # gives us 200$ for elrond->eth

    mxpy --verbose contract deploy --project=${PROJECT_SAFE} --recall-nonce --pem=${ALICE} \
    --gas-limit=150000000 \
    --arguments 0x${AGGREGATOR_ADDRESS_HEX} ${ESDT_SAFE_ETH_TX_GAS_LIMIT} \
    --send --outfile="deploy-safe-upgrade.interaction.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return

    ADDRESS=$(mxpy data parse --file="./deploy-safe-upgrade.interaction.json" --expression="data['contractAddress']")

    echo ""
    echo "Safe contract address: ${ADDRESS}"
}


upgradeSafeContract() {
    getEsdtSafeAddressHex
    getAggregatorAddressHex
    local ESDT_SAFE_ETH_TX_GAS_LIMIT=20000

    local NEW_SAFE_BECH=$(mxpy data parse --file="./deploy-safe-upgrade.interaction.json" --expression="data['contractAddress']")
    local NEW_SAFE_ADDR=$(mxpy wallet bech32 --decode $NEW_SAFE_BECH)



    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=400000000 --function="upgradeChildContractFromSource" \
    --arguments 0x${ESDT_SAFE_ADDRESS_HEX} 0x${NEW_SAFE_ADDR} 0x00 \
    0x${AGGREGATOR_ADDRESS_HEX} ${ESDT_SAFE_ETH_TX_GAS_LIMIT} \
    --send --outfile="upgradesafe-child-sc-spam.json" --proxy=${PROXY} --chain=${CHAIN_ID}
}

upgrade() {
    mxpy --verbose contract upgrade ${ADDRESS} --project=${PROJECT} --recall-nonce --pem=${ALICE} \
    --gas-limit=100000000 --send --outfile="upgrade.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return
}

upgradeMultisig() {
    getMultiTransferEsdtAddressHex
    getEsdtSafeAddressHex
    getMultiTransferEsdtAddressHex

    local SLASH_AMOUNT=0x00 # 0
    MIN_STAKE=$(echo "$RELAYER_REQUIRED_STAKE*10^18" | bc)
    mxpy --verbose contract upgrade ${ADDRESS} --bytecode=../output/multisig.wasm --recall-nonce --pem=${ALICE} \
    --arguments 0x${ESDT_SAFE_ADDRESS_HEX} 0x${MULTI_TRANSFER_ESDT_ADDRESS_HEX} \
    ${local} ${SLASH_AMOUNT} 0x07 \
    --gas-limit=200000000 --send --outfile="upgrade-multisig.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return
    
}