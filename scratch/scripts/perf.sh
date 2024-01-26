#!/bin/zsh

wrk -t8 -c100 -d30s http://127.0.0.1:3000/order -s config.lua
