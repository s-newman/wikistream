#!/bin/bash

tmpf=$(mktemp)
cat data/stream*.txt | grep "data: " | sed 's/^data: //' | jq -R 'fromjson? | .timestamp' | awk '{ rem = $1 % 60; minute = $1 - rem; print minute }' | sort > "${tmpf}"
uniq -c "${tmpf}" | awk '{ avg = int($1 / 60); print avg }' | sort -nr | uniq -c
echo "Total lines: $(wc -l "${tmpf}")"
rm "${tmpf}"
