#!/bin/sh
set -eu

SCRIPT_DIR=$(CDPATH='' cd -- "$(dirname "$0")" && pwd)
REPO_ROOT=$(CDPATH='' cd -- "$SCRIPT_DIR/.." && pwd)
cd "$REPO_ROOT"

if ! command -v cargo-binstall >/dev/null 2>&1; then
    curl -L --proto '=https' --tlsv1.2 -sSf \
        https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh \
        | bash
fi

cargo binstall --no-confirm bpf-linker

if ! command -v aya-tool >/dev/null 2>&1; then
    cargo install --git https://github.com/aya-rs/aya -- aya-tool
fi

aya-tool generate task_struct > ebpf/src/vmlinux.rs
