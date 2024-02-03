#!/bin/zsh

CONNECTIONS=$1
DURATION=$2
CONFIG=$3

SCRIPT_DIR=$(dirname "$0")

/bin/zsh -c "wrk -t8 -c$CONNECTIONS -d$DURATION http://127.0.0.1:3001/order -s $SCRIPT_DIR/$CONFIG.lua"
