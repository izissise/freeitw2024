#!/bin/env bash

set -eu

API=${API:-127.0.0.1:3000}

LAMBDA=$1
shift 1

curl -v -s -L -H "Transfer-Encoding: chunked" -H 'Content-Type: text' -X POST "$API"/lambdas/"$LAMBDA"/exec -T - -G --data status=true --data sandbox=host --data-urlencode "args=$*"

