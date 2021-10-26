# Alice will be the owner of the multisig
# Bob will be the single board member
# Quorum size will be 1

# Path towards PEM files
PROJECT="../"
ALICE="./wallets/alice.pem"
BOB="./wallets/bob.pem"

SHARD0="./walletsRelay/walletKey0.pem"
SHARD1="./walletsRelay/walletKey1.pem"
SHARD2="./walletsRelay/walletKey2.pem"
SHARD3="./walletsRelay/walletKey3.pem"
SHARD4="./walletsRelay/walletKey4.pem"
SHARD5="./walletsRelay/walletKey5.pem"
SHARD6="./walletsRelay/walletKey6.pem"
SHARD7="./walletsRelay/walletKey7.pem"
SHARD8="./walletsRelay/walletKey8.pem"
SHARD9="./walletsRelay/walletKey9.pem"

ADDRESS=$(erdpy data load --key=address-testnet-multisig)
DEPLOY_TRANSACTION=$(erdpy data load --key=deployTransaction-testnet)
PROXY=https://devnet-gateway.elrond.com
CHAIN_ID=D

RELAYER_REQUIRED_STAKE=0x0a # 1
ESDT_ISSUE_COST=0xB1A2BC2EC50000 # 0.05 eGLD
ESDT_ISSUE_COST_DECIMAL=50000000000000000

# Addresses in Hex
BOB_ADDRESS=0x8049d639e5a6980d1cd2392abcce41029cda74a1563523a202f09641cc2618f8 #erd1spyavw0956vq68xj8y4tenjpq2wd5a9p2c6j8gsz7ztyrnpxrruqzu66jx
RELAYER_ADDR_0=0xf211c0ef9e3e73e5d71b90b1702a1e8e91c673db61d0e37237f056c92ec789e1 #erd145e2wagtz839j9l9v7aehfww83ys86zcqh3a42ql3gqutp2r60ssqsdmrm
RELAYER_ADDR_1=0xad32a7750b11e25917e567bb9ba5ce3c4903e85805e3daa81f8a01c58543d3e1 #erd12dys5euw48xvlekgqwn44ans5vpx7t576delre88w8chqq20t4fs3njzv6
RELAYER_ADDR_2=0x07330515eac4693f9a82b545abc95c8b664849d8b6edfcc906036b38a2211de1 #erd1ques2902c35nlx5zk4z6hj2u3dnysjwckmklejgxqd4n3g3prhsseufpww
RELAYER_ADDR_3=0xfbad113f18a7a8d6bb58341841733503ddea02bbe4b163eec8c9412baec7b5e1 #erd1lwk3z0cc575ddw6cxsvyzue4q0w75q4mujck8mkge9qjhtk8khssys2w8x
RELAYER_ADDR_4=0x201407f111187fd6d79fb303f016fd61b0eaffbeb09e4bd78f34397dd5a82de1 #erd1yq2q0ug3rplad4ulkvplq9havxcw4la7kz0yh4u0xsuhm4dg9hssmd9c48
RELAYER_ADDR_5=0x6127f90f2a3e27ed63f9b0ad3cf80955e2d7341fa69370ac5af8ca1fe9e46ee1 #erd1vynljre28cn76clekzkne7qf2h3dwdql56fhptz6lr9pl60ydmssfuva7h
RELAYER_ADDR_6=0xe376d3dea0b86fb42fff41865d33362fd92256cd7d3f2dceb6d05e6fd216c0e1 #erd1udmd8h4qhphmgtllgxr96vek9lvjy4kd05ljmn4k6p0xl5skcrss7h4pxf
RELAYER_ADDR_7=0x92bb60da2d9325e973fc71639bab1b5eb14fda803f0ef665d8f4d31e8af416e1 #erd1j2akpk3djvj7juluw93eh2cmt6c5lk5q8u80vewc7nf3azh5zmssjngku5
RELAYER_ADDR_8=0x53f015fcea3a549d037c479fdad321534a8c9f188333fabcf711258d64e417e1 #erd120cptl828f2f6qmug70a45ep2d9ge8ccsvel408hzyjc6e8yzlssxu7x0z
RELAYER_ADDR_9=0x2e02f59e3bd0ecab3ec22b1ad4932628199e157d31af62e6fce49133577044e1 #erd19cp0t83m6rk2k0kz9vddfyex9qveu9taxxhk9ehuujgnx4msgnssv9gxrk


ESDT_SYSTEM_SC_ADDRESS=erd1qqqqqqqqqqqqqqqpqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzllls8a5w6u

# Setup and aggregator first, then put its address hex-encoded in this variable
AGGREGATOR_ADDRESS=0x0000000000000000050081d0b65d6bd5bd7d5af6df1a26e89513c6f38cd5e3df

#########################################################################
################## Update after issueing the tokens #####################
#########################################################################
WRAPPED_EGLD_TOKEN_ID=0x45474c442d663466346534
WRAPPED_ETH_TOKEN_ID=0x4554482d353461396635

deploy() {
    local SLASH_AMOUNT=0x0a # 1

    erdpy --verbose contract deploy --project=${PROJECT} --recall-nonce --pem=${ALICE} \
    --gas-limit=400000000 \
    --arguments ${RELAYER_REQUIRED_STAKE} ${SLASH_AMOUNT} 0x03 \
    ${RELAYER_ADDR_0} \
    ${RELAYER_ADDR_1} ${RELAYER_ADDR_2} ${RELAYER_ADDR_3} \
    ${RELAYER_ADDR_4} ${RELAYER_ADDR_5} ${RELAYER_ADDR_6} \
    ${RELAYER_ADDR_7} ${RELAYER_ADDR_8} ${RELAYER_ADDR_9} \
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
    local EGLD_ESDT_SWAP_CODE=0x"$(xxd -p ../../egld-esdt-swap/output/egld-esdt-swap.wasm | tr -d '\n')"
    local ESDT_SAFE_CODE=0x"$(xxd -p ../../esdt-safe/output/esdt-safe.wasm | tr -d '\n')"
    local MULTI_TRANSFER_ESDT_CODE=0x"$(xxd -p ../../multi-transfer-esdt/output/multi-transfer-esdt.wasm | tr -d '\n')"

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

    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${SHARD9} \
    --gas-limit=35000000 --function="stake" --value=${RELAYER_REQUIRED_STAKE_DECIMAL} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

addTokenToWhitelist() {
    local RELAYER_REQUIRED_STAKE_DECIMAL=1000

    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=60000000 --function="esdtSafeAddTokenToWhitelist" \
    --arguments ${WRAPPED_EGLD_TOKEN_ID} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

unstake() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${BOB} \
    --gas-limit=35000000 --function="unstake" \
    --arguments ${RELAYER_REQUIRED_STAKE} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

addMapping() {
    local WRAPPED_EGLD_ERC20=0x64a8bfab8e7ac5d5a3561d95b504542e9e29ce24
    local WRAPPED_ETH_ERC20=0x4d75EF4411cda0E0C257383054Fe68febB993D37

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
    local INITIAL_SUPPLY=0x00 # 0
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
    local INITIAL_SUPPLY=0x00 # 0
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

    sleep 10

    erdpy --verbose contract call ${ESDT_SYSTEM_SC_ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=60000000 --function="setSpecialRole" \
    --arguments ${WRAPPED_ETH_TOKEN_ID} 0x${ADDRESS_HEX} ${LOCAL_MINT_ROLE} ${LOCAL_BURN_ROLE} \
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
