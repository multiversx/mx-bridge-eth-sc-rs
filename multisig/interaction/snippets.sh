# Alice will be the owner of the multisig
# Bob will be the single board member
# Quorum size will be 1

ALICE="/home/elrond/elrond-sdk/erdpy/testnet/wallets/users/alice.pem"
BOB="/home/elrond/elrond-sdk/erdpy/testnet/wallets/users/bob.pem"
ADDRESS=$(erdpy data load --key=address-testnet)
DEPLOY_TRANSACTION=$(erdpy data load --key=deployTransaction-testnet)
PROXY=http://localhost:7950
CHAIN_ID=local-testnet

EGLD_ESDT_SWAP_CODE=0x"$(xxd -p ../egld-esdt-swap/output/egld-esdt-swap.wasm | tr -d '\n')"
ESDT_SAFE_CODE=0x"$(xxd -p ../esdt-safe/output/esdt-safe.wasm | tr -d '\n')"
MULTI_TRANSFER_ESDT_CODE=0x"$(xxd -p ../multi-transfer-esdt/output/multi-transfer-esdt.wasm | tr -d '\n')"
ETHEREUM_FEE_PREPAY_CODE=0x"$(xxd -p ../ethereum-fee-prepay/output/ethereum-fee-prepay.wasm | tr -d '\n')"

RELAYER_REQUIRED_STAKE=0x03e8 # 1000
RELAYER_REQUIRED_STAKE_DECIMAL=1000
SLASH_AMOUNT=0x01f4 # 500
ESDT_ISSUE_COST=0x4563918244f40000 # 5 eGLD
ESDT_ISSUE_COST_DECIMAL=5000000000000000000
BOB_ADDRESS=0x8049d639e5a6980d1cd2392abcce41029cda74a1563523a202f09641cc2618f8

# Setup and aggregator first, then put its address hex-encoded in this variable
AGGREGATOR_ADDRESS=0x00000000000000000500f8f0a3640b575e92b09423ccdb32dc6c2399eec869e1

deploy() {
    erdpy --verbose contract deploy --project=${PROJECT} --recall-nonce --pem=${ALICE} --gas-limit=200000000 --arguments ${RELAYER_REQUIRED_STAKE} ${SLASH_AMOUNT} 0x01 ${BOB_ADDRESS} --send --outfile="deploy-testnet.interaction.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return

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

deployChildContracts() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=400000000 --function="deployChildContracts" --arguments ${EGLD_ESDT_SWAP_CODE} ${MULTI_TRANSFER_ESDT_CODE} ${ETHEREUM_FEE_PREPAY_CODE} ${ESDT_SAFE_CODE} ${AGGREGATOR_ADDRESS} --send --outfile="deploy-child-sc-spam.json" --proxy=${PROXY} --chain=${CHAIN_ID}

    sleep 10

    # finish setup
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=50000000 --function="finishSetup" --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

stake() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${BOB} --gas-limit=25000000 --function="stake" --value=${RELAYER_REQUIRED_STAKE_DECIMAL} --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

unstake() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${BOB} --gas-limit=25000000 --function="unstake" --arguments ${RELAYER_REQUIRED_STAKE} --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

issueWrappedEgld() {
    local WRAPPED_EGLD_TOKEN_DISPLAY_NAME=0x5772617070656445676c64  # "WrappedEgld"
    local WRAPPED_EGLD_TOKEN_TICKER=0x5745474c44  # "WEGLD"
    local INITIAL_SUPPLY=0x03e8 # 1000

    # Alice pays for issue cost
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=25000000 --function="deposit" --value=${ESDT_ISSUE_COST_DECIMAL} --send --proxy=${PROXY} --chain=${CHAIN_ID}
    sleep 10

    # Bob proposes wrapped eGLD issue
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${BOB} --gas-limit=25000000 --function="proposeEgldEsdtSwapWrappedEgldIssue" --arguments ${WRAPPED_EGLD_TOKEN_DISPLAY_NAME} ${WRAPPED_EGLD_TOKEN_TICKER} ${INITIAL_SUPPLY} ${ESDT_ISSUE_COST} --send --proxy=${PROXY} --chain=${CHAIN_ID}
    sleep 10

    # Bob signs the action
    getActionLastIndex
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${BOB} --gas-limit=25000000 --function="sign" --arguments ${ACTION_INDEX} --send --proxy=${PROXY} --chain=${CHAIN_ID}
    sleep 10

    # Bob executes the action
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${BOB} --gas-limit=200000000 --function="performAction" --arguments ${ACTION_INDEX} --send --proxy=${PROXY} --chain=${CHAIN_ID}
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

getWrappedEgldTokenIdentifier() {
    getEgldEsdtSwapAddress
    local QUERY_OUTPUT=$(erdpy --verbose contract query ${EGLD_ESDT_SWAP_ADDRESS} --function="getWrappedEgldTokenIdentifier" --proxy=${PROXY})
    parseQueryOutput

    WRAPPED_EGLD_TOKEN_IDENTIFIER=0x${PARSED}

    echo "Wrapped eGLD token identifier: ${WRAPPED_EGLD_TOKEN_IDENTIFIER}"
}

getActionLastIndex() {
    local QUERY_OUTPUT=$(erdpy --verbose contract query ${ADDRESS} --function="getActionLastIndex" --proxy=${PROXY})
    parseQueryOutput

    ACTION_INDEX=0x${PARSED}

    echo "Last action index: ${ACTION_INDEX}"
}

# helpers

parseQueryOutput() {
    PARSED=$(jq -r '.[0].hex' <<< "${QUERY_OUTPUT}")
}

parsedAddressToBech32() {
    ADDRESS_BECH32=$(erdpy wallet bech32 --encode ${PARSED})
}
