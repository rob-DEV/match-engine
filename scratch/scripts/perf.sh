#!/bin/zsh

CONNECTIONS=$1
DURATION=$2


/bin/zsh -c "wrk -t8 -c$CONNECTIONS -d$DURATION http://127.0.0.1:3001/order -s conf.lua"
