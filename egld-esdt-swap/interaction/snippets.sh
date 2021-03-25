ALICE="/home/elrond/elrond-sdk/erdpy/testnet/wallets/users/alice.pem"
ADDRESS=$(erdpy data load --key=address-testnet)
DEPLOY_TRANSACTION=$(erdpy data load --key=deployTransaction-testnet)
PROXY=http://localhost:7950
CHAIN_ID=local-testnet

TOKEN_DISPLAY_NAME=0x5772617070656445676c64  # "WrappedEgld"
TOKEN_TICKER=0x5745474c44  # "WEGLD"
TOKEN_IDENTIFIER=0x5745474c442d383864366361 # Manually update after issue

UNWRAP_EGLD_ENDPOINT=0x756e7772617045676c64

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

issueWrappedEgld() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=60000000 --value=5000000000000000000 --function="issueWrappedEgld" --arguments ${TOKEN_DISPLAY_NAME} ${TOKEN_TICKER} 0x03E8 --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

setLocalMintRole() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=60000000 --function="setLocalMintRole" --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

localMint() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=10000000 --function="mintWrappedEgld" --arguments 0x64 --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

wrapEgld() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=10000000 --value=10 --function="wrapEgld" --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

unwrapEgld() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=1000000000 --function="ESDTTransfer" --arguments ${TOKEN_IDENTIFIER} 0x05 ${UNWRAP_EGLD_ENDPOINT} --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

# views

getWrappedEgldTokenIdentifier() {
    erdpy --verbose contract query ${ADDRESS} --function="getWrappedEgldTokenIdentifier" --proxy=${PROXY}
}

getLockedEgldBalance() {
    erdpy --verbose contract query ${ADDRESS} --function="getLockedEgldBalance" --proxy=${PROXY}
}

getWrappedEgldRemaining() {
    erdpy --verbose contract query ${ADDRESS} --function="getWrappedEgldRemaining" --proxy=${PROXY}
}
