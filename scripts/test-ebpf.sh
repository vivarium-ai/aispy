#!/bin/sh
set -eu

SCRIPT_DIR=$(CDPATH='' cd -- "$(dirname "$0")" && pwd)
REPO_ROOT=$(CDPATH='' cd -- "$SCRIPT_DIR/.." && pwd)
cd "$REPO_ROOT"

cargo test \
    --package aispy-ebpf-common

cargo +nightly check \
    --manifest-path ebpf/Cargo.toml \
    --target bpfel-unknown-none \
    -Z build-std=core

# Build the actual kernel program.
cargo +nightly build \
    --manifest-path ebpf/Cargo.toml \
    --target bpfel-unknown-none \
    -Z build-std=core \
    --release

# Run the real Linux/kernel integration test.
cargo test --test ebpf --no-run

TEST_BIN=$(
    cargo test --test ebpf --no-run --message-format=json |
    jq -r 'select(.profile.test == true) | .executable' |
    tail -1
)

sudo "$TEST_BIN" --ignored --nocapture
