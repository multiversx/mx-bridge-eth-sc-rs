ALICE="/home/elrond/elrond-sdk/erdpy/testnet/wallets/users/alice.pem"
BOB="/home/elrond/elrond-sdk/erdpy/testnet/wallets/users/bob.pem"
ADDRESS=$(erdpy data load --key=address-testnet)
DEPLOY_TRANSACTION=$(erdpy data load --key=deployTransaction-testnet)
PROXY=http://localhost:7950
CHAIN_ID=local-testnet

ALICE_ADDRESS=0x0139472eff6886771a982f3083da5d421f24c29181e63888228dc81ca60d69e1
BOB_ADDRESS=0x8049d639e5a6980d1cd2392abcce41029cda74a1563523a202f09641cc2618f8

TRANSACTION_FEE=1000
WRAPPED_EGLD_TOKEN_IDENTIFIER=0x5745474c442d633764373566 # issue from egld-esdt-swap first

CREATE_TRANSACTION_ENDPOINT=0x6372656174655472616e73616374696f6e

deploy() {
    erdpy --verbose contract deploy --project=${PROJECT} --recall-nonce --pem=${ALICE} --gas-limit=100000000 --arguments ${TRANSACTION_FEE} ${WRAPPED_EGLD_TOKEN_IDENTIFIER} --send --outfile="deploy-testnet.interaction.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return

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

getNextPendingTransaction() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=5000000 --function="getNextPendingTransaction" --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

setTransactionExecuted() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=90000000 --function="setTransactionStatus" --arguments ${ALICE_ADDRESS} 0x01 0x03 --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

setTransactionRejected() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=90000000 --function="setTransactionStatus" --arguments ${ALICE_ADDRESS} 0x01 0x04 --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

claim() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=5000000 --function="claim" --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

depositEgldForTransactionFee() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${BOB} --gas-limit=5000000 --value=${TRANSACTION_FEE} --function="depositEgldForTransactionFee" --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

whithdrawDeposit() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${BOB} --gas-limit=5000000 --function="whithdrawDeposit" --arguments 500 --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

# We're assuming Alice has the exact same address on another chain
# This is never the case, but we'll keep it like this for simplicity
createTransaction() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${BOB} --gas-limit=20000000 --function="ESDTTransfer" --arguments ${WRAPPED_EGLD_TOKEN_IDENTIFIER} 0x64 ${CREATE_TRANSACTION_ENDPOINT} ${ALICE_ADDRESS} --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

# views

getTransactionStatus() {
    erdpy --verbose contract query ${ADDRESS} --function="getTransactionStatus" --arguments ${BOB_ADDRESS} 0x01 --proxy=${PROXY}
}

getDeposit() {
    erdpy --verbose contract query ${ADDRESS} --function="getDeposit" --arguments ${BOB_ADDRESS} --proxy=${PROXY}
}

getClaimableTransactionFee() {
    erdpy --verbose contract query ${ADDRESS} --function="getClaimableTransactionFee" --proxy=${PROXY}
}