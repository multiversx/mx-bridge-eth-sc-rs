deployMultiTransfer() {
    CHECK_VARIABLES MULTI_TRANSFER_WASM BRIDGED_TOKENS_WRAPPER

    mxpy --verbose contract deploy --bytecode=${MULTI_TRANSFER_WASM} --recall-nonce --pem=${ALICE} \
    --gas-limit=100000000 \
    --arguments ${BRIDGED_TOKENS_WRAPPER} --metadata-payable \
    --send --outfile="deploy-multitransfer-testnet.interaction.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return

    ADDRESS=$(mxpy data parse --file="./deploy-multitransfer-testnet.interaction.json" --expression="data['contractAddress']")
    mxpy data store --key=address-testnet-multitransfer --value=${ADDRESS}

    echo ""
    echo "Multi transfer contract address: ${ADDRESS}"
}

setLocalRolesMultiTransferEsdt() {
    CHECK_VARIABLES ESDT_SYSTEM_SC_ADDRESS CHAIN_SPECIFIC_TOKEN MULTI_TRANSFER

    mxpy --verbose contract call ${ESDT_SYSTEM_SC_ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=60000000 --function="setSpecialRole" \
    --arguments str:${CHAIN_SPECIFIC_TOKEN} ${MULTI_TRANSFER} str:ESDTRoleLocalMint \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

unsetLocalRolesMultiTransferEsdt() {
    CHECK_VARIABLES ESDT_SYSTEM_SC_ADDRESS CHAIN_SPECIFIC_TOKEN MULTI_TRANSFER

    mxpy --verbose contract call ${ESDT_SYSTEM_SC_ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=60000000 --function="unSetSpecialRole" \
    --arguments str:${CHAIN_SPECIFIC_TOKEN} ${MULTI_TRANSFER} str:ESDTRoleLocalMint \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}