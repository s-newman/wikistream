#!/bin/bash
set -euxo pipefail

WS_VERSION="$(scripts/get-tag.sh)"
export WS_VERSION
cargo build --workspace "$@"