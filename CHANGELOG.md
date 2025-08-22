# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.5] - 2025-08-22

## [0.1.4] - 2025-08-16

### Changed
- **BREAKING**: Changed default API port from 8080 to 7701 to align with Meilisearch ecosystem
- **BREAKING**: Changed metrics port from 9090 to 7702 (future implementation)
- Updated all configuration files and documentation to reflect new port scheme
- Reserved ports 7703-7710 for future MeiliBridge features

### Added
- Port allocation documentation (`docs/port-allocation.md`)
- Docker environment variable support for dynamic port configuration
  - `MEILIBRIDGE_API_PORT` - Configure API port (default: 7701)
  - `MEILIBRIDGE_API_HOST` - Configure API host (default: 0.0.0.0)
  - `MEILIBRIDGE_METRICS_PORT` - Configure metrics port (default: 7702, future)
- Docker entrypoint script for simplified environment variable handling
- WebSocket API port reservation (7703) for future real-time streaming
- gRPC API port reservation (7704) for future high-performance protocol
- Admin UI port reservation (7705) for future web interface
- Debug API port reservation (7706) for future debugging endpoints
- Replication port reservation (7707) for future multi-instance coordination
- CDC Stream port reservation (7708) for future direct event consumption
- Plugin API port reservation (7709) for future plugin communication
- CDC implementation documentation (`docs/cdc-implementation.md`)
- Auto-create index feature in Meilisearch adapter
  - New configuration option `meilisearch.auto_create_index` (default: true)
  - Automatically creates missing indexes with appropriate primary keys
- Per-table primary key support for delete operations

### Fixed
- Fixed pgoutput binary protocol incompatibility by switching to test_decoding plugin
- Fixed PostgreSQL type conversions for pg_lsn and xid types (cast to ::text)
- Fixed automatic slot recreation when switching from pgoutput to test_decoding
- Fixed handling of existing publications and replication slots
- Added connection keepalive settings for replication connections
- Added retry logic with exponential backoff for failed replication queries
- Implemented CDC coordinator to fix slot contention issue
  - Single coordinator consumes from replication slot
  - Distributes events to appropriate sync tasks based on table name
  - Eliminates "shutdown signal" errors from competing consumers
- Fixed CDC event processing pipeline not reaching Meilisearch
  - Batch processor now applies batches immediately instead of waiting for batch size
  - CDC events are now successfully synchronized to Meilisearch in real-time
  - Added comprehensive debug logging throughout the CDC pipeline

### Known Issues
- Multiple ReplicationConsumer instances may create slot contention warnings
  - This doesn't affect functionality but creates noise in logs
- Prepared statement accumulation needs cache management

### Migration Guide

#### Port Migration
To migrate from the old port scheme (8080) to the new one (7701):

1. Update your `config.yaml`:
   ```yaml
   api:
     port: 7701  # Changed from 8080
   ```

2. Update any firewall rules or security groups to allow port 7701

3. Update any monitoring, alerting, or client configurations that reference port 8080

4. If using Docker, update port mappings:
   ```yaml
   ports:
     - "7701:7701"  # Changed from 8080:8080
   ```

5. Update health check URLs:
   ```bash
   # Old
   curl http://localhost:8080/health
   
   # New
   curl http://localhost:7701/health
   ```

#### Using Custom Ports in Docker

If you need to keep using port 8080 or any other custom port:

```bash
# Run with custom port
docker run \
  -e MEILIBRIDGE_API_PORT=8080 \
  -p 8080:8080 \
  meilibridge:latest

# Or in docker-compose.yml
services:
  meilibridge:
    environment:
      - MEILIBRIDGE_API_PORT=8080
    ports:
      - "8080:8080"
```