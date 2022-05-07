# 1. deployBridgedTokensWrapper
# 3. setLocalRolesBridgedTokensWrapper # - keep in mind we need to do this with the token owner
# 4. addWrappedToken
# 5. whitelistToken
# If the SC already exists, skip the first step
# If we want to add another chain, do only the last step

PROJECT_BRIDGED_TOKENS_WRAPPER="../../bridged-tokens-wrapper/"
ALICE="./wallets/alice.pem"
PROXY=https://testnet-gateway.elrond.com
CHAIN_ID=T

ESDT_SYSTEM_SC_ADDRESS=erd1qqqqqqqqqqqqqqqpqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzllls8a5w6u
bridged_tokens_wrapper_address=erd1qqqqqqqqqqqqqpgqyugrkjyqqkeg5r4hdyg3rucstwh0ydr3d8sswmw0te

deployBridgedTokensWrapper() {
    erdpy --verbose contract deploy --project=${PROJECT_BRIDGED_TOKENS_WRAPPER} --recall-nonce --pem=${ALICE} \
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
    getBridgedTokensWrapperAddress
    read -p "Universal token to be whitelisted: " UNIVERSAL_TOKEN_TO_BE_WHITELISTED

    erdpy --verbose contract call ${ESDT_SYSTEM_SC_ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=60000000 --function="setSpecialRole" \
    --arguments str:${UNIVERSAL_TOKEN_TO_BE_WHITELISTED} ${bridged_tokens_wrapper_address} str:ESDTRoleLocalMint str:ESDTRoleLocalBurn\
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

addWrappedToken() {
    getBridgedTokensWrapperAddress
    read -p "Universal token to be whitelisted: " UNIVERSAL_TOKEN_TO_BE_WHITELISTED

    erdpy --verbose contract call ${bridged_tokens_wrapper_ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=6000000 --function="addWrappedToken" \
    --arguments ${UNIVERSAL_TOKEN_TO_BE_WHITELISTED} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

whitelistToken() {
    getBridgedTokensWrapperAddress
    read -p "Universal token to be whitelisted: " UNIVERSAL_TOKEN_TO_BE_WHITELISTED
    read -p "ChainSpecific token to be whitelisted: " CHAINSPECIFIC_TOKEN_TO_BE_WHITELISTED
    erdpy --verbose contract call ${bridged_tokens_wrapper_ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=6000000 --function="whitelistToken" \
    --arguments str:${CHAINSPECIFIC_TOKEN_TO_BE_WHITELISTED} str:${UNIVERSAL_TOKEN_TO_BE_WHITELISTED} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}