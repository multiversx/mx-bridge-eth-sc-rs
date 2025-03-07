#!/bin/bash
set -e

#Make script aware of its location
SCRIPTPATH="$( cd "$(dirname -- "$0")" ; pwd -P )"

source $SCRIPTPATH/config/configs.cfg
source $SCRIPTPATH/config/helper.cfg
source $SCRIPTPATH/config/menu_functions.cfg
source $SCRIPTPATH/release-v3/menu_functions.cfg
source $SCRIPTPATH/release-v3/upgrade-v3p1.cfg

case "$1" in

### PART 1

'pause-contracts-on-devnet')
  confirmation pause-contracts-on-devnet
  ;;

'upgrade-contracts-v3p1-on-devnet')
  confirmation upgrade-contracts-v3p1-on-devnet
  ;;

'unpause-contracts-on-devnet')
  confirmation unpause-contracts-on-devnet
  ;;

### PART 2

'pause-contracts-on-eth')
  confirmation pause-contracts-on-eth
  ;;

'upgrade-contracts-v3p1-on-eth')
  confirmation upgrade-contracts-v3p1-on-eth
  ;;

'unpause-contracts-on-eth')
  confirmation unpause-contracts-on-eth
  ;;

### PART 3

'pause-contracts-on-bsc')
  confirmation pause-contracts-on-bsc
  ;;

'upgrade-contracts-v3p1-on-bsc')
  confirmation upgrade-contracts-v3p1-on-bsc
  ;;

'unpause-contracts-on-bsc')
  confirmation unpause-contracts-on-bsc
  ;;


*)
  echo "Usage: Invalid choice: '"$1"'"
  echo -e
  echo "Choose from:"
  echo -e
  echo "PART 1 - Upgrade DEVNET bridge:"
  echo " 1.1 pause-contracts-on-devnet"
  echo " 1.2 upgrade-contracts-v3p1-on-devnet"
  echo " 1.3 unpause-contracts-on-devnet"
  echo -e
  echo "PART 2 - Upgrade Ethereum bridge:"
  echo " 2.1 pause-contracts-on-eth"
  echo " 2.2 upgrade-contracts-v3p1-on-eth"
  echo " 2.3 unpause-contracts-on-eth"
  echo -e
  echo "PART 3 - Upgrade BSC bridge:"
  echo " 3.1 pause-contracts-on-bsc"
  echo " 3.2 upgrade-contracts-v3p1-on-bsc"
  echo " 3.3 unpause-contracts-on-bsc"
  ;;

esac