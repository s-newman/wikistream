#!/bin/bash
set -euo pipefail

if [ $# -ne 1 ]; then
  echo "USAGE: $0 IMAGE_NAME_NO_TAG" >&2
  exit 1
fi

IMAGE_NAME="${1}"
VERSION="$(scripts/get-tag.sh)"
LONG_NAME="${CI_REGISTRY_IMAGE}/${IMAGE_NAME}"

# Load the image from file (assuming the file includes the :latest image)
docker load --input "${IMAGE_NAME}.tar"

# Retag with :latest and the version string
docker tag "${IMAGE_NAME}:latest" "${LONG_NAME}:latest"
docker tag "${IMAGE_NAME}:latest" "${LONG_NAME}:${VERSION}"

# Push retagged images to registry
docker image push "${LONG_NAME}:latest"
docker image push "${LONG_NAME}:${VERSION}"