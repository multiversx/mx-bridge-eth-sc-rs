ALICE="/home/elrond/elrond-sdk/mxpy/testnet/wallets/users/alice.pem"
BOB="/home/elrond/elrond-sdk/mxpy/testnet/wallets/users/bob.pem"
ADDRESS=$(mxpy data load --key=address-testnet-esdt-safe)
DEPLOY_TRANSACTION=$(mxpy data load --key=deployTransaction-testnet)
PROXY=https://testnet-gateway.elrond.com
CHAIN_ID=T

BOB_ADDRESS=0x8049d639e5a6980d1cd2392abcce41029cda74a1563523a202f09641cc2618f8 # 32 bytes
CAROL_ADDRESS=0xb2a11555ce521e4944e09ab17549d85b487dcd26c84b5017a39e31a3670889ba # 32 bytes
ALICE_ETH_ADDRESS=0x7d61a56899dd55e5D16C1Bab38f46f42b4d33887 # 20 bytes

TX_STATUS_EXECUTED=0x03
TX_STATUS_REJECTED=0x04

ESDT_SYSTEM_SC_ADDRESS=erd1qqqqqqqqqqqqqqqpqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzllls8a5w6u

########################################################################
################## Update after issuing the tokens #####################
########################################################################
WRAPPED_EGLD_TOKEN_ID=0x
WRAPPED_ETH_TOKEN_ID=0x

deploy() {
    #######################################################################
    ################## Update with the contract's address #################
    #######################################################################
    local ETHEREUM_FEE_PREPAY_SC_ADDRESS=0x

    mxpy --verbose contract deploy --project=${PROJECT} --recall-nonce --pem=${ALICE} \
    --gas-limit=100000000 \
    --arguments ${ETHEREUM_FEE_PREPAY_SC_ADDRESS} ${WRAPPED_EGLD_TOKEN_ID} ${WRAPPED_ETH_TOKEN_ID} \
    --send --outfile="deploy-testnet.interaction.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return

    TRANSACTION=$(mxpy data parse --file="deploy-testnet.interaction.json" --expression="data['emitted_tx']['hash']")
    ADDRESS=$(mxpy data parse --file="deploy-testnet.interaction.json" --expression="data['emitted_tx']['address']")

    mxpy data store --key=address-testnet-esdt-safe --value=${ADDRESS}
    mxpy data store --key=deployTransaction-testnet --value=${TRANSACTION}

    echo ""
    echo "Smart contract address: ${ADDRESS}"
}

upgrade() {
    mxpy --verbose contract upgrade ${ADDRESS} --project=${PROJECT} --recall-nonce --pem=${ALICE} \
    --gas-limit=100000000 --send --outfile="upgrade.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return
}

issueWrappedEth() {
    local TOKEN_DISPLAY_NAME=0x57726170706564457468  # "WrappedEth"
    local TOKEN_TICKER=0x57455448  # "WETH"
    local INITIAL_SUPPLY=0x01 # 1
    local NR_DECIMALS=0x12 # 18
    local CAN_ADD_SPECIAL_ROLES=0x63616e4164645370656369616c526f6c6573 # "canAddSpecialRoles"
    local TRUE=0x74727565 # "true"

    mxpy --verbose contract call ${ESDT_SYSTEM_SC_ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=60000000 --value=5000000000000000000 --function="issue" \
    --arguments ${TOKEN_DISPLAY_NAME} ${TOKEN_TICKER} ${INITIAL_SUPPLY} ${NR_DECIMALS} ${CAN_ADD_SPECIAL_ROLES} ${TRUE} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

setLocalRolesWrappedEgld() {
    local LOCAL_BURN_ROLE=0x45534454526f6c654c6f63616c4275726e # "ESDTRoleLocalBurn"
    local ADDRESS_HEX = $(mxpy wallet bech32 --decode ${ADDRESS})

    mxpy --verbose contract call ${ESDT_SYSTEM_SC_ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=60000000 --function="setSpecialRole" \
    --arguments ${WRAPPED_EGLD_TOKEN_ID} ${ADDRESS_HEX} ${LOCAL_BURN_ROLE} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

setLocalRolesWrappedEth() {
    local LOCAL_BURN_ROLE=0x45534454526f6c654c6f63616c4275726e # "ESDTRoleLocalBurn"
    local ADDRESS_HEX = $(mxpy wallet bech32 --decode ${ADDRESS})

    mxpy --verbose contract call ${ESDT_SYSTEM_SC_ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=60000000 --function="setSpecialRole" \
    --arguments ${WRAPPED_ETH_TOKEN_ID} ${ADDRESS_HEX} ${LOCAL_BURN_ROLE} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

getNextPendingTransaction() {
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=25000000 --function="getNextPendingTransaction" \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

setTransactionExecuted() {
    local RELAYER_REWARD_ADDRESS = ${CAROL_ADDRESS}
    local ORIGINAL_TX_SENDER = ${BOB_ADDRESS}
    local TX_NONCE = 0x01
    local TX_STATUS = ${TX_STATUS_EXECUTED}

    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=90000000 --function="setTransactionStatus" \
    --arguments ${RELAYER_REWARD_ADDRESS} ${ORIGINAL_TX_SENDER} ${TX_NONCE} ${TX_STATUS} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

setTransactionRejected() {
    local RELAYER_REWARD_ADDRESS = ${CAROL_ADDRESS}
    local ORIGINAL_TX_SENDER = ${BOB_ADDRESS}
    local TX_NONCE = 0x02
    local TX_STATUS = ${TX_STATUS_REJECTED}

    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=90000000 --function="setTransactionStatus" \
    --arguments ${RELAYER_REWARD_ADDRESS} ${ORIGINAL_TX_SENDER} ${TX_NONCE} ${TX_STATUS} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}D}
}

createTransaction() {
    local CREATE_TRANSACTION_ENDPOINT = 0x6372656174655472616e73616374696f6e # "createTransaction"
    local DEST_ADDRESS = ${ALICE_ETH_ADDRESS}
    local TOKEN_USED_FOR_TX_FEES = 0x45474c44 # "EGLD"
    
    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${BOB} \
    --gas-limit=50000000 --function="ESDTTransfer" \
    --arguments ${WRAPPED_EGLD_TOKEN_IDENTIFIER} 0x64 ${CREATE_TRANSACTION_ENDPOINT} ${DEST_ADDRESS} ${TOKEN_USED_FOR_TX_FEES} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

# views

getTransactionStatus() {
    mxpy --verbose contract query ${ADDRESS} --function="getTransactionStatus" \
    --arguments ${BOB_ADDRESS} 0x01 \
    --proxy=${PROXY}
}
