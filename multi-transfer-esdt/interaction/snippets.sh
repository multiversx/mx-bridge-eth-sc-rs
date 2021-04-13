ALICE="/home/elrond/elrond-sdk/erdpy/testnet/wallets/users/alice.pem"
BOB="/home/elrond/elrond-sdk/erdpy/testnet/wallets/users/bob.pem"
ADDRESS=$(erdpy data load --key=address-testnet)
DEPLOY_TRANSACTION=$(erdpy data load --key=deployTransaction-testnet)
PROXY=http://localhost:7950
CHAIN_ID=local-testnet

ALICE_ADDRESS=0x0139472eff6886771a982f3083da5d421f24c29181e63888228dc81ca60d69e1
BOB_ADDRESS=0x8049d639e5a6980d1cd2392abcce41029cda74a1563523a202f09641cc2618f8

TOKEN_DISPLAY_NAME=0x57726170706564457468  # "WrappedEth"
TOKEN_TICKER=0x57455448  # "WETH"
TOKEN_IDENTIFIER=0x574554482d386230623966 # Manually update after issue

WRAPPED_EGLD_IDENTIFIER=0x # Manually update after issuing from Egld-Esdt-Swap SC

deploy() {
    erdpy --verbose contract deploy --project=${PROJECT} --recall-nonce --pem=${ALICE} --gas-limit=100000000 --send --outfile="deploy-testnet.interaction.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return

    TRANSACTION=$(erdpy data parse --file="deploy-testnet.interaction.json" --expression="data['emitted_tx']['hash']")
    ADDRESS=$(erdpy data parse --file="deploy-testnet.interaction.json" --expression="data['emitted_tx']['address']")

    erdpy data store --key=address-testnet --value=${ADDRESS}
    erdpy data store --key=deployTransaction-testnet --value=${TRANSACTION}

    echo ""
    echo "Smart contract address: ${ADDRESS}"
}

upgrade() {
    erdpy --verbose contract upgrade ${ADDRESS} --project=${PROJECT} --recall-nonce --pem=${ALICE} --gas-limit=100000000 --send --outfile="upgrade.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return
}

issueEsdtToken() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=90000000 --value=5000000000000000000 --function="issueEsdtToken" --arguments ${TOKEN_DISPLAY_NAME} ${TOKEN_TICKER} --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

setLocalMintRole() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=90000000 --function="setLocalMintRole" --arguments ${TOKEN_IDENTIFIER} --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

addTokenToIssuedList() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=10000000 --function="addTokenToIssuedList" --arguments ${WRAPPED_EGLD_IDENTIFIER} --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

transferEsdtToken() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=10000000 --function="transferEsdtToken" --arguments ${BOB_ADDRESS} ${TOKEN_IDENTIFIER} 0x64 --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

getLastIssuedToken() {
    erdpy --verbose contract query ${ADDRESS} --function="getLastIssuedToken" --proxy=${PROXY}
}

getScEsdtBalance() {
    erdpy --verbose contract query ${ADDRESS} --function="getScEsdtBalance" --arguments ${TOKEN_IDENTIFIER} --proxy=${PROXY}
}
