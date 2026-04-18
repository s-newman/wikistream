#!/bin/bash

tmpf=$(mktemp)
cat data/stream*.txt | grep "data: " | sed 's/^data: //' | jq -R 'fromjson? | .timestamp' | awk '{ rem = $1 % 60; minute = $1 - rem; print minute }' | sort > "${tmpf}"
cat "${tmpf}" | uniq -c | awk '{ avg = int($1 / 60); print avg }' | sort -nr | uniq -c
echo "Total lines: $(cat ${tmpf} | wc -l)"
rm "${tmpf}"
