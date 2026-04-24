#!/bin/bash

set -euo pipefail

tempf=$(mktemp)
find data -type f -name "events-*.jsonl" -print0 | parallel -0 jq -r \''select(.type == "log") | .log_type'\' | sort | uniq > "${tempf}"

for log_type in $(cat "${tempf}"); do
  grep --no-filename "\"log_type\":\"${log_type}\"" data/events-*.jsonl | head -n 1 | jq > "sample-log-${log_type}.json"
done