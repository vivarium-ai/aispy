#!/bin/sh
set -eu

SCRIPT_DIR=$(CDPATH='' cd -- "$(dirname "$0")" && pwd)
REPO_ROOT=$(CDPATH='' cd -- "$SCRIPT_DIR/.." && pwd)
cd "$REPO_ROOT"

limactl start \
  --name=aispy-amd64 \
  --vm-type=qemu \
  --arch=x86_64 \
  --mount-only "$(pwd):w" \
  template:_images/ubuntu
