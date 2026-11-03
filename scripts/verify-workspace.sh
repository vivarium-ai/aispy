#!/bin/sh
set -eu

SCRIPT_DIR=$(CDPATH='' cd -- "$(dirname "$0")" && pwd)
REPO_ROOT=$(CDPATH='' cd -- "$SCRIPT_DIR/.." && pwd)
cd "$REPO_ROOT"

# Validate manifests
cargo metadata --format-version 1 > /dev/null
cargo +nightly metadata \
  --manifest-path ebpf/Cargo.toml \
  --format-version 1 > /dev/null

# Check formatting
cargo fmt --all --check
cargo fmt --manifest-path ebpf/Cargo.toml --check

# Check workspace targets
cargo check --workspace --all-targets --all-features

# Check eBPF target
cargo +nightly check --manifest-path ebpf/Cargo.toml \
  --target bpfel-unknown-none \
  -Z build-std=core
