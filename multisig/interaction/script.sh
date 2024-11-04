#!/bin/bash
set -e

#Make script aware of its location
SCRIPTPATH="$( cd "$(dirname -- "$0")" ; pwd -P )"

source $SCRIPTPATH/config/configs.cfg
source $SCRIPTPATH/config/helper.cfg
source $SCRIPTPATH/config/menu_functions.cfg

case "$1" in
'deploy-bridge-contracts')
  confirmation deploy-bridge-contracts
  ;;

'add-relayer')
  confirmation addBoardMember
  ;;

'remove-relayer')
  confirmation removeBoardMember
  ;;

'whitelist-token')
  echo -e 
  echo "PREREQUIREMENTS: BRIDGED_TOKENS_WRAPPER needs to have MINT+BURN role for the UNIVERSAL TOKEN"
  echo "Check and update TOKENS SETTINGS section in configs.cfg"
  source $SCRIPTPATH/config/configs.cfg
  echo -e
  confirmation whitelist-token
  ;;

'whitelist-native-token')
  echo -e 
  echo "Check and update TOKENS SETTINGS section in configs.cfg"
  source $SCRIPTPATH/config/configs.cfg
  echo -e
  confirmation whitelist-native-token
  ;;

'remove-whitelist-token')
  echo -e 
  echo "PREREQUIREMENTS: BRIDGED_TOKENS_WRAPPER needs to have MINT+BURN role for the UNIVERSAL TOKEN"
  echo "Check and update TOKENS SETTINGS section in configs.cfg"
  source $SCRIPTPATH/config/configs.cfg
  echo -e
  confirmation remove-whitelist-token
  ;;

'set-safe-max-tx')
  confirmation set-safe-max-tx
  ;;

'set-safe-batch-block-duration')
  confirmation set-safe-batch-block-duration
  ;;

'change-quorum')
  confirmation change-quorum
  ;;

'pause-contracts')
  confirmation pause-contracts
  ;;

'unpause-contracts')
  confirmation unpause-contracts
  ;;

'set-swap-fee')
  confirmation set-fee
  ;;

'mint-chain-specific')
  confirmation mint-chain-specific
  ;;

'upgrade-wrapper-universal-token')
  confirmation upgrade-wrapper-universal-token   
  ;;

'upgrade-wrapper-chain-specific-token')
  confirmation upgrade-wrapper-chain-specific-token  
  ;;

'init-supply-mint-burn')
  confirmation init-supply-mint-burn
  ;;

'upgrade-aggregator')
  confirmation upgrade-aggregator
  ;;

'upgrade-wrapper')
  confirmation upgrade-wrapper
  ;;

'upgrade-safe')
  confirmation upgrade-safe
  ;;

'upgrade-multi-transfer')
  confirmation upgrade-multi-transfer
  ;;

'upgrade-proxy')
  confirmation upgrade-proxy
  ;;

'upgrade-multisig')
  confirmation upgrade-multisig
  ;;

'whitelist-native-token')
  confirmation whitelist-native-token
  ;;

'faucet-deposit')
  confirmation faucet-deposit
  ;;

'deploy-test-caller')
  confirmation deploy-test-caller
  ;;

'pause-v2-contracts')
  confirmation pause-v2-contracts
  ;;

*)
  echo "Usage: Invalid choice: '"$1"'" 
  echo -e 
  echo "Choose from:"
  echo "  { \"deploy-bridge-contracts\", "
  echo "    \"upgrade-aggregator\", \"upgrade-wrapper\", \"upgrade-safe\", \"upgrade-multi-transfer\", \"upgrade-proxy\", \"upgrade-multisig\" "
  echo "    \"pause-contracts\", \"unpause-contracts\", \"add-relayer\", \"remove-relayer\", \"pause-v2-contracts\", "
  echo "    \"set-safe-max-tx\", \"set-safe-batch-block-duration\", \"change-quorum\", \"set-swap-fee\", "
  echo "    \"whitelist-token\", \"whitelist-native-token\", \"remove-whitelist-token\", \"upgrade-wrapper-universal-token\", \"upgrade-wrapper-chain-specific-token\", "
  echo "    \"mint-chain-specific\", \"init-supply-mint-burn\", "
  echo "    \"faucet-deposit\", \"deploy-test-caller\""
  echo "  } "
  ;;

esac