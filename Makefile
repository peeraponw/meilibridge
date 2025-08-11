.PHONY: help build run test clean docker-up docker-down docker-logs docker-reset dev

# Default target
help:
	@echo "MeiliBridge Development Commands:"
	@echo "  make build        - Build the project in release mode"
	@echo "  make dev          - Build the project in debug mode"
	@echo "  make run          - Run MeiliBridge with local config"
	@echo "  make test         - Run all tests"
	@echo "  make clean        - Clean build artifacts"
	@echo ""
	@echo "Docker Commands:"
	@echo "  make docker-up    - Start all Docker services"
	@echo "  make docker-down  - Stop all Docker services"
	@echo "  make docker-logs  - View logs from all services"
	@echo "  make docker-reset - Reset all data (WARNING: destructive)"
	@echo ""
	@echo "Quick Start:"
	@echo "  make docker-up    - Start services"
	@echo "  make dev          - Build MeiliBridge"
	@echo "  make run          - Run MeiliBridge"

# Build commands
build:
	cargo build --release

dev:
	cargo build

run:
	cargo run -- --config config.yaml

test:
	cargo test

clean:
	cargo clean
	rm -rf logs/

# Docker commands
docker-up:
	@echo "Starting Docker services..."
	docker compose -f docker/compose.yaml up -d
	@echo "Waiting for services to be healthy..."
	@sleep 5
	@docker compose -f docker/compose.yaml ps
	@echo ""
	@echo "Services are running:"
	@echo "  PostgreSQL:      localhost:5433"
	@echo "  Redis:           localhost:6380"
	@echo "  Meilisearch:     http://localhost:7700"
	@echo "  Meilisearch UI:  http://localhost:24900"
	@echo "  Adminer:         http://localhost:8090"
	@echo "  RedisInsight:    http://localhost:8001"
	@echo ""
	@echo "Run 'make run' to start MeiliBridge"

docker-down:
	docker compose -f docker/compose.yaml down

docker-logs:
	docker compose -f docker/compose.yaml logs -f

docker-reset:
	@echo "WARNING: This will delete all Docker volumes and data!"
	@echo "Press Ctrl+C to cancel, or wait 5 seconds to continue..."
	@sleep 5
	docker compose -f docker/compose.yaml down -v
	@echo "All data has been reset"

# Combined commands
quick-start: docker-up dev run

# Development helpers
watch:
	cargo watch -x run

fmt:
	cargo fmt

lint:
	cargo clippy -- -D warnings

# Check if services are healthy
check-services:
	@echo "Checking service health..."
	@curl -s http://localhost:7700/health > /dev/null && echo "✓ Meilisearch is healthy" || echo "✗ Meilisearch is not responding"
	@docker exec meilibridge-postgres pg_isready > /dev/null 2>&1 && echo "✓ PostgreSQL is healthy" || echo "✗ PostgreSQL is not responding"
	@docker exec meilibridge-redis redis-cli ping > /dev/null 2>&1 && echo "✓ Redis is healthy" || echo "✗ Redis is not responding"

# Generate sample data
sample-data:
	@echo "Inserting sample data into PostgreSQL..."
	@docker exec meilibridge-postgres psql -U postgres -d meilibridge_dev -c "INSERT INTO users (email, first_name, last_name, age, metadata) VALUES ('alice.johnson@example.com', 'Alice', 'Johnson', 28, '{\"interests\": [\"art\", \"design\"]}'), ('charlie.brown@example.com', 'Charlie', 'Brown', 42, '{\"interests\": [\"finance\", \"investing\"]}'), ('diana.prince@example.com', 'Diana', 'Prince', 33, '{\"interests\": [\"fitness\", \"travel\"]}') ON CONFLICT (email) DO NOTHING;"
	@docker exec meilibridge-postgres psql -U postgres -d meilibridge_dev -c "INSERT INTO products (sku, name, description, price, category, tags) VALUES ('DESK-001', 'Standing Desk', 'Adjustable height standing desk', 599.99, 'Furniture', ARRAY['desk', 'office', 'ergonomic']), ('MOUSE-001', 'Wireless Mouse', 'Ergonomic wireless mouse', 49.99, 'Electronics', ARRAY['mouse', 'wireless', 'peripherals']), ('CHAIR-001', 'Ergonomic Chair', 'High-back office chair', 399.99, 'Furniture', ARRAY['chair', 'office', 'ergonomic']) ON CONFLICT (sku) DO NOTHING;"
	@echo "Sample data inserted!"

# Validate configuration
validate:
	cargo run -- validate

# Show current sync status
status:
	@echo "Checking MeiliBridge status..."
	@curl -s http://localhost:7701/api/status || echo "API server not running"