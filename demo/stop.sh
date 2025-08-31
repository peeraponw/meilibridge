#!/bin/bash

# MeiliBridge Demo Stop Script
# This script cleanly stops the demo environment

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_header() {
    echo -e "${BLUE}============================================${NC}"
    echo -e "${BLUE}    Stopping MeiliBridge Demo${NC}"
    echo -e "${BLUE}============================================${NC}"
    echo
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_info() {
    echo -e "${YELLOW}ℹ $1${NC}"
}

# Use docker-compose or docker compose based on availability
if command -v docker-compose &> /dev/null; then
    COMPOSE_CMD="docker-compose"
else
    COMPOSE_CMD="docker compose"
fi

print_header

echo "Stopping services..."
$COMPOSE_CMD down

echo
print_success "All services stopped"

echo
echo "Options:"
echo "  • To remove all data:    $COMPOSE_CMD down -v"
echo "  • To restart:            ./start.sh"
echo
print_info "Demo stopped successfully!"