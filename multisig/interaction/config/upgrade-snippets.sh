#TODO: check & updates upgrade snippets
deploySafeForUpgrade() {
    getAggregatorAddressHex

    local ESDT_SAFE_ETH_TX_GAS_LIMIT=20000 # gives us 200$ for multiversx->eth

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
    CHECK_VARIABLES RELAYER_ADDR_0 RELAYER_ADDR_1 RELAYER_ADDR_2 RELAYER_ADDR_3 \
    RELAYER_ADDR_4 RELAYER_ADDR_5 RELAYER_ADDR_6 RELAYER_ADDR_7 RELAYER_ADDR_8 \
    RELAYER_ADDR_9 SAFE MULTI_TRANSFER RELAYER_REQUIRED_STAKE SLASH_AMOUNT QUORUM MULTISIG MULTISIG_WASM

    MIN_STAKE=$(echo "$RELAYER_REQUIRED_STAKE*10^18" | bc)
    mxpy --verbose contract upgrade ${MULTISIG} --bytecode=${MULTISIG_WASM} --recall-nonce --pem=${ALICE} \
    --gas-limit=200000000 \
    --arguments ${SAFE} ${MULTI_TRANSFER} \
    ${MIN_STAKE} ${SLASH_AMOUNT} ${QUORUM} \
    --send --outfile="deploy-testnet.interaction.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return

    TRANSACTION=$(mxpy data parse --file="./deploy-testnet.interaction.json" --expression="data['emitted_tx']['hash']")
    ADDRESS=$(mxpy data parse --file="./deploy-testnet.interaction.json" --expression="data['contractAddress']")

    mxpy data store --key=address-testnet-multisig --value=${ADDRESS}
    mxpy data store --key=deployTransaction-testnet --value=${TRANSACTION}

    echo ""
    echo "Multisig contract address: ${ADDRESS}"
}