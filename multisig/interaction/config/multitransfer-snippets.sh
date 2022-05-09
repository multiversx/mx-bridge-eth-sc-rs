deployMultiTransfer() {
    erdpy --verbose contract deploy --bytecode=${MULTI_TRANSFER_WASM} --recall-nonce --pem=${ALICE} \
    --gas-limit=100000000 \
    --arguments ${BRIDGED_TOKENS_WRAPPER} --metadata-payable \
    --send --outfile="deploy-multitransfer-testnet.interaction.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return

    ADDRESS=$(erdpy data parse --file="./deploy-multitransfer-testnet.interaction.json" --expression="data['contractAddress']")
    erdpy data store --key=address-testnet-multitransfer --value=${ADDRESS}

    echo ""
    echo "Multi transfer contract address: ${ADDRESS}"
}

setLocalRolesMultiTransferEsdt() {
    erdpy --verbose contract call ${ESDT_SYSTEM_SC_ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=60000000 --function="setSpecialRole" \
    --arguments str:${CHAIN_SPECIFIC_TOKEN} ${MULTISIG} str:ESDTRoleLocalMint \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}