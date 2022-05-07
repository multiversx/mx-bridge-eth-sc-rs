deployMultiTransfer() {

    erdpy --verbose contract deploy --project=${PROJECT_MULTI_TRANSFER} --recall-nonce --pem=${ALICE} \
    --gas-limit=100000000 \
    --arguments ${bridged_tokens_wrapper_address} --metadata-payable \
    --send --outfile="deploy-multitransfer-testnet.interaction.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return

    ADDRESS=$(erdpy data parse --file="./deploy-multitransfer-testnet.interaction.json" --expression="data['contractAddress']")
    erdpy data store --key=address-testnet-multitransfer --value=${ADDRESS}

    echo ""
    echo "Multi transfer contract address: ${ADDRESS}"
}