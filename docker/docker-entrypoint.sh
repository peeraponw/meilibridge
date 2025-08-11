#!/bin/bash
set -e

# Docker entrypoint for MeiliBridge
# Handles environment variable configuration for ports

# Set the API port via environment variable if provided
if [ -n "${MEILIBRIDGE_API_PORT}" ]; then
    export MEILIBRIDGE__API__PORT="${MEILIBRIDGE_API_PORT}"
fi

# Set the metrics port via environment variable if provided (for future use)
if [ -n "${MEILIBRIDGE_METRICS_PORT}" ]; then
    export MEILIBRIDGE__METRICS__PORT="${MEILIBRIDGE_METRICS_PORT}"
fi

# Allow setting the bind address
if [ -n "${MEILIBRIDGE_API_HOST}" ]; then
    export MEILIBRIDGE__API__HOST="${MEILIBRIDGE_API_HOST}"
fi

# Log the configuration
echo "Starting MeiliBridge with configuration:"
echo "  API Host: ${MEILIBRIDGE__API__HOST:-0.0.0.0}"
echo "  API Port: ${MEILIBRIDGE__API__PORT:-7701}"
echo "  Metrics Port: ${MEILIBRIDGE__METRICS__PORT:-7702} (future)"

# Execute MeiliBridge with all arguments passed to the entrypoint
exec meilibridge "$@"