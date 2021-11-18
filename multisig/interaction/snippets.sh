# First independent steps (considering ./build-wasm.sh was run)
# 1. Make sure we have the tokens deployed
# 2. Update WRAPPED_USDC_TOKEN_ID, WRAPPED_USDC_TOKEN_TICKER, WRAPPED_USDC_ERC20
# 3. Update paths to Alice (Owner, Relayers, and Relayer Keys)
# 4. Deploy aggregator (compile, copy it) and run deployAggregator
# 5. Call submitAggregatorBatch to set gas price for eth
# 6. deploySafe
# 7. deployMultiTransfer
# 8. deployMultisig
# 9. changeChildContractsOwnership # - this changes the owher of the safe and multitransfer to the multisig
# 10. setLocalRolesEsdtSafe # - keep in mind we need to do this with the token owner
# 11. setLocalRolesMultiTransferEsdt # - keep in mind we need to do this with the token owner
# 12. addMapping
# 13. addTokenToWhitelist
# 14. stake # foreach relayer


PROJECT="../"
PROJECT_SAFE="../../esdt-safe/"
PROJECT_MULTI_TRANSFER="../../multi-transfer-esdt/"
ALICE="./wallets/alice.pem"

# We don't care about Bob
BOB="./wallets/bob.pem"

RELAYER_WALLET0="./walletsRelay/walletKey0.pem"
RELAYER_WALLET1="./walletsRelay/walletKey1.pem"
RELAYER_WALLET2="./walletsRelay/walletKey2.pem"
RELAYER_WALLET3="./walletsRelay/walletKey3.pem"
RELAYER_WALLET4="./walletsRelay/walletKey4.pem"
RELAYER_WALLET5="./walletsRelay/walletKey5.pem"
RELAYER_WALLET6="./walletsRelay/walletKey6.pem"
RELAYER_WALLET7="./walletsRelay/walletKey7.pem"
RELAYER_WALLET8="./walletsRelay/walletKey8.pem"
RELAYER_WALLET9="./walletsRelay/walletKey9.pem"

ADDRESS=$(erdpy data load --key=address-testnet-multisig)
DEPLOY_TRANSACTION=$(erdpy data load --key=deployTransaction-testnet)
PROXY=https://testnet-gateway.elrond.com
CHAIN_ID=T

RELAYER_REQUIRED_STAKE=0x00
ESDT_ISSUE_COST=0xB1A2BC2EC50000 # 0.05 eGLD
ESDT_ISSUE_COST_DECIMAL=50000000000000000

# Addresses in Hex
BOB_ADDRESS=0x8049d639e5a6980d1cd2392abcce41029cda74a1563523a202f09641cc2618f8 #erd1spyavw0956vq68xj8y4tenjpq2wd5a9p2c6j8gsz7ztyrnpxrruqzu66jx

RELAYER_ADDR_0=0xb329c8f455de725bceabb0babde4149cecb93271b5e65d0c2bfbd69b076d3ce1 # erd1kv5u3az4mee9hn4tkzatmeq5nnktjvn3khn96rptl0tfkpmd8nssy33j7h 
RELAYER_ADDR_1=0x5448490305368f25bc50621365d8f61635d5cffb40c0926fe04322e23519a4e1 # erd123yyjqc9x68jt0zsvgfktk8kzc6atnlmgrqfymlqgv3wydge5nsszgrh43 
RELAYER_ADDR_2=0x8ee4f54b4a921b4a2edfbac7a13c595fd24994df41ed4c588a76b99029aef8e1 # erd13mj02j62jgd55tklhtr6z0zetlfyn9xlg8k5cky2w6ueq2dwlrssyal2h0 
RELAYER_ADDR_3=0xb1688f415a1f8f2edff0511ecd46c01542cf276bca4fda9e41a2d771771953e1 # erd1k95g7s26r78jahls2y0v63kqz4pv7fmtef8a48jp5tthzace20sskjhyhk 
RELAYER_ADDR_4=0x3844f480db43bc3979657c1b0dd8cc2e65c8fa65de52e99626c63407076b59e1 # erd18pz0fqxmgw7rj7t90sdsmkxv9eju37n9mefwn93xcc6qwpmtt8ssxwt23m 
RELAYER_ADDR_5=0xa7ed1b4b1a81fe04660decddba8598b557c720dd9952ba13c10a3acf7c2d31e1 # erd15lk3kjc6s8lqgesdanwm4pvck4tuwgxan9ft5y7ppgav7lpdx8ss20s4t3 
RELAYER_ADDR_6=0x92b56ae0f225d7c46b3808d56e35903f99e9f3c73b95fdf214c1d39f5952d9e1 # erd1j26k4c8jyhtug6ecpr2kudvs87v7nu788w2lmus5c8fe7k2jm8sstdga6w 
RELAYER_ADDR_7=0x0af2faeb60ed12f015917952c2d8c925bbca790688d76e6186d867d9eef3dee1 # erd1pte046mqa5f0q9v309fv9kxfykau57gx3rtkucvxmpnanmhnmmsstjcrl2 
RELAYER_ADDR_8=0xba63c4b5cf545a23b1bf81608063c9240941a357cd3a098102669cc33dad3de1 # erd1hf3ufdw023dz8vdls9sgqc7fysy5rg6he5aqnqgzv6wvx0dd8hsskdgfkh 
RELAYER_ADDR_9=0xa4c2a2aa3a744e4d77f6af37ef1e3cab10d4f6cf030edd11f73e3eb58a180ee1 # erd15np29236w38y6alk4um7783u4vgdfak0qv8d6y0h8clttzscpmssfp6u37 



ESDT_SYSTEM_SC_ADDRESS=erd1qqqqqqqqqqqqqqqpqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzllls8a5w6u

#########################################################################
################## Update after issueing the tokens #####################
#########################################################################
WRAPPED_EGLD_TOKEN_ID=0x45474c442d373166643366
WRAPPED_ETH_TOKEN_ID=0x4554482d386562613330
WRAPPED_USDC_TOKEN_ID=0x57555344432d656464333133

# Token ticker - needed for mainnet
WRAPPED_USDC_TOKEN_TICKER=0x5755534443

# ETH Tokens
WRAPPED_USDC_ERC20=0x1A49c6550A93d0b24417D154D2B062258dEDEd79

deployAggregator() {
    erdpy --verbose contract deploy --bytecode=../../price-aggregator/price-aggregator.wasm --recall-nonce --pem=${ALICE} \
    --gas-limit=100000000 --arguments ${WRAPPED_USDC_TOKEN_ID} 0x0139472eff6886771a982f3083da5d421f24c29181e63888228dc81ca60d69e1 0x01 0x02 0x00 \
    --send --outfile=price-aggregator.interaction.json --proxy=${PROXY} --chain=${CHAIN_ID} || return
}

submitAggregatorBatch() {
    getAggregatorAddress

    local GWEI_TICKER=0x47574549
    local GAS_PRICE_ON_ETH=0x2710

    erdpy --verbose contract call ${AGGREGATOR_ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=15000000 --function="submitBatch" \
    --arguments ${GWEI_TICKER} ${WRAPPED_USDC_TOKEN_TICKER} ${GAS_PRICE_ON_ETH} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID} || return
}

deploySafe() {
    getAggregatorAddressHex

    local ESDT_SAFE_ETH_TX_GAS_LIMIT=20000 # gives us 200$ for elrond->eth

    erdpy --verbose contract deploy --project=${PROJECT_SAFE} --recall-nonce --pem=${ALICE} \
    --gas-limit=100000000 \
    --arguments 0x${AGGREGATOR_ADDRESS_HEX} ${ESDT_SAFE_ETH_TX_GAS_LIMIT} \
    --send --outfile="deploy-safe-testnet.interaction.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return

    TRANSACTION=$(erdpy data parse --file="./deploy-safe-testnet.interaction.json" --expression="data['emitted_tx']['hash']")
    ADDRESS=$(erdpy data parse --file="./deploy-safe-testnet.interaction.json" --expression="data['emitted_tx']['address']")

    erdpy data store --key=address-testnet-safe --value=${ADDRESS}
    erdpy data store --key=deployTransaction-testnet --value=${TRANSACTION}

    echo ""
    echo "Safe contract address: ${ADDRESS}"
}

deployMultiTransfer() {
    getAggregatorAddressHex

    local MULTI_TRANSFER_ESDT_TX_GAS_LIMIT=10000 # gives us 100$ fee for eth->elrond

    erdpy --verbose contract deploy --project=${PROJECT_MULTI_TRANSFER} --recall-nonce --pem=${ALICE} \
    --gas-limit=100000000 \
    --arguments 0x${AGGREGATOR_ADDRESS_HEX} ${MULTI_TRANSFER_ESDT_TX_GAS_LIMIT} \
    --send --outfile="deploy-multitransfer-testnet.interaction.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return

    ADDRESS=$(erdpy data parse --file="./deploy-multitransfer-testnet.interaction.json" --expression="data['emitted_tx']['address']")
    erdpy data store --key=address-testnet-multitransfer --value=${ADDRESS}

    echo ""
    echo "Multi transfer contract address: ${ADDRESS}"
}

deployMultisig() {
    local SLASH_AMOUNT=0x00 # 1

    getEsdtSafeAddressHex
    getMultiTransferEsdtAddressHex

    erdpy --verbose contract deploy --project=${PROJECT} --recall-nonce --pem=${ALICE} \
    --gas-limit=200000000 \
    --arguments 0x${ESDT_SAFE_ADDRESS_HEX} 0x${MULTI_TRANSFER_ESDT_ADDRESS_HEX} \
    ${RELAYER_REQUIRED_STAKE} ${SLASH_AMOUNT} 0x07 \
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
    echo "Multisig contract address: ${ADDRESS}"
}

changeChildContractsOwnership() {
    getEsdtSafeAddress
    getMultiTransferEsdtAddress

    bech32ToHex ${ADDRESS}

    erdpy --verbose contract call ${ESDT_SAFE_ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=10000000 --function="ChangeOwnerAddress" \
    --arguments 0x${ADDRESS_HEX} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}

    sleep 10

    erdpy --verbose contract call ${MULTI_TRANSFER_ESDT_ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=10000000 --function="ChangeOwnerAddress" \
    --arguments 0x${ADDRESS_HEX} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

setLocalRolesEsdtSafe() {
    getEsdtSafeAddress
    bech32ToHex ${ESDT_SAFE_ADDRESS}

    local LOCAL_BURN_ROLE=0x45534454526f6c654c6f63616c4275726e # "ESDTRoleLocalBurn"

    erdpy --verbose contract call ${ESDT_SYSTEM_SC_ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=60000000 --function="setSpecialRole" \
    --arguments ${WRAPPED_USDC_TOKEN_ID} 0x${ADDRESS_HEX} ${LOCAL_BURN_ROLE} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

setLocalRolesMultiTransferEsdt() {
    getMultiTransferEsdtAddress
    bech32ToHex ${MULTI_TRANSFER_ESDT_ADDRESS}

    local LOCAL_MINT_ROLE=0x45534454526f6c654c6f63616c4d696e74 # "ESDTRoleLocalMint"

    erdpy --verbose contract call ${ESDT_SYSTEM_SC_ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=60000000 --function="setSpecialRole" \
    --arguments ${WRAPPED_USDC_TOKEN_ID} 0x${ADDRESS_HEX} ${LOCAL_MINT_ROLE} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

addMapping() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=40000000 --function="addMapping" \
    --arguments ${WRAPPED_USDC_ERC20} ${WRAPPED_USDC_TOKEN_ID} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

addTokenToWhitelist() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=60000000 --function="esdtSafeAddTokenToWhitelist" \
    --arguments ${WRAPPED_USDC_TOKEN_ID} ${WRAPPED_USDC_TOKEN_TICKER} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}

    sleep 10

    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=60000000 --function="multiTransferEsdtaddTokenToWhitelist" \
    --arguments ${WRAPPED_USDC_TOKEN_ID} ${WRAPPED_USDC_TOKEN_TICKER} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

stake() {
    local RELAYER_REQUIRED_STAKE_DECIMAL=0

    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${RELAYER_WALLET9} \
    --gas-limit=35000000 --function="stake" --value=${RELAYER_REQUIRED_STAKE_DECIMAL} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}


#==========================
deploySafeForUpgrade() {
    getAggregatorAddressHex

    local ESDT_SAFE_ETH_TX_GAS_LIMIT=20000 # gives us 200$ for elrond->eth

    erdpy --verbose contract deploy --project=${PROJECT_SAFE} --recall-nonce --pem=${ALICE} \
    --gas-limit=100000000 \
    --arguments 0x${AGGREGATOR_ADDRESS_HEX} ${ESDT_SAFE_ETH_TX_GAS_LIMIT} \
    --send --outfile="deploy-safe-upgrade.interaction.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return

    ADDRESS=$(erdpy data parse --file="./deploy-safe-upgrade.interaction.json" --expression="data['emitted_tx']['address']")

    echo ""
    echo "Safe contract address: ${ADDRESS}"
}


upgradeSafeContract() {
    local ESDT_SAFE_ETH_TX_GAS_LIMIT=20000

    local OLD_SAFE_BECH=$(erdpy data parse --file="./deploy-safe-testnet.interaction.json" --expression="data['emitted_tx']['address']")
    local OLD_SAFE_ADDR=$(erdpy wallet bech32 --decode $OLD_SAFE_BECH)

    local NEW_SAFE_BECH=$(erdpy data parse --file="./deploy-safe-upgrade.interaction.json" --expression="data['emitted_tx']['address']")
    local NEW_SAFE_ADDR=$(erdpy wallet bech32 --decode $NEW_SAFE_BECH)

    local AGG_ADDR_BECH=$(erdpy data parse --file="./price-aggregator.interaction.json" --expression="data['emitted_tx']['address']")
    local AGG_ADDR=$(erdpy wallet bech32 --decode $AGG_ADDR_BECH)


    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=400000000 --function="upgradeChildContractFromSource" \
    --arguments 0x${OLD_SAFE_ADDR} 0x${NEW_SAFE_ADDR} \
    0x${AGG_ADDR} ${ESDT_SAFE_ETH_TX_GAS_LIMIT} \
    --send --outfile="upgradesafe-child-sc-spam.json" --proxy=${PROXY} --chain=${CHAIN_ID}
}

upgrade() {
    erdpy --verbose contract upgrade ${ADDRESS} --project=${PROJECT} --recall-nonce --pem=${ALICE} \
    --gas-limit=100000000 --send --outfile="upgrade.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return
}

upgradeMultisig() {
    local MULTISIG_CURR_ADDR="erd1qqqqqqqqqqqqqpgqeyayz09s2a4gnvcghdh9ma3he3j7cda0d8ss2apk2a"
    local SLASH_AMOUNT=0x0a # 1


    erdpy --verbose contract upgrade ${MULTISIG_CURR_ADDR} --bytecode=../output/multisig.wasm --recall-nonce --pem=${ALICE} \
    --arguments 0x00000000000000000500e4666522747ef0403cd8405ded02de0aebf12ddc69e1 0x00000000000000000500f91fb98b98bb28f6732a4598d4b2b307657049f869e1 \
    ${RELAYER_REQUIRED_STAKE} ${SLASH_AMOUNT} 0x03 \
    --gas-limit=200000000 --send --outfile="upgrade-multisig.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return
    
}

# ====================================================================================================

deployNewSafe() {

    local ESDT_SAFE_ETH_TX_GAS_LIMIT=20000 # gives us 100$ for elrond->eth

    erdpy --verbose contract deploy --project=${PROJECT_SAFE} --recall-nonce --pem=${ALICE} \
    --gas-limit=100000000 \
    --arguments 0x00000000000000000500db2991666072326ef7b69d72b2510a9e192ddfa069e1 ${ESDT_SAFE_ETH_TX_GAS_LIMIT} \
    --send --outfile="deploy-safe-testnet.interaction.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return

    TRANSACTION=$(erdpy data parse --file="./deploy-safe-testnet.interaction.json" --expression="data['emitted_tx']['hash']")
    ADDRESS=$(erdpy data parse --file="./deploy-safe-testnet.interaction.json" --expression="data['emitted_tx']['address']")

    erdpy data store --key=address-testnet-safe --value=${ADDRESS}
    erdpy data store --key=deployTransaction-testnet --value=${TRANSACTION}

    echo ""
    echo "Safe contract address: ${ADDRESS}"
}



updateAggregator() {
    NEW_AGG_ADDR=0x00000000000000000500db2991666072326ef7b69d72b2510a9e192ddfa069e1
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=60000000 --function="changeFeeEstimatorContractAddress" \
    --arguments ${NEW_AGG_ADDR} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

unstake() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${BOB} \
    --gas-limit=35000000 --function="unstake" \
    --arguments ${RELAYER_REQUIRED_STAKE} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

clearMapping() {
    local WRAPPED_USDC_ERC20=0x9ae05a468ffDf9E3c5CC6B84B486EFAA9554f0B1

     erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=40000000 --function="clearMapping" \
    --arguments ${WRAPPED_USDC_ERC20} ${WRAPPED_USDC_TOKEN_ID} \
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

issueWrappedUSDC() {
    local TOKEN_DISPLAY_NAME=0x5772617070656455534443  # "WrappedUSDC"
    local TOKEN_TICKER=0x5755534443  # "WUSDC"
    local INITIAL_SUPPLY=0x00 # 0
    local NR_DECIMALS=0x06 # 6
    local CAN_ADD_SPECIAL_ROLES=0x63616e4164645370656369616c526f6c6573 # "canAddSpecialRoles"
    local TRUE=0x74727565 # "true"

    erdpy --verbose contract call ${ESDT_SYSTEM_SC_ADDRESS} --recall-nonce --pem=${ALICE} \
    --gas-limit=60000000 --value=${ESDT_ISSUE_COST_DECIMAL} --function="issue" \
    --arguments ${TOKEN_DISPLAY_NAME} ${TOKEN_TICKER} ${INITIAL_SUPPLY} ${NR_DECIMALS} ${CAN_ADD_SPECIAL_ROLES} ${TRUE} \
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
    ADDRESS_BECH32=$(erdpy data parse --file="./deploy-safe-testnet.interaction.json" --expression="data['emitted_tx']['address']")
    ESDT_SAFE_ADDRESS=${ADDRESS_BECH32}
    echo "EsdtSafe address: ${ESDT_SAFE_ADDRESS}"
}

getMultiTransferEsdtAddress() {
    ADDRESS_BECH32=$(erdpy data parse --file="./deploy-multitransfer-testnet.interaction.json" --expression="data['emitted_tx']['address']")
    MULTI_TRANSFER_ESDT_ADDRESS=${ADDRESS_BECH32}
    echo "MultiTransferEsdt address: ${MULTI_TRANSFER_ESDT_ADDRESS}"
}

getAggregatorAddress() {
    AGGREGATOR_ADDRESS=$(erdpy data parse --file="./price-aggregator.interaction.json" --expression="data['emitted_tx']['address']")
    echo "MultiTransferEsdt address: ${AGGREGATOR_ADDRESS}"
}

getEsdtSafeAddressHex() {
    getEsdtSafeAddress
    ESDT_SAFE_ADDRESS_HEX=$(erdpy wallet bech32 --decode $ESDT_SAFE_ADDRESS)  
}

getMultiTransferEsdtAddressHex() {
    getMultiTransferEsdtAddress
    MULTI_TRANSFER_ESDT_ADDRESS_HEX=$(erdpy wallet bech32 --decode $MULTI_TRANSFER_ESDT_ADDRESS)
}

getAggregatorAddressHex() {
    getAggregatorAddress
    AGGREGATOR_ADDRESS_HEX=$(erdpy wallet bech32 --decode $AGGREGATOR_ADDRESS)
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
