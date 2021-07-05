# Alice will be the owner of the multisig
# Bob will be the single board member
# Quorum size will be 1

ALICE="/home/elrond/elrond-sdk/erdpy/testnet/wallets/users/alice.pem"
BOB="/home/elrond/elrond-sdk-erdpy/erdpy/testnet/wallets/users/bob.pem"
CAROL="/home/elrond/elrond-sdk-erdpy/erdpy/testnet/wallets/users/carol.pem"

ADDRESS=$(erdpy data load --key=address-testnet-multisig)
DEPLOY_TRANSACTION=$(erdpy data load --key=deployTransaction-testnet)
PROXY=https://testnet-gateway.elrond.com
CHAIN_ID=T

RELAYER_REQUIRED_STAKE=0x03e8 # 1000
ESDT_ISSUE_COST=0x4563918244f40000 # 5 eGLD
ESDT_ISSUE_COST_DECIMAL=5000000000000000000
BOB_ADDRESS=0x8049d639e5a6980d1cd2392abcce41029cda74a1563523a202f09641cc2618f8

ESDT_SYSTEM_SC_ADDRESS=erd1qqqqqqqqqqqqqqqpqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzllls8a5w6u

# Setup and aggregator first, then put its address hex-encoded in this variable
AGGREGATOR_ADDRESS=0xb0d1c728af35de1ff2dab61d960bab6c756e875d73dac06bdcd59cc3790ed4bc

#########################################################################
################## Update after issueing the tokens #####################
#########################################################################
WRAPPED_EGLD_TOKEN_ID=0x
WRAPPED_ETH_TOKEN_ID=0x

deploy() {
    local SLASH_AMOUNT=0x01f4 # 500

    erdpy --verbose contract deploy --project=${PROJECT} --recall-nonce --pem=${ALICE} \
    --gas-limit=200000000 \
    --arguments ${RELAYER_REQUIRED_STAKE} ${SLASH_AMOUNT} 0x01 ${BOB_ADDRESS} \
    --send --outfile="deploy-testnet.interaction.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return

    TRANSACTION=$(erdpy data parse --file="deploy-testnet.interaction.json" --expression="data['emitted_tx']['hash']")
    ADDRESS=$(erdpy data parse --file="deploy-testnet.interaction.json" --expression="data['emitted_tx']['address']")

    erdpy data store --key=address-testnet-multisig --value=${ADDRESS}
    erdpy data store --key=deployTransaction-testnet --value=${TRANSACTION}

    echo ""
    echo "Smart contract address: ${ADDRESS}"
}

upgrade() {
    erdpy --verbose contract upgrade ${ADDRESS} --project=${PROJECT} --recall-nonce --pem=${ALICE} \
    --gas-limit=100000000 --send --outfile="upgrade.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return
}

deployChildContracts() {
    local EGLD_ESDT_SWAP_CODE=0x"$(xxd -p ../egld-esdt-swap/output/egld-esdt-swap.wasm | tr -d '\n')"
    local ESDT_SAFE_CODE=0x"$(xxd -p ../esdt-safe/output/esdt-safe.wasm | tr -d '\n')"
    local MULTI_TRANSFER_ESDT_CODE=0x"$(xxd -p ../multi-transfer-esdt/output/multi-transfer-esdt.wasm | tr -d '\n')"
    local ETHEREUM_FEE_PREPAY_CODE=0x"$(xxd -p ../ethereum-fee-prepay/output/ethereum-fee-prepay.wasm | tr -d '\n')"

    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=400000000 --function="deployChildContracts" \
    --arguments ${EGLD_ESDT_SWAP_CODE} ${MULTI_TRANSFER_ESDT_CODE} ${ETHEREUM_FEE_PREPAY_CODE} ${ESDT_SAFE_CODE} ${AGGREGATOR_ADDRESS} ${WRAPPED_EGLD_TOKEN_ID} ${WRAPPED_ETH_TOKEN_ID} \
    --send --outfile="deploy-child-sc-spam.json" --proxy=${PROXY} --chain=${CHAIN_ID}

    sleep 10

    # finish setup
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=50000000 --function="finishSetup" \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

stake() {
    local RELAYER_REQUIRED_STAKE_DECIMAL=1000

    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${BOB} \
    --gas-limit=25000000 --function="stake" --value=${RELAYER_REQUIRED_STAKE_DECIMAL} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

unstake() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${BOB} \
    --gas-limit=25000000 --function="unstake" \
    --arguments ${RELAYER_REQUIRED_STAKE} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

# Issue Tokens

issueWrappedEgld() {
    local TOKEN_DISPLAY_NAME=0x5772617070656445676c64  # "WrappedEgld"
    local TOKEN_TICKER=0x5745474c44  # "WEGLD"
    local INITIAL_SUPPLY=0x01 # 1
    local NR_DECIMALS=0x12 # 18
    local CAN_ADD_SPECIAL_ROLES=0x63616e4164645370656369616c526f6c6573 # "canAddSpecialRoles"
    local TRUE=0x74727565 # "true"

    erdpy --verbose contract call ${ESDT_SYSTEM_SC_ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=60000000 --value=5000000000000000000 --function="issue" \
    --arguments ${TOKEN_DISPLAY_NAME} ${TOKEN_TICKER} ${INITIAL_SUPPLY} ${NR_DECIMALS} ${CAN_ADD_SPECIAL_ROLES} ${TRUE} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

issueWrappedEth() {
    local TOKEN_DISPLAY_NAME=0x57726170706564457468  # "WrappedEth"
    local TOKEN_TICKER=0x57455448  # "WETH"
    local INITIAL_SUPPLY=0x01 # 1
    local NR_DECIMALS=0x12 # 18
    local CAN_ADD_SPECIAL_ROLES=0x63616e4164645370656369616c526f6c6573 # "canAddSpecialRoles"
    local TRUE=0x74727565 # "true"

    erdpy --verbose contract call ${ESDT_SYSTEM_SC_ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=60000000 --value=5000000000000000000 --function="issue" \
    --arguments ${TOKEN_DISPLAY_NAME} ${TOKEN_TICKER} ${INITIAL_SUPPLY} ${NR_DECIMALS} ${CAN_ADD_SPECIAL_ROLES} ${TRUE} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

# Set Local Roles

setLocalRolesEgldEsdtSwap() {
    getEgldEsdtSwapAddress
    bech32ToHex ${EGLD_ESDT_SWAP_ADDRESS}

    local LOCAL_MINT_ROLE=0x45534454526f6c654c6f63616c4d696e74 # "ESDTRoleLocalMint"
    local LOCAL_BURN_ROLE=0x45534454526f6c654c6f63616c4275726e # "ESDTRoleLocalBurn"

    erdpy --verbose contract call ${ESDT_SYSTEM_SC_ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=60000000 --function="setSpecialRole" \
    --arguments ${WRAPPED_EGLD_TOKEN_ID} ${ADDRESS_HEX} ${LOCAL_MINT_ROLE} ${LOCAL_BURN_ROLE} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

# Note: increase sleep time if needed
setLocalRolesEsdtSafe() {
    getEsdtSafeAddress
    bech32ToHex ${ESDT_SAFE_ADDRESS}

    local LOCAL_BURN_ROLE=0x45534454526f6c654c6f63616c4275726e # "ESDTRoleLocalBurn"

    # set roles for WrappedEgld
    erdpy --verbose contract call ${ESDT_SYSTEM_SC_ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=60000000 --function="setSpecialRole" \
    --arguments ${WRAPPED_EGLD_TOKEN_ID} ${ADDRESS_HEX} ${LOCAL_BURN_ROLE} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}

    sleep 10

    # set roles for WrappedEth
    erdpy --verbose contract call ${ESDT_SYSTEM_SC_ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=60000000 --function="setSpecialRole" \
    --arguments ${WRAPPED_ETH_TOKEN_ID} ${ADDRESS_HEX} ${LOCAL_BURN_ROLE} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

# Note: increase sleep time if needed
setLocalRolesMultiTransferEsdt() {
    getMultiTransferEsdtAddress
    bech32ToHex ${MULTI_TRANSFER_ESDT_ADDRESS}

    local LOCAL_MINT_ROLE=0x45534454526f6c654c6f63616c4d696e74 # "ESDTRoleLocalMint"

    # set roles for WrappedEgld
    erdpy --verbose contract call ${ESDT_SYSTEM_SC_ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=60000000 --function="setSpecialRole" \
    --arguments ${WRAPPED_EGLD_TOKEN_ID} ${ADDRESS_HEX} ${LOCAL_MINT_ROLE} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}

    sleep 10

    # set roles for WrappedEth
    erdpy --verbose contract call ${ESDT_SYSTEM_SC_ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=60000000 --function="setSpecialRole" \
    --arguments ${WRAPPED_ETH_TOKEN_ID} ${ADDRESS_HEX} ${LOCAL_MINT_ROLE} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

# MultiTransferEsdtCalls

transferEsdt() {
    local BATCH_ID = 0x01
    local DEST = ${CAROL_ADDRESS}
    local TOKEN_ID = ${WRAPPED_ETH_TOKEN_ID}
    local AMOUNT = 0x0A

    # Bob proposes action
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${BOB} \
    --gas-limit=25000000 --function="proposeMultiTransferEsdtBatch" \
    --arguments ${BATCH_ID} ${DEST} ${TOKEN_ID} ${AMOUNT} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}

    sleep 10

    # Bob signs the action
    getActionLastIndex
    bobSign
    sleep 10

    # Bob executes the action
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${BOB} \
    --gas-limit=100000000 --function="performAction" \
    --arguments ${ACTION_INDEX} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

getNextTransactionBatch() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${BOB} \
    --gas-limit=25000000 --function="getNextTransactionBatch" \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

setTransactionExecuted() {
    local RELAYER_REWARD_ADDRESS = ${BOB_ADDRESS}
    local TX_STATUS = 0x03

    # Bob proposes action
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${BOB} \
    --gas-limit=25000000 --function="proposeEsdtSafeSetCurrentTransactionBatchStatus" \
    --arguments ${RELAYER_REWARD_ADDRESS} ${TX_STATUS} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}

    # Bob signs the action
    getActionLastIndex
    bobSign
    sleep 10

    # Bob executes the action
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${BOB} \
    --gas-limit=100000000 --function="performAction" \
    --arguments ${ACTION_INDEX} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

setTransactionRejected() {
    local RELAYER_REWARD_ADDRESS = ${BOB_ADDRESS}
    local TX_STATUS = 0x04

    # Bob proposes action
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${BOB} \
    --gas-limit=25000000 --function="proposeEsdtSafeSetCurrentTransactionBatchStatus" \
    --arguments ${RELAYER_REWARD_ADDRESS} ${TX_STATUS} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}

    # Bob signs the action
    getActionLastIndex
    bobSign
    sleep 10

    # Bob executes the action
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${BOB} \
    --gas-limit=100000000 --function="performAction" \
    --arguments ${ACTION_INDEX} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

addBoardMember() {
    # Bob proposes action
    #erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${BOB} --gas-limit=100000000 --function="proposeAddBoardMember" --arguments 0xb0d1c728af35de1ff2dab61d960bab6c756e875d73dac06bdcd59cc3790ed4bc --send --proxy=${PROXY} --chain=${CHAIN_ID}

    # Bob signs the action
    #getActionLastIndex
    #bobSign
    #sleep 10

    # Bob executes the action
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${BOB} --gas-limit=100000000 --function="performAction" --arguments 0x02 --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

# views

getEgldEsdtSwapAddress() {
    local QUERY_OUTPUT=$(erdpy --verbose contract query ${ADDRESS} --function="getEgldEsdtSwapAddress" --proxy=${PROXY})
    parseQueryOutput
    parsedAddressToBech32

    EGLD_ESDT_SWAP_ADDRESS=${ADDRESS_BECH32}
    echo "EgldEsdtSwap address: ${EGLD_ESDT_SWAP_ADDRESS}"
}

getEsdtSafeAddress() {
    local QUERY_OUTPUT=$(erdpy --verbose contract query ${ADDRESS} --function="getEsdtSafeAddress" --proxy=${PROXY})
    parseQueryOutput
    parsedAddressToBech32

    ESDT_SAFE_ADDRESS=${ADDRESS_BECH32}
    echo "EsdtSafe address: ${ESDT_SAFE_ADDRESS}"
}

getMultiTransferEsdtAddress() {
    local QUERY_OUTPUT=$(erdpy --verbose contract query ${ADDRESS} --function="getMultiTransferEsdtAddress" --proxy=${PROXY})
    parseQueryOutput
    parsedAddressToBech32

    MULTI_TRANSFER_ESDT_ADDRESS=${ADDRESS_BECH32}
    echo "MultiTransferEsdt address: ${MULTI_TRANSFER_ESDT_ADDRESS}"
}

getEthereumFeePrepayAddress() {
    local QUERY_OUTPUT=$(erdpy --verbose contract query ${ADDRESS} --function="getEthereumFeePrepayAddress" --proxy=${PROXY})
    parseQueryOutput
    parsedAddressToBech32

    ETHEREUM_FEE_PREPAY_ADDRESS=${ADDRESS_BECH32}
    echo "EthereumFeePrepay address: ${ETHEREUM_FEE_PREPAY_ADDRESS}"
}

getActionLastIndex() {
    local QUERY_OUTPUT=$(erdpy --verbose contract query ${ADDRESS} --function="getActionLastIndex" --proxy=${PROXY})
    parseQueryOutput

    ACTION_INDEX=0x${PARSED}

    echo "Last action index: ${ACTION_INDEX}"
}

getCurrentTx() {
    erdpy --verbose contract query ${ADDRESS} --function="getCurrentTx" --proxy=${PROXY}
}

# helpers

parseQueryOutput() {
    PARSED=$(jq -r '.[0].hex' <<< "${QUERY_OUTPUT}")
}

parsedAddressToBech32() {
    ADDRESS_BECH32=$(erdpy wallet bech32 --encode ${PARSED})
}

bobSign() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${BOB} \
    --gas-limit=25000000 --function="sign" \
    --arguments ${ACTION_INDEX} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

bech32ToHex() {
    ADDRESS_HEX=$(erdpy wallet bech32 --decode $1)
}
