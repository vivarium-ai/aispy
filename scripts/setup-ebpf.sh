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

VMLINUX=ebpf/src/vmlinux.rs
TMP=$(mktemp)

aya-tool generate task_struct > "$TMP"

cat > "$VMLINUX" <<'EOF'
#![allow(
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unsafe_op_in_unsafe_fn
)]

EOF

cat "$TMP" >> "$VMLINUX"
rm "$TMP"
