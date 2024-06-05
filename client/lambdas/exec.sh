#!/bin/env bash

set -eu

API=${API:-127.0.0.1:3000}

LAMBDA=$1
shift 1

url_params=""
for arg in "$@"; do
  encoded_arg=$(echo "$arg" | jq -s -R -r @uri)
  url_params="${url_params}&param=${encoded_arg}"
done

curl -v -s -L -X POST "$API"/lambdas/"$1"/exec"?$url_params"
