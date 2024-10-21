deployFaucet() {
    CHECK_VARIABLES FAUCET_WASM ALICE

    mxpy --verbose contract deploy --bytecode=${FAUCET_WASM} --recall-nonce --pem=${ALICE} \
    --gas-limit=20000000 \
    --send --outfile=deploy-faucet-testnet.interaction.json --proxy=${PROXY} --chain=${CHAIN_ID} || return

    TRANSACTION=$(mxpy data parse --file="./deploy-faucet-testnet.interaction.json" --expression="data['emittedTransactionHash']")
    ADDRESS=$(mxpy data parse --file="./deploy-faucet-testnet.interaction.json" --expression="data['contractAddress']")

    mxpy data store --key=address-testnet-faucet --value=${ADDRESS}
    mxpy data store --key=deployTransaction-testnet --value=${TRANSACTION}

    echo ""
    echo "Faucet: ${ADDRESS}"
    update-config FAUCET ${ADDRESS}
}

setMintRoleForUniversalToken() {
  CHECK_VARIABLES ALICE ALICE_ADDRESS

  mxpy --verbose contract call ${ESDT_SYSTEM_SC_ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=60000000 --function="setSpecialRole" \
    --arguments str:${UNIVERSAL_TOKEN} ${ALICE_ADDRESS} str:ESDTRoleLocalMint \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

mintAndDeposit() {
  CHECK_VARIABLES ALICE ALICE_ADDRESS FAUCET

  read -p "Amount to mint (without decimals): " AMOUNT_TO_MINT
  VALUE_TO_MINT=$(echo "scale=0; $AMOUNT_TO_MINT*10^$NR_DECIMALS_UNIVERSAL/1" | bc)
  mxpy --verbose contract call ${ALICE_ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=300000 --function="ESDTLocalMint" \
    --arguments str:${UNIVERSAL_TOKEN} ${VALUE_TO_MINT} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}

  sleep 6

  mxpy --verbose contract call ${FAUCET} --recall-nonce --pem=${ALICE} \
    --gas-limit=5000000 --function="ESDTTransfer" \
    --arguments str:${UNIVERSAL_TOKEN} ${VALUE_TO_MINT} str:deposit 100 \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

unSetMintRoleForUniversalToken() {
    CHECK_VARIABLES ALICE ALICE_ADDRESS ESDT_SYSTEM_SC_ADDRESS

    mxpy --verbose contract call ${ESDT_SYSTEM_SC_ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=60000000 --function="unSetSpecialRole" \
    --arguments str:${UNIVERSAL_TOKEN} ${ALICE_ADDRESS} str:ESDTRoleLocalMint \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

deployTestCaller() {
    CHECK_VARIABLES TEST_CALLER_WASM ALICE

    mxpy --verbose contract deploy --bytecode=${TEST_CALLER_WASM} --recall-nonce --pem=${ALICE} \
    --gas-limit=20000000 \
    --send --outfile=deploy-test-caller.interaction.json --proxy=${PROXY} --chain=${CHAIN_ID} || return

    TRANSACTION=$(mxpy data parse --file="./deploy-test-caller.interaction.json" --expression="data['emittedTransactionHash']")
    ADDRESS=$(mxpy data parse --file="./deploy-test-caller.interaction.json" --expression="data['contractAddress']")

    mxpy data store --key=address-test-caller --value=${ADDRESS}
    mxpy data store --key=deployTransaction-testnet --value=${TRANSACTION}

    echo ""
    echo "Test caller: ${ADDRESS}"
    update-config TEST_CALLER ${ADDRESS}
}
