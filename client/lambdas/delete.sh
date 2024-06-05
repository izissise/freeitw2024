#!/bin/env bash

set -eu

API=${API:-127.0.0.1:3000}

curl -s -w '%{http_code}' -L -X DELETE "$API"/lambdas/"$1"
