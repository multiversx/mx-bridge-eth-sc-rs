addBoardMember() {
    CHECK_VARIABLES MULTISIG

    read -p "Relayer address: " RELAYER_ADDR
    mxpy --verbose contract call ${MULTISIG} --recall-nonce --pem=${ALICE} \
    --gas-limit=35000000 --function="addBoardMember" --arguments ${RELAYER_ADDR} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

removeBoardMember() {
    CHECK_VARIABLES MULTISIG

    read -p "Relayer address: " RELAYER_ADDR
    mxpy --verbose contract call ${MULTISIG} --recall-nonce --pem=${ALICE} \
    --gas-limit=35000000 --function="removeUser" --arguments ${RELAYER_ADDR} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

unstake() {
    CHECK_VARIABLES MULTISIG RELAYER_REQUIRED_STAKE

    read -p "Relayer address: " RELAYER_ADDR
    MIN_STAKE=$(echo "$RELAYER_REQUIRED_STAKE*10^18" | bc)
    mxpy --verbose contract call ${MULTISIG} --recall-nonce --pem="./walletsRelay/${RELAYER_ADDR}.pem" \
    --gas-limit=35000000 --function="unstake" \
    --arguments ${MIN_STAKE} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

stake() {
    CHECK_VARIABLES MULTISIG RELAYER_REQUIRED_STAKE \
    RELAYER_WALLET0 RELAYER_WALLET1 RELAYER_WALLET2 RELAYER_WALLET3 RELAYER_WALLET4 \
    RELAYER_WALLET5 RELAYER_WALLET6 RELAYER_WALLET7 RELAYER_WALLET8 RELAYER_WALLET9

    MIN_STAKE=$(echo "$RELAYER_REQUIRED_STAKE*10^18" | bc)
    mxpy --verbose contract call ${MULTISIG} --recall-nonce --pem=${RELAYER_WALLET0} \
    --gas-limit=35000000 --function="stake" --value=${MIN_STAKE} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
    echo "---------------------------------------------------------"
    echo "---------------------------------------------------------"
    mxpy --verbose contract call ${MULTISIG} --recall-nonce --pem=${RELAYER_WALLET1} \
    --gas-limit=35000000 --function="stake" --value=${MIN_STAKE} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
    echo "---------------------------------------------------------"
    echo "---------------------------------------------------------"
    mxpy --verbose contract call ${MULTISIG} --recall-nonce --pem=${RELAYER_WALLET2} \
    --gas-limit=35000000 --function="stake" --value=${MIN_STAKE} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
    echo "---------------------------------------------------------"
    echo "---------------------------------------------------------"
    mxpy --verbose contract call ${MULTISIG} --recall-nonce --pem=${RELAYER_WALLET3} \
    --gas-limit=35000000 --function="stake" --value=${MIN_STAKE} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
    echo "---------------------------------------------------------"
    echo "---------------------------------------------------------"
    mxpy --verbose contract call ${MULTISIG} --recall-nonce --pem=${RELAYER_WALLET4} \
    --gas-limit=35000000 --function="stake" --value=${MIN_STAKE} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
    echo "---------------------------------------------------------"
    echo "---------------------------------------------------------"
    mxpy --verbose contract call ${MULTISIG} --recall-nonce --pem=${RELAYER_WALLET5} \
    --gas-limit=35000000 --function="stake" --value=${MIN_STAKE} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
    echo "---------------------------------------------------------"
    echo "---------------------------------------------------------"
    mxpy --verbose contract call ${MULTISIG} --recall-nonce --pem=${RELAYER_WALLET6} \
    --gas-limit=35000000 --function="stake" --value=${MIN_STAKE} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
    echo "---------------------------------------------------------"
    echo "---------------------------------------------------------"
    mxpy --verbose contract call ${MULTISIG} --recall-nonce --pem=${RELAYER_WALLET7} \
    --gas-limit=35000000 --function="stake" --value=${MIN_STAKE} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
    echo "---------------------------------------------------------"
    echo "---------------------------------------------------------"
    mxpy --verbose contract call ${MULTISIG} --recall-nonce --pem=${RELAYER_WALLET8} \
    --gas-limit=35000000 --function="stake" --value=${MIN_STAKE} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
    echo "---------------------------------------------------------"
    echo "---------------------------------------------------------"
    mxpy --verbose contract call ${MULTISIG} --recall-nonce --pem=${RELAYER_WALLET9} \
    --gas-limit=35000000 --function="stake" --value=${MIN_STAKE} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}