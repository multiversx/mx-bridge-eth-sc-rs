deployMultisigMainnetV3() {
    CHECK_VARIABLES RELAYER_ADDR_0 RELAYER_ADDR_1 RELAYER_ADDR_2 RELAYER_ADDR_3 \
    RELAYER_ADDR_4 RELAYER_ADDR_5 RELAYER_ADDR_6 RELAYER_ADDR_7 RELAYER_ADDR_8 \
    RELAYER_ADDR_9 SAFE MULTI_TRANSFER BRIDGE_PROXY RELAYER_REQUIRED_STAKE SLASH_AMOUNT QUORUM MULTISIG_WASM

    MIN_STAKE=$(echo "$RELAYER_REQUIRED_STAKE*10^18" | bc)
    mxpy contract deploy --bytecode=${MULTISIG_WASM} --recall-nonce "${MXPY_SIGN[@]}" \
    --gas-limit=200000000 \
    --arguments ${SAFE} ${MULTI_TRANSFER} ${BRIDGE_PROXY} \
    ${MIN_STAKE} ${SLASH_AMOUNT} ${QUORUM} \
    ${RELAYER_ADDR_0} ${RELAYER_ADDR_1} ${RELAYER_ADDR_2} ${RELAYER_ADDR_3} \
    ${RELAYER_ADDR_4} ${RELAYER_ADDR_5} ${RELAYER_ADDR_6} ${RELAYER_ADDR_7} \
    ${RELAYER_ADDR_8} ${RELAYER_ADDR_9} \
    --send --outfile="deploy-testnet.interaction.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return

    TRANSACTION=$(mxpy data parse --file="./deploy-testnet.interaction.json" --expression="data['emitted_tx']['hash']")
    ADDRESS=$(mxpy data parse --file="./deploy-testnet.interaction.json" --expression="data['contractAddress']")

    mxpy data store --key=address-testnet-multisig --value=${ADDRESS}
    mxpy data store --key=deployTransaction-testnet --value=${TRANSACTION}

    echo ""
    echo "Multisig contract address: ${ADDRESS}"
    update-config MULTISIG ${ADDRESS}
}