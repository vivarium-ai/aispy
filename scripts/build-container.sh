#!/bin/sh
set -eu

SCRIPT_DIR=$(CDPATH='' cd -- "$(dirname "$0")" && pwd)
REPO_ROOT=$(CDPATH='' cd -- "$SCRIPT_DIR/.." && pwd)
cd "$REPO_ROOT"

DOCKER_IMAGE="${docker_image:-ghcr.io/vivarium-ai/aispy}"
GIT_TAG="${GIT_TAG:-dev}"

docker buildx build \
  --platform linux/amd64 \
  --label "org.opencontainers.image.source=https://github.com/vivarium-ai/aispy" \
  --label "org.opencontainers.image.revision=${REVISION}" \
  --label "org.opencontainers.image.version=${GIT_TAG}" \
  --tag "${DOCKER_IMAGE}:${GIT_TAG}" \
  --tag "${DOCKER_IMAGE}:latest" \
  --load \
  .
