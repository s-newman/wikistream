#!/bin/bash

if [ $# -ne 2 ]; then
  echo "USAGE: $0 FILTER FILE" >&2
  exit 1
fi

filter="${1}"
file="${2}"

zstd -d --stdout "${file}" | jq -r "${filter}"