#!/bin/bash
set -e

# Docker push script for MeiliBridge
# Usage: ./docker/docker-push.sh [tag]

# Configuration
DOCKER_REGISTRY="${DOCKER_REGISTRY:-docker.io}"
DOCKER_USERNAME="${DOCKER_USERNAME:-meilibridge}"
IMAGE_NAME="${IMAGE_NAME:-meilibridge}"

# Get version from Cargo.toml (from parent directory)
VERSION=$(grep '^version' ../Cargo.toml | head -1 | cut -d'"' -f2)

# Use provided tag or version
TAG="${1:-$VERSION}"

# Build tags
FULL_IMAGE_NAME="${DOCKER_REGISTRY}/${DOCKER_USERNAME}/${IMAGE_NAME}"
VERSION_TAG="${FULL_IMAGE_NAME}:${VERSION}"
LATEST_TAG="${FULL_IMAGE_NAME}:latest"
CUSTOM_TAG="${FULL_IMAGE_NAME}:${TAG}"

echo "Pushing MeiliBridge to Docker Hub..."
echo "Registry: ${DOCKER_REGISTRY}"
echo "Image: ${FULL_IMAGE_NAME}"

# Check if logged in
if ! docker info 2>/dev/null | grep -q "Username: ${DOCKER_USERNAME}"; then
  echo "Not logged in to Docker Hub. Please run:"
  echo "  docker login -u ${DOCKER_USERNAME}"
  exit 1
fi

# Push version tag
echo "Pushing ${VERSION_TAG}..."
docker push "${VERSION_TAG}"

# Push latest tag
echo "Pushing ${LATEST_TAG}..."
docker push "${LATEST_TAG}"

# Push custom tag if different
if [ "${TAG}" != "${VERSION}" ] && [ "${TAG}" != "latest" ]; then
  echo "Pushing ${CUSTOM_TAG}..."
  docker push "${CUSTOM_TAG}"
fi

echo ""
echo "Push complete!"
echo ""
echo "Images available at:"
echo "  ${VERSION_TAG}"
echo "  ${LATEST_TAG}"
if [ "${TAG}" != "${VERSION}" ] && [ "${TAG}" != "latest" ]; then
  echo "  ${CUSTOM_TAG}"
fi
echo ""
echo "To pull and run:"
echo "  docker pull ${VERSION_TAG}"
echo "  docker run -v \$(pwd)/config.yaml:/etc/meilibridge/config.yaml ${VERSION_TAG}"