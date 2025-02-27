#!/bin/bash
set -e

#Make script aware of its location
SCRIPTPATH="$( cd "$(dirname -- "$0")" ; pwd -P )"

source $SCRIPTPATH/config/configs.cfg
source $SCRIPTPATH/config/helper.cfg
source $SCRIPTPATH/config/menu_functions.cfg
source $SCRIPTPATH/release-v3/menu_functions.cfg

case "$1" in

### PART 1

'deploy-bridge-contracts-eth-v3')
  confirmation deploy-bridge-contracts-eth-v3
  ;;

'unpause-contracts-eth-v3')
  confirmation unpause-contracts-eth-v3
  ;;

'set-tokens-on-eth')
  confirmation set-tokens-on-eth
  ;;

'stake-oracles')
  confirmation stake-oracles
  ;;

'submit-aggregation-batches-eth')
  confirmation submit-aggregation-batches-eth
  ;;

'stake-relayers-eth')
  confirmation stake-relayers-eth
  ;;

'set-roles-on-esdt-safe-eth')
  confirmation set-roles-on-esdt-safe-eth
  ;;

### PART 2

'deploy-bridge-contracts-bsc-v3')
  confirmation deploy-bridge-contracts-bsc-v3
  ;;

'unpause-contracts-bsc-v3')
  confirmation unpause-contracts-bsc-v3
  ;;

'set-tokens-on-bsc')
  confirmation set-tokens-on-bsc
  ;;

'submit-aggregation-batches-bsc')
  confirmation submit-aggregation-batches-bsc
  ;;

'stake-relayers-bsc')
  confirmation stake-relayers-bsc
  ;;

'set-roles-on-esdt-safe-bsc')
  confirmation set-roles-on-esdt-safe-bsc
  ;;

### PART 3

'upgrade-wrapper')
  confirmation upgrade-wrapper
  ;;

'unpause-wrapper')
  confirmation unpause-wrapper
  ;;

'set-token-limits-on-eth')
  confirmation set-token-limits-on-eth
  ;;

'set-token-limits-on-bsc')
  confirmation set-token-limits-on-bsc
  ;;

*)
  echo "Usage: Invalid choice: '"$1"'"
  echo -e
  echo "Choose from:"
  echo "PART 1 - Ethereum:"
  echo " 1.1 deploy-bridge-contracts-eth-v3"
  echo " 1.2 unpause-contracts-eth-v3"
  echo " 1.3 set-tokens-on-eth"
  echo " -----------"
  echo " 1.4 stake-oracles"
  echo " 1.5 submit-aggregation-batches-eth"
  echo " 1.6 stake-relayers-eth"
  echo " -----------"
  echo " 1.7 set-roles-on-esdt-safe-eth"
  echo -e
  echo "PART 2 - BSC:"
  echo " 2.1 deploy-bridge-contracts-bsc-v3"
  echo " 2.2 unpause-contracts-bsc-v3"
  echo " 2.3 set-tokens-on-bsc"
  echo " -----------"
  echo " 2.4 submit-aggregation-batches-bsc"
  echo " 2.5 stake-relayers-bsc"
  echo " -----------"
  echo " 2.6 set-roles-on-esdt-safe-bsc"
  echo -e
  echo "PART 3 - Upgrade wrapper:"
  echo " 3.1 upgrade-wrapper"
  echo " 3.2 unpause-wrapper"
  echo -e
  echo "PART 4 - Fixes:"
  echo " 4.1 set-token-limits-on-eth"
  echo " 4.2 set-token-limits-on-eth"
  echo -e
  ;;

esac