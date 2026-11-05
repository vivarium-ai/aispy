#!/bin/sh
set -eu

SCRIPT_DIR=$(CDPATH='' cd -- "$(dirname "$0")" && pwd)
REPO_ROOT=$(CDPATH='' cd -- "$SCRIPT_DIR/.." && pwd)
cd "$REPO_ROOT"

cargo build --lib --package aispy
cargo build --lib --package aispy-observe
cargo build --lib --package aispy-ebpf-common
