deployMultisig() {
    CHECK_VARIABLES RELAYER_ADDR_0 RELAYER_ADDR_1 RELAYER_ADDR_2 RELAYER_ADDR_3 \
    RELAYER_ADDR_4 RELAYER_ADDR_5 RELAYER_ADDR_6 RELAYER_ADDR_7 RELAYER_ADDR_8 \
    RELAYER_ADDR_9 SAFE MULTI_TRANSFER BRIDGE_PROXY RELAYER_REQUIRED_STAKE SLASH_AMOUNT QUORUM MULTISIG_WASM

    MIN_STAKE=$(echo "$RELAYER_REQUIRED_STAKE*10^18" | bc)
    mxpy contract deploy --bytecode=${MULTISIG_WASM} --recall-nonce "${MXPY_SIGN[@]}" \
    --gas-limit=200000000 \
    --arguments ${SAFE} ${MULTI_TRANSFER} ${BRIDGE_PROXY} \
    ${MIN_STAKE} ${SLASH_AMOUNT} ${QUORUM} \
    ${RELAYER_ADDR_0} ${RELAYER_ADDR_1} ${RELAYER_ADDR_2} ${RELAYER_ADDR_3} \
    --send --outfile="deploy-testnet.interaction.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return

    TRANSACTION=$(mxpy data parse --file="./deploy-testnet.interaction.json" --expression="data['emitted_tx']['hash']")
    ADDRESS=$(mxpy data parse --file="./deploy-testnet.interaction.json" --expression="data['contractAddress']")

    mxpy data store --key=address-testnet-multisig --value=${ADDRESS}
    mxpy data store --key=deployTransaction-testnet --value=${TRANSACTION}

    echo ""
    echo "Multisig contract address: ${ADDRESS}"
    update-config MULTISIG ${ADDRESS}
}

changeChildContractsOwnershipSafe() {
    CHECK_VARIABLES SAFE MULTISIG

    mxpy contract call ${SAFE} --recall-nonce "${MXPY_SIGN[@]}" \
    --gas-limit=10000000 --function="ChangeOwnerAddress" \
    --arguments ${MULTISIG} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

changeChildContractsOwnershipProxy() {
    CHECK_VARIABLES BRIDGE_PROXY MULTISIG

    mxpy contract call ${BRIDGE_PROXY} --recall-nonce "${MXPY_SIGN[@]}" \
    --gas-limit=10000000 --function="ChangeOwnerAddress" \
    --arguments ${MULTISIG} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

changeChildContractsOwnershipMultiTransfer() {
    CHECK_VARIABLES MULTI_TRANSFER MULTISIG

    mxpy contract call ${MULTI_TRANSFER} --recall-nonce "${MXPY_SIGN[@]}" \
    --gas-limit=10000000 --function="ChangeOwnerAddress" \
    --arguments ${MULTISIG} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

clearMapping() {
    CHECK_VARIABLES ERC20_TOKEN CHAIN_SPECIFIC_TOKEN MULTISIG

    mxpy contract call ${MULTISIG} --recall-nonce "${MXPY_SIGN[@]}" \
    --gas-limit=40000000 --function="clearMapping" \
    --arguments ${ERC20_TOKEN} str:${CHAIN_SPECIFIC_TOKEN} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

addMapping() {
    CHECK_VARIABLES ERC20_TOKEN CHAIN_SPECIFIC_TOKEN MULTISIG

    mxpy contract call ${MULTISIG} --recall-nonce "${MXPY_SIGN[@]}" \
    --gas-limit=40000000 --function="addMapping" \
    --arguments ${ERC20_TOKEN} str:${CHAIN_SPECIFIC_TOKEN} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

addTokenToWhitelist() {
    CHECK_VARIABLES CHAIN_SPECIFIC_TOKEN CHAIN_SPECIFIC_TOKEN_TICKER MULTISIG MINTBURN_WHITELIST NATIVE_TOKEN

    BALANCE=$(echo "$TOTAL_BALANCE*10^$NR_DECIMALS_CHAIN_SPECIFIC" | bc)
    MINT=$(echo "$MINT_BALANCE*10^$NR_DECIMALS_CHAIN_SPECIFIC" | bc)
    BURN=$(echo "$BURN_BALANCE*10^$NR_DECIMALS_CHAIN_SPECIFIC" | bc)

    mxpy contract call ${MULTISIG} --recall-nonce "${MXPY_SIGN[@]}" \
    --gas-limit=60000000 --function="esdtSafeAddTokenToWhitelist" \
    --arguments str:${CHAIN_SPECIFIC_TOKEN} str:${CHAIN_SPECIFIC_TOKEN_TICKER} ${MINTBURN_WHITELIST} ${NATIVE_TOKEN} \
    ${BALANCE} ${MINT} ${BURN} --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

removeTokenFromWhitelist() {
    CHECK_VARIABLES CHAIN_SPECIFIC_TOKEN CHAIN_SPECIFIC_TOKEN_TICKER MULTISIG

    mxpy contract call ${MULTISIG} --recall-nonce "${MXPY_SIGN[@]}" \
    --gas-limit=60000000 --function="esdtSafeRemoveTokenFromWhitelist" \
    --arguments str:${CHAIN_SPECIFIC_TOKEN} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

esdtSafeSetMaxTxBatchSize() {
    CHECK_VARIABLES MAX_TX_PER_BATCH MULTISIG

    mxpy contract call ${MULTISIG} --recall-nonce "${MXPY_SIGN[@]}" \
    --gas-limit=30000000 --function="esdtSafeSetMaxTxBatchSize" \
    --arguments ${MAX_TX_PER_BATCH} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

esdtSafeSetMaxTxBatchBlockDuration() {
    CHECK_VARIABLES MAX_TX_BLOCK_DURATION_PER_BATCH MULTISIG

    mxpy contract call ${MULTISIG} --recall-nonce "${MXPY_SIGN[@]}" \
    --gas-limit=30000000 --function="esdtSafeSetMaxTxBatchBlockDuration" \
    --arguments ${MAX_TX_BLOCK_DURATION_PER_BATCH} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

clearMapping() {
    CHECK_VARIABLES ERC20_TOKEN CHAIN_SPECIFIC_TOKEN MULTISIG

     mxpy contract call ${MULTISIG} --recall-nonce "${MXPY_SIGN[@]}" \
    --gas-limit=40000000 --function="clearMapping" \
    --arguments ${ERC20_TOKEN} str:${CHAIN_SPECIFIC_TOKEN} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

changeQuorum() {
    CHECK_VARIABLES QUORUM MULTISIG

    mxpy contract call ${MULTISIG} --recall-nonce "${MXPY_SIGN[@]}" \
    --gas-limit=40000000 --function="changeQuorum" \
    --arguments ${QUORUM} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

pause() {
    CHECK_VARIABLES MULTISIG

    mxpy contract call ${MULTISIG} --recall-nonce "${MXPY_SIGN[@]}" \
    --gas-limit=40000000 --function="pause" \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

pauseV2() {
    CHECK_VARIABLES MULTISIG_v2

    mxpy contract call ${MULTISIG_v2} --recall-nonce "${MXPY_SIGN[@]}" \
    --gas-limit=40000000 --function="pause" \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

pauseEsdtSafe() {
    CHECK_VARIABLES MULTISIG

    mxpy contract call ${MULTISIG} --recall-nonce "${MXPY_SIGN[@]}" \
    --gas-limit=40000000 --function="pauseEsdtSafe" \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

pauseEsdtSafeV2() {
    CHECK_VARIABLES MULTISIG_v2

    mxpy contract call ${MULTISIG_v2} --recall-nonce "${MXPY_SIGN[@]}" \
    --gas-limit=40000000 --function="pauseEsdtSafe" \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

pauseProxy() {
    CHECK_VARIABLES MULTISIG

    mxpy contract call ${MULTISIG} --recall-nonce "${MXPY_SIGN[@]}" \
    --gas-limit=40000000 --function="pauseProxy" \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

unpause() {
    CHECK_VARIABLES MULTISIG

    mxpy contract call ${MULTISIG} --recall-nonce "${MXPY_SIGN[@]}" \
    --gas-limit=40000000 --function="unpause" \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

unpauseEsdtSafe() {
    CHECK_VARIABLES MULTISIG

    mxpy contract call ${MULTISIG} --recall-nonce "${MXPY_SIGN[@]}" \
    --gas-limit=40000000 --function="unpauseEsdtSafe" \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

unpauseProxy() {
    CHECK_VARIABLES MULTISIG

    mxpy contract call ${MULTISIG} --recall-nonce "${MXPY_SIGN[@]}" \
    --gas-limit=40000000 --function="unpauseProxy" \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

esdtSafeSetMaxBridgedAmountForToken() {
    CHECK_VARIABLES MAX_AMOUNT NR_DECIMALS_CHAIN_SPECIFIC CHAIN_SPECIFIC_TOKEN MULTISIG

    MAX=$(echo "scale=0; $MAX_AMOUNT*10^$NR_DECIMALS_CHAIN_SPECIFIC/1" | bc)
    mxpy contract call ${MULTISIG} --recall-nonce "${MXPY_SIGN[@]}" \
    --gas-limit=40000000 --function="esdtSafeSetMaxBridgedAmountForToken" \
    --arguments str:${CHAIN_SPECIFIC_TOKEN} ${MAX} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

multiTransferEsdtSetMaxBridgedAmountForToken() {
    CHECK_VARIABLES MAX_AMOUNT NR_DECIMALS_CHAIN_SPECIFIC CHAIN_SPECIFIC_TOKEN MULTISIG

    MAX=$(echo "scale=0; $MAX_AMOUNT*10^$NR_DECIMALS_CHAIN_SPECIFIC/1" | bc)
    mxpy contract call ${MULTISIG} --recall-nonce "${MXPY_SIGN[@]}" \
    --gas-limit=40000000 --function="multiTransferEsdtSetMaxBridgedAmountForToken" \
    --arguments str:${CHAIN_SPECIFIC_TOKEN} ${MAX} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}


setMultiTransferOnEsdtSafeThroughMultisig() {
    CHECK_VARIABLES MULTISIG

    mxpy contract call ${MULTISIG} --recall-nonce "${MXPY_SIGN[@]}" \
    --gas-limit=60000000 --function="setMultiTransferOnEsdtSafe" \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

setEsdtSafeOnMultiTransferThroughMultisig() {
    CHECK_VARIABLES MULTISIG

    mxpy contract call ${MULTISIG} --recall-nonce "${MXPY_SIGN[@]}" \
    --gas-limit=60000000 --function="setEsdtSafeOnMultiTransfer" \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

initSupplyMintBurn() {
  CHECK_VARIABLES MULTISIG

  echo -e
  echo "PREREQUIREMENTS: The MINT_BALANCE & BURN_BALANCE values should be defined in configs.cfg file"
  echo "The script automatically apply denomination factors based on the number of the decimals the token has"
  echo -e

  confirmation-with-skip manual-update-config-file

  MINT=$(echo "$MINT_BALANCE*10^$NR_DECIMALS_CHAIN_SPECIFIC" | bc)
  BURN=$(echo "$BURN_BALANCE*10^$NR_DECIMALS_CHAIN_SPECIFIC" | bc)

  MINT=$(echo ${MINT%.*}) # trim decimals, if existing
  BURN=$(echo ${BURN%.*}) # trim decimals, if existing

  mxpy contract call ${MULTISIG} --recall-nonce "${MXPY_SIGN[@]}" \
  --gas-limit=60000000 --function="initSupplyMintBurnEsdtSafe" \
  --arguments str:${CHAIN_SPECIFIC_TOKEN} ${MINT} ${BURN} \
  --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

syncValueWithEthereumDenom() {
  CHECK_VARIABLES MULTISIG SAFE

  read -p "Chain specific token (human readable): " TOKEN
  read -p "Denominated value on Ethereum (should contain all digits): " ETH_VALUE

  EXISTING_BURN=$(mxpy contract query ${SAFE} --proxy=${PROXY} --function getBurnBalances --arguments str:$TOKEN | jq '.[0].number')
  EXISTING_MINT=$(mxpy contract query ${SAFE} --proxy=${PROXY} --function getMintBalances --arguments str:$TOKEN | jq '.[0].number')
  NEW_MINT=$(echo "$ETH_VALUE+$EXISTING_BURN" | bc)
  DIFF=$(echo "$EXISTING_MINT-$EXISTING_BURN" | bc)
  NEW_DIFF=$(echo "$NEW_MINT-$EXISTING_BURN" | bc)

  echo "For token ${TOKEN} the existing mint is ${EXISTING_MINT} and existing burn is ${EXISTING_BURN}. The minted value will be replaced with ${NEW_MINT}"
  echo "Existing diff ${DIFF}, new diff will be ${NEW_DIFF}"

  mxpy contract call ${MULTISIG} --recall-nonce "${MXPY_SIGN[@]}" \
    --gas-limit=60000000 --function="initSupplyMintBurnEsdtSafe" \
    --arguments str:${TOKEN} ${NEW_MINT} ${EXISTING_BURN} \
    --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

upgradeMultisig() {
    CHECK_VARIABLES SAFE MULTI_TRANSFER BRIDGE_PROXY MULTISIG_WASM

    mxpy contract upgrade ${MULTISIG} --bytecode=${MULTISIG_WASM} --recall-nonce "${MXPY_SIGN[@]}" \
      --gas-limit=100000000 --send \
      --arguments ${SAFE} ${MULTI_TRANSFER} ${BRIDGE_PROXY} \
      --outfile="upgrade-multisig-child-sc.json" --proxy=${PROXY} --chain=${CHAIN_ID} || return

    TRANSACTION=$(mxpy data parse --file="./upgrade-multisig-child-sc.json" --expression="data['emitted_tx']['hash']")
    ADDRESS=$(mxpy data parse --file="./upgrade-multisig-child-sc.json" --expression="data['contractAddress']")

    echo ""
    echo "Multisig contract updated: ${ADDRESS}"
}
