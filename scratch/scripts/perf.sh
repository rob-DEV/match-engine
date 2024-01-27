#!/bin/zsh

CONNECTIONS=$1

/bin/zsh -c "wrk -t8 -c$CONNECTIONS -d30s http://127.0.0.1:3000/order -s conf.lua"
