# Docker Setup for MeiliBridge

This directory contains all Docker-related files for MeiliBridge.

## Files

- **Dockerfile** - Multi-stage Docker image for MeiliBridge
- **compose.yaml** - Docker Compose configuration for local development
- **docker-compose.override.example.yml** - Example override file for custom configurations
- **docker-entrypoint.sh** - Entrypoint script for the Docker container
- **docker-build.sh** - Script to build Docker images
- **docker-push.sh** - Script to push images to Docker Hub

## Quick Start

### Development Environment

From the project root directory:

```bash
# Start all services
docker compose -f docker/compose.yaml up -d

# View logs
docker compose -f docker/compose.yaml logs -f

# Stop services
docker compose -f docker/compose.yaml down
```

Or use the Makefile shortcuts:

```bash
make docker-up
make docker-logs
make docker-down
```

### Building the Docker Image

```bash
# From project root
docker build -f docker/Dockerfile -t meilibridge:latest .

# Or use the build script
./docker/docker-build.sh
```

### Custom Configuration

1. Copy the override example:
```bash
cp docker/docker-compose.override.example.yml docker/docker-compose.override.yml
```

2. Edit `docker/docker-compose.override.yml` with your custom settings

3. Docker Compose will automatically use both files

## Services

The `compose.yaml` file includes:

- **PostgreSQL** (port 5433) - With logical replication enabled
- **Meilisearch** (port 7700) - Search engine
- **Redis** (port 6380) - For distributed mode
- **Adminer** (port 8090) - Database management UI
- **RedisInsight** (port 8001) - Redis management UI
- **Meilisearch UI** (port 24900) - Meilisearch dashboard

## Environment Variables

Create a `.env` file in the project root:

```env
# PostgreSQL
POSTGRES_PASSWORD=postgres
POSTGRES_DB=meilibridge_dev

# Meilisearch
MEILI_MASTER_KEY=masterkey
MEILI_ENV=development

# Redis
REDIS_PASSWORD=redis123

# MeiliBridge
RUST_LOG=info
```

## Volumes

- `postgres_data` - PostgreSQL data
- `meilisearch_data` - Meilisearch data
- `redis_data` - Redis data

To reset all data:
```bash
docker compose -f docker/compose.yaml down -v
```

## Production Deployment

For production, build and run the image directly:

```bash
# Build
docker build -f docker/Dockerfile -t meilibridge:prod .

# Run
docker run -d \
  --name meilibridge \
  -v /path/to/config.yaml:/config.yaml \
  -e MEILIBRIDGE_CONFIG=/config.yaml \
  -p 7708:7708 \
  meilibridge:prod
```

## Docker Hub

Images are automatically published to Docker Hub on releases:
- `meilibridge/meilibridge:latest`
- `meilibridge/meilibridge:1.0.0`

Pull and run:
```bash
docker pull meilibridge/meilibridge:latest
docker run -d -v $(pwd)/config.yaml:/config.yaml meilibridge/meilibridge:latest
```