#!/bin/bash

if [ $# -ne 1 ]; then
  echo "USAGE: $0 FILE" >&2
  exit 1
fi

outdir=$(dirname -- "${1}")
filename=$(basename -- "${1}")
filename="${filename%.*}"
outpath="${outdir}/${filename}.jsonl"

grep "data: " "${1}" | sed 's/^data: //' | jq --unbuffered -R -c 'fromjson? | .' > "${outpath}"