ALICE="/home/elrond/elrond-sdk/mxpy/testnet/wallets/users/alice.pem"
BOB="/home/elrond/elrond-sdk/mxpy/testnet/wallets/users/bob.pem"
ADDRESS=$(mxpy data load --key=address-testnet-multi-transfer-esdt)
DEPLOY_TRANSACTION=$(mxpy data load --key=deployTransaction-testnet)
PROXY=https://testnet-gateway.elrond.com
CHAIN_ID=T

ALICE_ADDRESS=0x0139472eff6886771a982f3083da5d421f24c29181e63888228dc81ca60d69e1
BOB_ADDRESS=0x8049d639e5a6980d1cd2392abcce41029cda74a1563523a202f09641cc2618f8

########################################################################
################## Update after issuing the tokens #####################
########################################################################
WRAPPED_EGLD_TOKEN_ID=0x
WRAPPED_ETH_TOKEN_ID=0x

deploy() {
    mxpy --verbose contract deploy --project=${PROJECT} \
    --arguments ${WRAPPED_EGLD_TOKEN_ID} ${WRAPPED_ETH_TOKEN_ID} \
    --recall-nonce --pem=${ALICE} --gas-limit=100000000 --send \
    --outfile="deploy-testnet.interaction.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return

    TRANSACTION=$(mxpy data parse --file="deploy-testnet.interaction.json" --expression="data['emitted_tx']['hash']")
    ADDRESS=$(mxpy data parse --file="deploy-testnet.interaction.json" --expression="data['emitted_tx']['address']")

    mxpy data store --key=address-testnet --value=${ADDRESS}
    mxpy data store --key=deployTransaction-testnet-multi-transfer-esdt --value=${TRANSACTION}

    echo ""
    echo "Smart contract address: ${ADDRESS}"
}

upgrade() {
    mxpy --verbose contract upgrade ${ADDRESS} --project=${PROJECT} --recall-nonce --pem=${ALICE} \
    --gas-limit=100000000 --send --outfile="upgrade.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return
}

setLocalRolesWrappedEgld() {
    local LOCAL_MINT_ROLE=0x45534454526f6c654c6f63616c4d696e74 # "ESDTRoleLocalMint"
    local ADDRESS_HEX = $(mxpy wallet bech32 --decode ${ADDRESS})

    mxpy --verbose contract call ${ESDT_SYSTEM_SC_ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=60000000 --function="setSpecialRole" \
    --arguments ${WRAPPED_EGLD_TOKEN_ID} ${ADDRESS_HEX} ${LOCAL_MINT_ROLE} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

setLocalRolesWrappedEth() {
    local LOCAL_MINT_ROLE=0x45534454526f6c654c6f63616c4d696e74 # "ESDTRoleLocalMint"
    local ADDRESS_HEX = $(mxpy wallet bech32 --decode ${ADDRESS})

    mxpy --verbose contract call ${ESDT_SYSTEM_SC_ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=60000000 --function="setSpecialRole" \
    --arguments ${WRAPPED_ETH_TOKEN_ID} ${ADDRESS} ${LOCAL_MINT_ROLE} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

transferEsdtToken() {
    local DEST_ADDRESS = BOB_ADDRESS
    local TOKEN_ID = WRAPPED_ETH_TOKEN_ID
    local AMOUNT = 0x05

    mxpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=10000000 --function="transferEsdtToken" \
    --arguments ${DEST_ADDRESS} ${TOKEN_ID} ${AMOUNT} --send --proxy=${PROXY} --chain=${CHAIN_ID}
}
