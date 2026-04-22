#!/bin/bash

find data -type f -name 'events-*.jsonl' -print0 | parallel -0 jq -r '.timestamp' | awk '{ rem = $1 % 60; minute = $1 - rem; print minute }' | sort | uniq -c | awk '{ avg = int($1 / 60); print avg }' | sort -nr | uniq -c
