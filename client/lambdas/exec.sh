#!/bin/env bash

set -eu

API=${API:-127.0.0.1:3000}

LAMBDA=$1
shift 1

printf '%s ' "$@" | curl -v -s -L -H 'Content-Type: text' -X POST "$API"/lambdas/"$LAMBDA"/exec
