#!/bin/bash

# MeiliBridge Demo Starter Script
# This script helps start and manage the demo environment

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Functions
print_header() {
    echo -e "${BLUE}============================================${NC}"
    echo -e "${BLUE}       MeiliBridge Demo Environment${NC}"
    echo -e "${BLUE}============================================${NC}"
    echo
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_info() {
    echo -e "${YELLOW}ℹ $1${NC}"
}

check_requirements() {
    echo "Checking requirements..."
    
    # Check Docker
    if command -v docker &> /dev/null; then
        print_success "Docker is installed"
    else
        print_error "Docker is not installed. Please install Docker first."
        exit 1
    fi
    
    # Check Docker Compose
    if command -v docker-compose &> /dev/null || docker compose version &> /dev/null; then
        print_success "Docker Compose is available"
    else
        print_error "Docker Compose is not available. Please install Docker Compose."
        exit 1
    fi
    
    # Check ports
    for port in 5432 7700 8080 6379 24900; do
        if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1; then
            print_error "Port $port is already in use. Please free up the port."
            exit 1
        fi
    done
    print_success "All required ports are available"
    
    echo
}

start_services() {
    echo "Starting demo services..."
    
    # Use docker-compose or docker compose based on availability
    if command -v docker-compose &> /dev/null; then
        COMPOSE_CMD="docker-compose"
    else
        COMPOSE_CMD="docker compose"
    fi
    
    $COMPOSE_CMD up -d
    
    echo
    print_info "Waiting for services to initialize (30 seconds)..."
    
    # Wait with progress indicator
    for i in {1..30}; do
        printf "\r${YELLOW}Progress: [%-30s] %d%%${NC}" "$(printf '#%.0s' $(seq 1 $i))" "$((i * 100 / 30))"
        sleep 1
    done
    echo
    echo
    
    # Check service health
    echo "Checking service health..."
    
    # Check PostgreSQL
    if $COMPOSE_CMD exec -T postgres pg_isready -U postgres &> /dev/null; then
        print_success "PostgreSQL is ready"
    else
        print_error "PostgreSQL is not ready"
    fi
    
    # Check Meilisearch
    if curl -s http://localhost:7700/health | grep -q "available"; then
        print_success "Meilisearch is ready"
    else
        print_error "Meilisearch is not ready"
    fi
    
    # Check Redis
    if $COMPOSE_CMD exec -T redis redis-cli ping | grep -q "PONG"; then
        print_success "Redis is ready"
    else
        print_error "Redis is not ready"
    fi
    
    # Check MeiliBridge
    if curl -s http://localhost:8080/health | grep -q "healthy"; then
        print_success "MeiliBridge is ready"
    else
        print_error "MeiliBridge is not ready"
    fi
    
    echo
}

show_demo_info() {
    echo -e "${BLUE}Demo Environment Ready!${NC}"
    echo
    echo "Service URLs:"
    echo "  • MeiliBridge API: http://localhost:8080"
    echo "  • Meilisearch API: http://localhost:7700 (API key: masterKey123)"
    echo "  • Meilisearch UI:  http://localhost:24900"
    echo "  • PostgreSQL:      localhost:5432 (user: postgres, password: postgres)"
    echo "  • Redis:           localhost:6379"
    echo
    echo "Useful Commands:"
    echo "  • View logs:         docker compose logs -f meilibridge"
    echo "  • Check sync status: curl http://localhost:8080/tasks"
    echo "  • View metrics:      curl http://localhost:8080/metrics"
    echo "  • Search products:   curl -X POST 'http://localhost:7700/indexes/products/search' \\"
    echo "                         -H 'Authorization: Bearer masterKey123' \\"
    echo "                         -H 'Content-Type: application/json' \\"
    echo "                         -d '{\"q\":\"laptop\",\"offset\":0,\"limit\":20}'"
    echo
    echo "Demo Features:"
    echo "  ✓ 1000+ sample products loaded"
    echo "  ✓ Real-time CDC streaming active"
    echo "  ✓ Automatic data generator running"
    echo "  ✓ Full monitoring and metrics enabled"
    echo
    echo -e "${GREEN}To stop the demo, run: ./stop.sh${NC}"
}

# Main execution
print_header
check_requirements
start_services
show_demo_info