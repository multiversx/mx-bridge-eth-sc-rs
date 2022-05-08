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
  confirmation deploy-bridge-contracts
  ;;

'add-relayer')
  confirmation addRelayer
  ;;

'remove-relayer')
  confirmation removeRelayer
  ;;

'whitelist-token')
  confirmation whitelist-token
  ;;

'get_logs')
  confirmation get_logs
  ;;

*)
  echo "Usage: Missing parameter ! [deploy-aggregator|deploy-wrapper|deploy-bridge-contracts|add-relayer|remove-relayer|whitelist-token]"
  ;;

esac