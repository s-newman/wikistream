#!/bin/bash

# make globs match hidden files
shopt -s dotglob

set -euxo pipefail

cargo clippy --no-deps --all-targets -- -D warnings
cargo fmt --check
shellcheck -- **/*.sh