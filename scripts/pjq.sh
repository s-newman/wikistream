#!/bin/bash

if [ $# -ne 1 ]; then
  "USAGE: $0 FILTER" >&2
  exit 1
fi

export filter="${1}"

run_jq() {
  jq -r "${filter}" "${1}"
}
export -f run_jq

find data -type f -name "events-*.jsonl" -print0 | parallel -0 run_jq