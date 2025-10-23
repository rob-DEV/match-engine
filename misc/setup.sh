#!/bin/zsh

PASSWORD=$1

echo  "CREATE SPACE IF NOT EXISTS executions" | skysh --password $PASSWORD
echo  "USE SPACE executions" | skysh --password $PASSWORD
