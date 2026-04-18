#!/bin/bash
trap "echo Quitting!; exit;" SIGINT SIGTERM

_curl_ver=$(curl --version | head -n 1 | cut -d ' ' -f 1-2 | tr ' ' /)
user_agent="wikistream/1.0 (nwmn_devcontact@fastmail.com) ${_curl_ver}"

while true; do
    fname="data/stream-$(date +%s).txt"
    curl --user-agent "${user_agent}" https://stream.wikimedia.org/v2/stream/recentchange > "${fname}"
done
