deployMultiTransfer() {
    CHECK_VARIABLES MULTI_TRANSFER_WASM

    mxpy --verbose contract deploy --bytecode=${MULTI_TRANSFER_WASM} --recall-nonce --pem=${ALICE} \
    --gas-limit=100000000 --metadata-payable \
    --send --outfile="deploy-multitransfer-testnet.interaction.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return

    ADDRESS=$(mxpy data parse --file="./deploy-multitransfer-testnet.interaction.json" --expression="data['contractAddress']")
    mxpy data store --key=address-testnet-multitransfer --value=${ADDRESS}

    echo ""
    echo "Multi transfer contract address: ${ADDRESS}"
    update-config MULTI_TRANSFER ${ADDRESS}
}

setBridgeProxyContractAddress() {
    CHECK_VARIABLES MULTI_TRANSFER BRIDGE_PROXY

    mxpy --verbose contract call ${MULTI_TRANSFER} --recall-nonce --pem=${ALICE} \
    --gas-limit=60000000 --function="setBridgeProxyContractAddress" \
    --arguments ${BRIDGE_PROXY} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

setWrappingContractAddress() {
    CHECK_VARIABLES MULTISIG BRIDGED_TOKENS_WRAPPER

    mxpy --verbose contract call ${MULTISIG} --recall-nonce --pem=${ALICE} \
    --gas-limit=60000000 --function="multiTransferEsdtSetWrappingContractAddress" \
    --arguments ${BRIDGED_TOKENS_WRAPPER} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}