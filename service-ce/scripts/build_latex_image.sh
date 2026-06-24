#!/bin/bash
set -e

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
IMAGE_NAME="pexm/latex"

echo "==> Building Docker image: $IMAGE_NAME"
cd "$ROOT_DIR/service-ce/latex-docker"
podman build --no-cache -t "$IMAGE_NAME" .

echo "==> Done. Image $IMAGE_NAME built successfully."
