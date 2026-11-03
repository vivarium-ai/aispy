#!/bin/sh
set -eu

SCRIPT_DIR=$(CDPATH='' cd -- "$(dirname "$0")" && pwd)
REPO_ROOT=$(CDPATH='' cd -- "$SCRIPT_DIR/.." && pwd)
cd "$REPO_ROOT"

cargo +nightly build \
    --manifest-path ebpf/Cargo.toml \
    --target bpfel-unknown-none \
    -Z build-std=core \
    --release
