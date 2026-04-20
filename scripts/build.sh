#!/bin/bash
set -euxo pipefail

cargo build --workspace "$@"