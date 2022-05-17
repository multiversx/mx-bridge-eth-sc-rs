#!/bin/bash
set -e

#Make script aware of its location
SCRIPTPATH="$( cd "$(dirname -- "$0")" ; pwd -P )"

source $SCRIPTPATH/config/configs.cfg
source $SCRIPTPATH/config/menu_functions.cfg
CONFIG_FILE=$SCRIPTPATH/config/configs.cfg

case "$1" in
'deploy-aggregator')
  confirmation deploy-aggregator
  ;;

'deploy-wrapper')
  confirmation deploy-wrapper    
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
  confirmation pause
  continue-confirmation pauseEsdtSafe
  ;;

'unpause-contracts')
  confirmation unpause
  continue-confirmation unpauseEsdtSafe
  ;;

*)
  echo "Usage: Invalid choice: '"$1"'" 
  echo -e 
  echo "Choose from:"
  echo "  { \"deploy-aggregator\", \"deploy-wrapper\", \"deploy-bridge-contracts\", \"add-relayer\", \"remove-relayer\", \"whitelist-token\", "
  echo "    \"set-safe-max-tx\", \"set-safe-batch-block-duration\", \"change-quorum\", \"pause-contracts\", \"unpause-contracts\" }"
  ;;

esac