#!/bin/sh
set -eu

SCRIPT_DIR=$(CDPATH='' cd -- "$(dirname "$0")" && pwd)
REPO_ROOT=$(CDPATH='' cd -- "$SCRIPT_DIR/../.." && pwd)
cd "$REPO_ROOT"

# Validate manifests
cargo metadata --format-version 1 > /dev/null

# Check formatting
cargo fmt --all --check

# Check all workspace targets
cargo check --workspace --all-targets --all-features
