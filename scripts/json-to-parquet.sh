#!/bin/bash

rm -f data.parquet
query=$(cat <<EOF
COPY (
  SELECT *, filename
  FROM read_json_auto('data/events-*.jsonl')
) to 'data.parquet' (FORMAT parquet);
EOF
)
scripts/duckdb.sh -c "${query}"