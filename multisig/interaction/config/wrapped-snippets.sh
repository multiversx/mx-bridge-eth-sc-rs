# 1. deployBridgedTokensWrapper
# 3. setLocalRolesBridgedTokensWrapper # - keep in mind we need to do this with the token owner
# 4. addWrappedToken
# 5. whitelistToken
# If the SC already exists, skip the first step
# If we want to add another chain, do only the last step

deployBridgedTokensWrapper() {
    erdpy --verbose contract deploy --bytecode=${BRIDGED_TOKENS_WRAPPER_WASM} --recall-nonce --pem=${ALICE} \
    --gas-limit=50000000 \
    --send --outfile="deploy-bridged-tokens-wrapper-testnet.interaction.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return

    TRANSACTION=$(erdpy data parse --file="./deploy-bridged-tokens-wrapper-testnet.interaction.json" --expression="data['emittedTransactionHash']")
    ADDRESS=$(erdpy data parse --file="./deploy-bridged-tokens-wrapper-testnet.interaction.json" --expression="data['contractAddress']")

    erdpy data store --key=address-testnet-bridged-tokens-wrapper --value=${ADDRESS}
    erdpy data store --key=deployTransaction-testnet --value=${TRANSACTION}

    echo ""
    echo "Bridged tokens wrapper SC: ${ADDRESS}"
}

setLocalRolesBridgedTokensWrapper() {
    erdpy --verbose contract call ${ESDT_SYSTEM_SC_ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=60000000 --function="setSpecialRole" \
    --arguments str:${UNIVERSAL_TOKEN} ${BRIDGED_TOKENS_WRAPPER} str:ESDTRoleLocalMint str:ESDTRoleLocalBurn\
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

addWrappedToken() {
    erdpy --verbose contract call ${BRIDGED_TOKENS_WRAPPER} --recall-nonce --pem=${ALICE} \
    --gas-limit=6000000 --function="addWrappedToken" \
    --arguments str:${UNIVERSAL_TOKEN} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

wrapper-whitelistToken() {
    erdpy --verbose contract call ${BRIDGED_TOKENS_WRAPPER} --recall-nonce --pem=${ALICE} \
    --gas-limit=6000000 --function="whitelistToken" \
    --arguments str:${CHAIN_SPECIFIC_TOKEN} str:${UNIVERSAL_TOKEN} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}