#!/bin/env bash

set -eu

API=${API:-127.0.0.1:3000}

curl -s -L -X DELETE "$API"/lambdas/"$1" | jq -r .
