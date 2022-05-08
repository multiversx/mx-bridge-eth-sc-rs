ESDT_ISSUE_COST=50000000000000000

issueUniversalToken() {
    erdpy --verbose contract call ${ESDT_SYSTEM_SC_ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=60000000 --value=${ESDT_ISSUE_COST} --function="issue" \
    --arguments str:${UNIVERSAL_TOKEN_DISPLAY_NAME} str:${UNIVERSAL_TOKEN_TICKER} \
    0 ${NR_DECIMALS} str:canAddSpecialRoles str:true \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

issueChainSpecificToken() {
    VALUE_TO_MINT=$(echo "$UNIVERSAL_TOKENS_ALREADY_MINTED*10^$NR_DECIMALS" | bc)

    erdpy --verbose contract call ${ESDT_SYSTEM_SC_ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=60000000 --value=${ESDT_ISSUE_COST} --function="issue" \
    --arguments str:${CHAIN_SPECIFIC_TOKEN_DISPLAY_NAME} str:${CHAIN_SPECIFIC_TOKEN_TICKER} \
    ${VALUE_TO_MINT} ${NR_DECIMALS} str:canAddSpecialRoles str:true \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

transferToSC() {
    VALUE_TO_MINT=$(echo "$UNIVERSAL_TOKENS_ALREADY_MINTED*10^$NR_DECIMALS" | bc)

    erdpy --verbose contract call ${BRIDGED_TOKENS_WRAPPER} --recall-nonce --pem=${ALICE} \
    --gas-limit=500000 --function="ESDTTransfer" \
    --arguments str:${CHAIN_SPECIFIC_TOKEN_TICKER} ${VALUE_TO_MINT} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}