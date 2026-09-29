#!/bin/sh
set -eu

SCRIPT_DIR=$(CDPATH='' cd -- "$(dirname "$0")" && pwd)
REPO_ROOT=$(CDPATH='' cd -- "$SCRIPT_DIR/../.." && pwd)
cd "$REPO_ROOT"

docker buildx build \
  --label org.opencontainers.image.source=https://github.com/vivarium-ai/cuda-oxide-kernels \
  --platform linux/amd64 \
  --file docker/Dockerfile \
  --tag ghcr.io/vivarium-ai/cuda-oxide-kernels-dev:latest \
  --load \
  .

docker push ghcr.io/vivarium-ai/cuda-oxide-kernels-dev:latest
