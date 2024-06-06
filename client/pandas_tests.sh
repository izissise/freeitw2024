#!/bin/env bash

set -eu

HERE="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" && pwd)"

run_example() {
    local code=$(curl -sSL "https://raw.githubusercontent.com/lshang0311/pandas-examples/master/$1" -o -)

    jq -n --arg name "$1" --argjson "py" "$(jq -n --arg pycode "$code" --arg entrypoint "" '$ARGS.named')" '$ARGS.named' | "$HERE"/lambdas/put.sh

    echo "$1"
    "$HERE"/lambdas/exec.sh "$1" "bwrap"
    echo
}

examples=(
"create_a_new_column_by_adding_values_from_other_columns.py"
"create_dataframe_from_a_list_of_dicts.py"
"drop_duplicates.py"
"generate_example_series_and_dataframe.py"
"get_last_friday_with_relativedelta_in_dateutil.py"
"groupby_split_apply_combine.py"
"multiple_indexers.py"
"replace_nans_by_preceding_values.py"
"reset_index.py"
"sort_index.py"
"use_lambda_to_rename_columns.py"
"use_list_comprehension_to_rename_columns.py"
)

for e in "${examples[@]}"; do
    run_example "$e"
done
