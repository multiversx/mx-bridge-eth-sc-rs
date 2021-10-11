# Alice will be the owner of the multisig
# Bob will be the single board member
# Quorum size will be 1

# Path towards PEM files
<<<<<<< Updated upstream
ALICE="/home/elrond/Downloads/devnetWalletKey.pem"
BOB=""
=======
ALICE="./wallets/alice.pem"
BOB="./wallets/bob.pem"
>>>>>>> Stashed changes

ADDRESS=erd1qqqqqqqqqqqqqpgq5300tayry6ardyw66azx3tljp3uhl8jq082sluzkm4
DEPLOY_TRANSACTION=$(erdpy data load --key=deployTransaction-testnet)
PROXY=https://devnet-gateway.elrond.com
CHAIN_ID=D

RELAYER_REQUIRED_STAKE=0x0a # 1
ESDT_ISSUE_COST=0xB1A2BC2EC50000 # 0.05 eGLD
ESDT_ISSUE_COST_DECIMAL=50000000000000000

# Addresses in Hex
BOB_ADDRESS=0x

RELAYER_ADDR_0=0x5cc00bb6d62665482fb7a98f688de4576908b3d86bb6b905786c50e9a6ca3493
RELAYER_ADDR_1=0x316df040b2377b904ca7287d72f1445690e399dbacd1e6387bd72fd23f790c03
RELAYER_ADDR_2=0x861414a440f506b5e728e6367083b9eb78af2707645146d8c387471290b01c55
RELAYER_ADDR_3=0x15788d4dada25be9aaf8f9db773738ececedd379d94d69ac1773a70a5e969c36
RELAYER_ADDR_4=0x61d4fea8d1f876dd8f5ce7c12b4d68059798aa639331999b5f130973dfa711e3
RELAYER_ADDR_5=0xcb56df4813f7db30010fb3b9bc0713785c20738d7bca4028fc3840d9c7fbeb58
RELAYER_ADDR_6=0xc1d3fb6ee84b9b2ffef639f18cda542dfab5bfa86f1b6e82f6c9bc9283e695f3
RELAYER_ADDR_7=0xebe11b66f2d641c161ab02fcec75c0d1b5b883c111246d63d3293583d2b15081
RELAYER_ADDR_8=0x25b9743889b9c6b3ab8409adda606a39183dd6f2e6edf48a43b8c42dfeebb45f
RELAYER_ADDR_9=0x9b7971db47e3815a669a91c3f1bcb21e0b81f2de04bf11faa7a34b9b10e7cfbb

ESDT_SYSTEM_SC_ADDRESS=erd1qqqqqqqqqqqqqqqpqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzllls8a5w6u

# Setup and aggregator first, then put its address hex-encoded in this variable
AGGREGATOR_ADDRESS=0x0000000000000000050081d0b65d6bd5bd7d5af6df1a26e89513c6f38cd5e3df

#########################################################################
################## Update after issueing the tokens #####################
#########################################################################
WRAPPED_EGLD_TOKEN_ID=0x45474c442d316138626639
WRAPPED_ETH_TOKEN_ID=0x4554482d353063336133

deploy() {
    local SLASH_AMOUNT=0x0a # 1

    erdpy --verbose contract deploy --project=${PROJECT} --recall-nonce --pem=${ALICE} \
    --gas-limit=400000000 \
    --arguments ${RELAYER_REQUIRED_STAKE} ${SLASH_AMOUNT} 0x02 \
    ${RELAYER_ADDR_1} ${RELAYER_ADDR_2} ${RELAYER_ADDR_3} \
    --send --outfile="deploy-testnet.interaction.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return

    TRANSACTION=$(erdpy data parse --file="./deploy-testnet.interaction.json" --expression="data['emitted_tx']['hash']")
    ADDRESS=$(erdpy data parse --file="./deploy-testnet.interaction.json" --expression="data['emitted_tx']['address']")

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

    local ESDT_SAFE_ETH_TX_GAS_LIMIT=150000
    local MULTI_TRANSFER_ESDT_TX_GAS_LIMIT=10000

    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=400000000 --function="deployChildContracts" \
    --arguments ${EGLD_ESDT_SWAP_CODE} ${MULTI_TRANSFER_ESDT_CODE} ${ESDT_SAFE_CODE} \
    ${AGGREGATOR_ADDRESS} ${ESDT_SAFE_ETH_TX_GAS_LIMIT} ${MULTI_TRANSFER_ESDT_TX_GAS_LIMIT} \
    ${WRAPPED_EGLD_TOKEN_ID} ${WRAPPED_ETH_TOKEN_ID} \
    --send --outfile="deploy-child-sc-spam.json" --proxy=${PROXY} --chain=${CHAIN_ID}
}

stake() {
    local RELAYER_REQUIRED_STAKE_DECIMAL=1000

    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${BOB} \
    --gas-limit=35000000 --function="stake" --value=${RELAYER_REQUIRED_STAKE_DECIMAL} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

unstake() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${BOB} \
    --gas-limit=35000000 --function="unstake" \
    --arguments ${RELAYER_REQUIRED_STAKE} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

addMapping() {
    local WRAPPED_EGLD_ERC20=0xC06606b0248F56aA93DB3236dB0bED97B9Ad1135
    local WRAPPED_ETH_ERC20=0x1F3ff2dA93DB23be6F73696950701F5cE471D7d4

    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=40000000 --function="addMapping" \
    --arguments ${WRAPPED_EGLD_ERC20} ${WRAPPED_EGLD_TOKEN_ID} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}

    sleep 10

    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=40000000 --function="addMapping" \
    --arguments ${WRAPPED_ETH_ERC20} ${WRAPPED_ETH_TOKEN_ID} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

changeQuorum() {
    local NEW_QUORUM=0x02

    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=40000000 --function="changeQuorum" \
    --arguments ${NEW_QUORUM} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

# Issue Tokens

issueWrappedEgld() {
    local TOKEN_DISPLAY_NAME=0x5772617070656445676c64  # "WrappedEgld"
    local TOKEN_TICKER=0x45474c44  # "EGLD"
    local INITIAL_SUPPLY=0x01 # 1
    local NR_DECIMALS=0x12 # 18
    local CAN_ADD_SPECIAL_ROLES=0x63616e4164645370656369616c526f6c6573 # "canAddSpecialRoles"
    local TRUE=0x74727565 # "true"

    erdpy --verbose contract call ${ESDT_SYSTEM_SC_ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=60000000 --value=${ESDT_ISSUE_COST_DECIMAL} --function="issue" \
    --arguments ${TOKEN_DISPLAY_NAME} ${TOKEN_TICKER} ${INITIAL_SUPPLY} ${NR_DECIMALS} ${CAN_ADD_SPECIAL_ROLES} ${TRUE} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

issueWrappedEth() {
    local TOKEN_DISPLAY_NAME=0x57726170706564457468  # "WrappedEth"
    local TOKEN_TICKER=0x455448  # "ETH"
    local INITIAL_SUPPLY=0x01 # 1
    local NR_DECIMALS=0x12 # 18
    local CAN_ADD_SPECIAL_ROLES=0x63616e4164645370656369616c526f6c6573 # "canAddSpecialRoles"
    local TRUE=0x74727565 # "true"

    erdpy --verbose contract call ${ESDT_SYSTEM_SC_ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=60000000 --value=${ESDT_ISSUE_COST_DECIMAL} --function="issue" \
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
    --arguments ${WRAPPED_EGLD_TOKEN_ID} 0x${ADDRESS_HEX} ${LOCAL_MINT_ROLE} ${LOCAL_BURN_ROLE} \
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
    --arguments ${WRAPPED_EGLD_TOKEN_ID} 0x${ADDRESS_HEX} ${LOCAL_BURN_ROLE} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}

    sleep 10

    # set roles for WrappedEth
    erdpy --verbose contract call ${ESDT_SYSTEM_SC_ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=60000000 --function="setSpecialRole" \
    --arguments ${WRAPPED_ETH_TOKEN_ID} 0x${ADDRESS_HEX} ${LOCAL_BURN_ROLE} \
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
    --arguments ${WRAPPED_EGLD_TOKEN_ID} 0x${ADDRESS_HEX} ${LOCAL_MINT_ROLE} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}

    sleep 10

    # set roles for WrappedEth
    erdpy --verbose contract call ${ESDT_SYSTEM_SC_ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=60000000 --function="setSpecialRole" \
    --arguments ${WRAPPED_ETH_TOKEN_ID} 0x${ADDRESS_HEX} ${LOCAL_MINT_ROLE} \
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

fetchNextTransactionBatch() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${BOB} \
    --gas-limit=25000000 --function="fetchNextTransactionBatch" \
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

getActionLastIndex() {
    local QUERY_OUTPUT=$(erdpy --verbose contract query ${ADDRESS} --function="getActionLastIndex" --proxy=${PROXY})
    parseQueryOutput

    ACTION_INDEX=0x${PARSED}

    echo "Last action index: ${ACTION_INDEX}"
}

calculateTxCostInEgld() {
    getEsdtSafeAddress

    erdpy --verbose contract call ${ESDT_SAFE_ADDRESS} --recall-nonce --pem=${BOB} \
    --gas-limit=50000000 --function="calculateRequiredFee" \
    --arguments ${WRAPPED_EGLD_TOKEN_ID} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}

    #local QUERY_OUTPUT=$(erdpy --verbose contract query ${ESDT_SAFE_ADDRESS} --function="calculateRequiredFee" --arguments ${WRAPPED_EGLD_TOKEN_ID} --proxy=${PROXY})
    #parseQueryOutput

    #COST=0x${PARSED}

    #echo "Last action index: ${COST}"
}

calculateTxCostInEth() {
    getEsdtSafeAddress

    erdpy --verbose contract call ${ESDT_SAFE_ADDRESS} --recall-nonce --pem=${BOB} \
    --gas-limit=50000000 --function="calculateRequiredFee" \
    --arguments ${WRAPPED_ETH_TOKEN_ID} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

manualQuery() {
    erdpy --verbose contract call erd1qqqqqqqqqqqqqpgqs8gtvhtt6k7h6khkmudzd6y4z0r08rx4u00svnnxt2 --recall-nonce --pem=${BOB} \
    --gas-limit=50000000 --function="latestPriceFeedOptional" \
    --arguments 0x47574549 0x45474c44 \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

getCurrentBatch() {
    erdpy --verbose contract query ${ADDRESS} --function="getCurrentTxBatch" --proxy=${PROXY}
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
