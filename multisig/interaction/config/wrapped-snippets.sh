# 1. deployBridgedTokensWrapper
# 3. setLocalRolesBridgedTokensWrapper # - keep in mind we need to do this with the token owner
# 4. addWrappedToken
# 5. whitelistToken
# If the SC already exists, skip the first step
# If we want to add another chain, do only the last step

deployBridgedTokensWrapper() {
    CHECK_VARIABLES BRIDGED_TOKENS_WRAPPER_WASM
    
    erdpy --verbose contract deploy --bytecode=${BRIDGED_TOKENS_WRAPPER_WASM} --recall-nonce --pem=${ALICE} \
    --gas-limit=30000000 \
    --send --outfile="deploy-bridged-tokens-wrapper-testnet.interaction.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return

    TRANSACTION=$(erdpy data parse --file="./deploy-bridged-tokens-wrapper-testnet.interaction.json" --expression="data['emittedTransactionHash']")
    ADDRESS=$(erdpy data parse --file="./deploy-bridged-tokens-wrapper-testnet.interaction.json" --expression="data['contractAddress']")

    erdpy data store --key=address-testnet-bridged-tokens-wrapper --value=${ADDRESS}
    erdpy data store --key=deployTransaction-testnet --value=${TRANSACTION}

    echo ""
    echo "Bridged tokens wrapper SC: ${ADDRESS}"
}

setLocalRolesBridgedTokensWrapper() {
    CHECK_VARIABLES ESDT_SYSTEM_SC_ADDRESS UNIVERSAL_TOKEN BRIDGED_TOKENS_WRAPPER
    
    erdpy --verbose contract call ${ESDT_SYSTEM_SC_ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=60000000 --function="setSpecialRole" \
    --arguments str:${UNIVERSAL_TOKEN} ${BRIDGED_TOKENS_WRAPPER} str:ESDTRoleLocalMint str:ESDTRoleLocalBurn\
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

unsetLocalRolesBridgedTokensWrapper() {
    CHECK_VARIABLES ESDT_SYSTEM_SC_ADDRESS UNIVERSAL_TOKEN BRIDGED_TOKENS_WRAPPER
    
    erdpy --verbose contract call ${ESDT_SYSTEM_SC_ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=60000000 --function="unSetSpecialRole" \
    --arguments str:${UNIVERSAL_TOKEN} ${BRIDGED_TOKENS_WRAPPER} str:ESDTRoleLocalMint str:ESDTRoleLocalBurn\
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

addWrappedToken() {
    CHECK_VARIABLES BRIDGED_TOKENS_WRAPPER UNIVERSAL_TOKEN

    erdpy --verbose contract call ${BRIDGED_TOKENS_WRAPPER} --recall-nonce --pem=${ALICE} \
    --gas-limit=6000000 --function="addWrappedToken" \
    --arguments str:${UNIVERSAL_TOKEN} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

removeWrappedToken() {
    CHECK_VARIABLES BRIDGED_TOKENS_WRAPPER UNIVERSAL_TOKEN

    erdpy --verbose contract call ${BRIDGED_TOKENS_WRAPPER} --recall-nonce --pem=${ALICE} \
    --gas-limit=6000000 --function="removeWrappedToken" \
    --arguments str:${UNIVERSAL_TOKEN} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

wrapper-whitelistToken() {
    CHECK_VARIABLES BRIDGED_TOKENS_WRAPPER CHAIN_SPECIFIC_TOKEN UNIVERSAL_TOKEN

    erdpy --verbose contract call ${BRIDGED_TOKENS_WRAPPER} --recall-nonce --pem=${ALICE} \
    --gas-limit=6000000 --function="whitelistToken" \
    --arguments str:${CHAIN_SPECIFIC_TOKEN} str:${UNIVERSAL_TOKEN} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

wrapper-blacklistToken() {
    CHECK_VARIABLES BRIDGED_TOKENS_WRAPPER CHAIN_SPECIFIC_TOKEN UNIVERSAL_TOKEN

    erdpy --verbose contract call ${BRIDGED_TOKENS_WRAPPER} --recall-nonce --pem=${ALICE} \
    --gas-limit=6000000 --function="blacklistToken" \
    --arguments str:${CHAIN_SPECIFIC_TOKEN} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

upgradeBridgedTokensWrapper() {
    CHECK_VARIABLES BRIDGED_TOKENS_WRAPPER BRIDGED_TOKENS_WRAPPER_WASM

    erdpy --verbose contract upgrade ${BRIDGED_TOKENS_WRAPPER} --bytecode=${BRIDGED_TOKENS_WRAPPER_WASM} --recall-nonce --pem=${ALICE} \
    --gas-limit=50000000 --send \
    --outfile="upgrade-bridged-tokens-wrapper.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return 
}