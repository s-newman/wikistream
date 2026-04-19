#!/bin/bash

compress() {
  zstd --quiet -o "${FILE}"
}
export -f compress
export SHELL=/bin/bash

find data -type f -name "stream-*.txt" -print0 | parallel -0 scripts/process-one-raw.sh

cat data/stream*.jsonl | split -l 10000 -a 4 -d --additional-suffix=".jsonl.zstd" --filter="compress" - archive/events-
rm -f data/stream*.jsonl