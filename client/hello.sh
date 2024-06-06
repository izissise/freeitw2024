#!/bin/env bash

set -eu

HERE="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" && pwd)"

echo "[+] Install lambda"

jq -n --arg name "hello" --argjson "bash" "$(jq -n --arg script 'echo Hello $@' '$ARGS.named')" '$ARGS.named' | "$HERE"/lambdas/put.sh

read -p "What's your name: " -r name

"$HERE"/lambdas/exec.sh "hello" "host" "$name"
echo
