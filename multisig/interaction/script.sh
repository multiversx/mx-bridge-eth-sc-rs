#!/bin/bash
set -e

#Make script aware of its location
SCRIPTPATH="$( cd "$(dirname -- "$0")" ; pwd -P )"

source $SCRIPTPATH/config/configs.cfg
source $SCRIPTPATH/config/helper.cfg
source $SCRIPTPATH/config/menu_functions.cfg

case "$1" in
'deploy-aggregator')
  confirmation deploy-aggregator
  ;;

'deploy-wrapper')
  confirmation deploy-wrapper    
  ;;

'upgrade-wrapper')
  confirmation upgrade-wrapper    
  ;;

'deploy-bridge-contracts')
  echo -e 
  echo "PREREQUIREMENTS: AGGREGATOR & BRIDGED_TOKENS_WRAPPER deployed"
  echo -e 
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

*)
  echo "Usage: Invalid choice: '"$1"'" 
  echo -e 
  echo "Choose from:"
  echo "  { \"deploy-aggregator\", \"deploy-wrapper\", \"upgrade-wrapper\", \"deploy-bridge-contracts\", \"add-relayer\", \"remove-relayer\", \"whitelist-token\", "
  echo "    \"remove-whitelist-token\", \"set-safe-max-tx\", \"set-safe-batch-block-duration\", \"change-quorum\", \"pause-contracts\", \"unpause-contracts\", "
  echo "    \"set-swap-fee\", \"mint-chain-specific\", \"upgrade-wrapper-universal-token\", \"upgrade-wrapper-chain-specific-token\" }"
  ;;

esac