#!/bin/env bash

set -eu

API=${API:-127.0.0.1:3000}

curl -s -L -X GET "$API"/sandboxs | jq -r .
