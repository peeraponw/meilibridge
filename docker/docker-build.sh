#!/bin/bash
set -e

# Docker build script for MeiliBridge
# Usage: ./docker/docker-build.sh [tag]

# Configuration
DOCKER_REGISTRY="${DOCKER_REGISTRY:-docker.io}"
DOCKER_USERNAME="${DOCKER_USERNAME:-meilibridge}"
IMAGE_NAME="${IMAGE_NAME:-meilibridge}"
DEFAULT_TAG="${DEFAULT_TAG:-latest}"

# Get version from Cargo.toml (from parent directory)
VERSION=$(grep '^version' ../Cargo.toml | head -1 | cut -d'"' -f2)

# Use provided tag or default
TAG="${1:-$DEFAULT_TAG}"

# Build tags
FULL_IMAGE_NAME="${DOCKER_REGISTRY}/${DOCKER_USERNAME}/${IMAGE_NAME}"
VERSION_TAG="${FULL_IMAGE_NAME}:${VERSION}"
LATEST_TAG="${FULL_IMAGE_NAME}:latest"
CUSTOM_TAG="${FULL_IMAGE_NAME}:${TAG}"

echo "Building MeiliBridge Docker image..."
echo "Registry: ${DOCKER_REGISTRY}"
echo "Image: ${FULL_IMAGE_NAME}"
echo "Version: ${VERSION}"
echo "Tags: ${VERSION_TAG}, ${LATEST_TAG}"

# Build the image (from parent directory)
cd .. && docker build \
  --build-arg BUILD_DATE=$(date -u +'%Y-%m-%dT%H:%M:%SZ') \
  --build-arg VERSION="${VERSION}" \
  --build-arg VCS_REF=$(git rev-parse --short HEAD) \
  -t "${VERSION_TAG}" \
  -t "${LATEST_TAG}" \
  -f docker/Dockerfile \
  .

# Tag with custom tag if different from version
if [ "${TAG}" != "${VERSION}" ] && [ "${TAG}" != "latest" ]; then
  docker tag "${VERSION_TAG}" "${CUSTOM_TAG}"
  echo "Tagged as: ${CUSTOM_TAG}"
fi

echo "Build complete!"
echo ""
echo "To push to Docker Hub, run:"
echo "  ./docker/docker-push.sh"
echo ""
echo "To run locally:"
echo "  docker run -v \$(pwd)/config.yaml:/etc/meilibridge/config.yaml ${VERSION_TAG}"