#!/usr/bin/env bash
# Docker wrapper for convert-to-md-rs
#
# Usage:
#   ./scripts/convert.sh
#   ./scripts/convert.sh --list
#   ./scripts/convert.sh --overwrite
#   ./scripts/convert.sh --build           # force rebuild image
#   ./scripts/convert.sh --input /path/to/file.pdf
#
# Volume mounts (defaults):
#   Host ./resources/               → /app/resources      (input)
#   Host ./data/output/markdown/    → /app/data/output/markdown  (output)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
IMAGE_NAME="convert-to-md-rs"
DOCKERFILE="${PROJECT_ROOT}/Dockerfile"

build_image() {
    echo "[INFO] Building Docker image: ${IMAGE_NAME}"
    docker build -f "${DOCKERFILE}" -t "${IMAGE_NAME}" "${PROJECT_ROOT}"
    echo "[INFO] Build complete."
}

# --build flag: force rebuild then exit (if no other args) or continue
if [[ "${1:-}" == "--build" ]]; then
    build_image
    shift
    [[ $# -eq 0 ]] && exit 0
fi

# Auto-build if image not present
if ! docker image inspect "${IMAGE_NAME}" &>/dev/null; then
    build_image
fi

# Prepare output directory
mkdir -p "${PROJECT_ROOT}/resources"
mkdir -p "${PROJECT_ROOT}/data/output/markdown"

exec docker run --rm \
    -v "${PROJECT_ROOT}/resources:/app/resources:ro" \
    -v "${PROJECT_ROOT}/data/output/markdown:/app/data/output/markdown" \
    "${IMAGE_NAME}" \
    "$@"
