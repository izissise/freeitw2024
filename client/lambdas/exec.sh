#!/bin/env bash

set -eu

API=${API:-127.0.0.1:3000}

LAMBDA=$1
SANDBOX=$2
shift 2

curl -s -L -H "Transfer-Encoding: chunked" -H 'Content-Type: text' -X POST "$API"/lambdas/"$LAMBDA"/exec -T - -G --data status=true --data sandbox="$SANDBOX" --data-urlencode "args=$*"

