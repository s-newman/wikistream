#!/bin/bash
set -euo pipefail

if [ $# -ne 2 ]; then
  echo "USAGE: $0 NAME DOCKERFILE-NAME" >&2
  echo "example:"
  echo "$0 ws-sse-cli cli.Dockerfile"
  exit 1
fi

NAME="${1}"
DOCKERFILE="configuration/docker/${2}"
WS_VERSION="$(scripts/get-tag.sh)"

docker build \
  -t "${NAME}:latest" \
  -t "${NAME}:${WS_VERSION}" \
  -f "${DOCKERFILE}" \
  .