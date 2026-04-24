#!/bin/bash

if [ $# -ne 2 ]; then
  "USAGE: $0 LINE FILE" >&2
  exit 1
fi

sed -n "${1}p" "${2}" | jq