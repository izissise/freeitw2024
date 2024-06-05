#!/bin/env bash

set -eu

API=${API:-127.0.0.1:3000}

curl -v -L -H 'Content-Type: application/json' -X PUT "$API"/lambdas --data @/dev/stdin
