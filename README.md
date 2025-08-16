<div align="center">
<h1>MeiliBridge</h1>
</div>

<div align="center">

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Docker](https://img.shields.io/badge/docker-%230db7ed.svg?style=flat&logo=docker&logoColor=white)](https://hub.docker.com/r/binarytouch/meilibridge)
[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![codecov](https://codecov.io/github/binary-touch/meilibridge/graph/badge.svg?token=E8W87QQO3G)](https://codecov.io/github/binary-touch/meilibridge)
[![GitHub Release](https://img.shields.io/github/v/release/binary-touch/meilibridge?style=flat&logo=github)](https://github.com/binary-touch/meilibridge/releases/latest)

**Lightning-fast PostgreSQL to Meilisearch sync engine**

Real-time data synchronization with automatic retries, parallel processing, and zero downtime

[Features](#-features) • [Quick Start](#-quick-start) • [Configuration](#-configuration) • [Contributing](#-contributing) • [Documentation](docs/)

</div>

---

### Core Capabilities

- **🚄 Real-time CDC** - Sub-second data synchronization using PostgreSQL logical replication
- **⚡ High Performance** - Process 10,000+ events/second with parallel work-stealing architecture
- **🔄 Automatic Recovery** - Built-in retry mechanisms with exponential backoff and circuit breakers
- **💾 Persistent State** - Redis-based checkpointing for seamless restarts and recovery
- **📊 Production Ready** - Comprehensive metrics, health checks, and monitoring integrations
- **🎯 Flexible Mapping** - Transform, filter, and enrich data with powerful pipeline configuration
- **🔌 Extensible** - Plugin system for custom transformations and data processing

### Data Integrity & Reliability

- **✅ Exactly-Once Delivery** - Transaction-based checkpointing with event deduplication
- **🔐 Atomic Operations** - Two-phase commit protocol ensures data consistency
- **🗄️ Multi-Source Support** - Sync from multiple PostgreSQL databases simultaneously
- **🗑️ Soft Delete Handling** - Configurable detection and transformation of soft deletes
- **📦 Dead Letter Queue** - Automatic handling of failed events with retry policies
- **🔍 Snapshot Isolation** - Consistent reads during full table synchronization

### Performance Optimization

- **📈 Adaptive Batching** - Dynamic batch sizing based on workload and latency
- **🧠 Smart Work Stealing** - Automatic load balancing across parallel workers
- **💪 Connection Pooling** - Optimized connection management for high throughput
- **🚦 Memory Efficient** - Streaming processing with bounded memory usage
- **⏱️ Sub-100ms P50 Latency** - Optimized for real-time synchronization

### Operations & Monitoring

- **📡 Prometheus Metrics** - Comprehensive metrics for monitoring and alerting
- **🔧 REST API** - Full management API for runtime control and diagnostics
- **🏥 Health Checks** - Liveness and readiness probes for container orchestration
- **📋 Event Replay** - Replay events from specific checkpoints for recovery
- **🔍 Diagnostic Tools** - Built-in debugging and troubleshooting endpoints
- **📚 Structured Logging** - JSON-formatted logs with correlation IDs

---

## 🚀 Quick Start

Get MeiliBridge running in under 2 minutes!

### Prerequisites

- PostgreSQL 10+ with logical replication enabled
- Meilisearch 1.0+ instance
- Docker (recommended) or Rust 1.70+ (for manual build)

### PostgreSQL Setup

Before starting MeiliBridge, prepare your PostgreSQL database:

```sql
-- 1. Enable logical replication in postgresql.conf
-- wal_level = logical
-- max_replication_slots = 4
-- max_wal_senders = 4

-- 2. Create a user with replication privileges
CREATE USER meilibridge WITH REPLICATION LOGIN PASSWORD 'your_password';

-- 3. Grant necessary permissions on your database
GRANT CONNECT ON DATABASE your_database TO meilibridge;
GRANT USAGE ON SCHEMA public TO meilibridge;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO meilibridge;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT ON TABLES TO meilibridge;

-- 4. Create publication for the tables you want to sync
CREATE PUBLICATION meilibridge_pub FOR TABLE users, products, orders;
-- Or for all tables:
-- CREATE PUBLICATION meilibridge_pub FOR ALL TABLES;
```

**Note**: MeiliBridge will automatically create the replication slot if configured with `create_slot: true`.

### Docker (Recommended)

```bash
# Pull and run with minimal config
docker run -d \
  --name meilibridge \
  -e POSTGRES_URL="postgresql://user:pass@host:5432/db" \
  -e MEILISEARCH_URL="http://localhost:7700" \
  -e MEILISEARCH_API_KEY="your-api-key" \
  -p 7701:7701 \
  binarytouch/meilibridge:latest

# Check health status
curl http://localhost:7701/health

# View logs
docker logs -f meilibridge
```

### Docker Compose (Full Stack)

```bash
# Clone the repository
git clone https://github.com/binary-touch/meilibridge.git
cd meilibridge

# Copy example environment file
cp .env.example .env

# Start PostgreSQL, Meilisearch, Redis, and MeiliBridge
docker-compose up -d

# Verify all services are running
docker-compose ps

# Check synchronization status
curl http://localhost:7701/api/v1/status
```

### Configuration File Setup

Create a `config.yaml` file:

```yaml
# Minimal configuration
source:
  type: postgresql
  postgresql:
    connection:
      host: localhost
      port: 5432
      database: myapp
      username: postgres
      password: ${POSTGRES_PASSWORD}

meilisearch:
  url: http://localhost:7700
  api_key: ${MEILI_MASTER_KEY}

sync_tasks:
  - table: users
    index: users
    primary_key: id
    full_sync_on_start: true
```

Run with configuration:

```bash
# Using Docker
docker run -d \
  --name meilibridge \
  -v $(pwd)/config.yaml:/config.yaml \
  -p 7701:7701 \
  binarytouch/meilibridge:latest --config /config.yaml

# Using binary
./meilibridge --config config.yaml
```

### Manual Installation

<details>
<summary><b>Build from Source</b></summary>

```bash
# Clone repository
git clone https://github.com/binary-touch/meilibridge.git
cd meilibridge

# Build in release mode
cargo build --release

# Run with configuration
./target/release/meilibridge --config config.yaml
```
</details>

<details>
<summary><b>Download Pre-built Binary</b></summary>

```bash
# Linux
curl -L https://github.com/binary-touch/meilibridge/releases/latest/download/meilibridge-linux-amd64 -o meilibridge

# macOS (Intel)
curl -L https://github.com/binary-touch/meilibridge/releases/latest/download/meilibridge-darwin-amd64 -o meilibridge

# macOS (Apple Silicon)
curl -L https://github.com/binary-touch/meilibridge/releases/latest/download/meilibridge-darwin-arm64 -o meilibridge

# Make executable
chmod +x meilibridge

# Run
./meilibridge --config config.yaml
```
</details>

### Verify Installation

```bash
# Check version
meilibridge --version

# Validate configuration
meilibridge validate --config config.yaml

# Generate sample configuration
meilibridge generate-sample > config.yaml

# Start with debug logging
meilibridge --config config.yaml --log-level debug
```

### Next Steps

- 📚 Read the [Getting Started Guide](docs/getting-started.md) for detailed setup
- 📊 Set up [monitoring](#monitoring--logging) with Prometheus
- 🚀 Deploy to production with our [deployment guide](docs/deployment.md)

---

## ⚙️ Configuration

Create a `config.yaml` file with your settings:

```yaml
# Basic connection settings
source:
  type: postgresql
  postgresql:
    connection:
      host: localhost
      port: 5432
      database: myapp
      username: postgres
      password: ${POSTGRES_PASSWORD}  # Environment variable support

meilisearch:
  url: http://localhost:7700
  api_key: ${MEILI_MASTER_KEY}

# Define sync tasks
sync_tasks:
  - table: users
    index: users
    primary_key: id
    full_sync_on_start: true
```

### Configuration Reference

<details>
<summary><b>Source Configuration</b></summary>

#### PostgreSQL Source

```yaml
source:
  type: postgresql
  postgresql:
    connection:
      host: localhost           # PostgreSQL host
      port: 5432               # PostgreSQL port
      database: myapp          # Database name
      username: postgres       # Username (needs REPLICATION privilege)
      password: secret         # Password (supports ${ENV_VAR})
    
    # Replication settings
    slot_name: meilibridge_slot      # Replication slot name
    publication_name: meilibridge_pub # Publication name
    create_slot: true                # Auto-create replication slot
    
    # Connection pool settings
    pool:
      max_size: 10              # Maximum connections
      min_idle: 2               # Minimum idle connections
      acquire_timeout: 30       # Connection acquire timeout (seconds)
      idle_timeout: 600         # Idle connection timeout (seconds)
      max_lifetime: 1800        # Maximum connection lifetime (seconds)
    
    # Statement cache
    statement_cache:
      enabled: true             # Enable prepared statement caching
      max_size: 100            # Maximum cached statements
```
</details>

<details>
<summary><b>Destination Configuration</b></summary>

#### Meilisearch Destination

```yaml
meilisearch:
  url: http://localhost:7700    # Meilisearch URL
  api_key: masterKey           # API key (supports ${ENV_VAR})
  timeout: 30                  # Request timeout (seconds)
  
  # Retry settings
  max_retries: 3               # Maximum retry attempts
  retry_on_timeout: true       # Retry on timeout errors
  auto_create_index: true      # Auto-create missing indexes
  primary_key: id              # Default primary key field
  
  # Circuit breaker (fault tolerance)
  circuit_breaker:
    enabled: true              # Enable circuit breaker
    failure_threshold: 5       # Failures before opening
    reset_timeout: 60          # Reset timeout (seconds)
```
</details>

<details>
<summary><b>Sync Task Configuration</b></summary>

```yaml
sync_tasks:
  - id: users_sync              # Unique task ID
    table: public.users         # Source table (schema.table)
    index: users                # Target Meilisearch index
    primary_key: id             # Primary key field
    
    # Sync behavior
    full_sync_on_start: true    # Perform full sync on startup
    auto_start: true            # Auto-start this task
    
    # Filtering
    filter:
      event_types: [create, update, delete]  # Event types to process
      conditions:
        - field: deleted
          op: not_equals
          value: true           # Skip soft-deleted records
    
    # Field transformations
    transform:
      fields:
        email:
          type: lowercase       # Convert email to lowercase
        full_name:
          type: compute
          expression: "concat(first_name, ' ', last_name)"
    
    # Field mapping
    mapping:
      fields:
        user_id: id            # Rename user_id to id
        created_at: created_timestamp
      unmapped_fields_strategy: include  # include/exclude/prefix
    
    # Processing options
    options:
      batch_size: 1000          # Events per batch
      batch_timeout_ms: 1000    # Batch timeout (milliseconds)
      retry:
        max_retries: 3          # Max retry attempts
        initial_delay: 1000     # Initial retry delay (ms)
        max_delay: 60000        # Maximum retry delay (ms)
        multiplier: 2.0         # Backoff multiplier
```
</details>

<details>
<summary><b>Advanced Configuration</b></summary>

#### Redis Configuration (Optional)

```yaml
redis:
  url: redis://localhost:6379   # Redis URL
  password: ${REDIS_PASSWORD}   # Redis password
  database: 0                   # Redis database number
  key_prefix: meilibridge       # Key prefix for all keys
  
  pool:
    max_size: 10               # Maximum connections
    min_idle: 2                # Minimum idle connections
    connection_timeout: 5      # Connection timeout (seconds)
```

#### Performance Tuning

```yaml
performance:
  parallel_processing:
    enabled: true              # Enable parallel processing
    workers_per_table: 4       # Worker threads per table
    max_concurrent_events: 100 # Max concurrent events
    work_stealing: true        # Enable work stealing
    work_steal_interval_ms: 50 # Work steal check interval
  
  buffer_size: 10000           # Event buffer size
  checkpoint_interval: 10      # Checkpoint save interval (seconds)
```

#### API Server Configuration

```yaml
api:
  enabled: true                # Enable REST API
  host: 0.0.0.0               # API host
  port: 7701                  # API port
  
  cors:
    enabled: true              # Enable CORS
    origins: ["*"]             # Allowed origins
  
  auth:
    enabled: false             # Enable authentication
    type: bearer               # Auth type (bearer)
    tokens:
      - name: admin
        token: ${API_TOKEN}    # Admin token
        role: admin
```

#### Monitoring & Logging

```yaml
logging:
  level: info                  # Log level (trace/debug/info/warn/error)
  format: pretty               # Log format (pretty/json)
  
metrics:
  enabled: true                # Enable Prometheus metrics
  port: 9090                  # Metrics port
  path: /metrics              # Metrics endpoint

features:
  auto_recovery: true         # Auto-recover from failures
  health_checks: true         # Enable health endpoints
  distributed_mode: false     # Enable distributed mode
```
</details>

### Environment Variables

All configuration values support environment variable substitution:

```yaml
password: ${POSTGRES_PASSWORD}
api_key: ${MEILI_MASTER_KEY:-default_value}  # With default
```

Common environment variables:
- `MEILIBRIDGE_CONFIG` - Config file path
- `MEILIBRIDGE_LOG_LEVEL` - Log level
- `POSTGRES_PASSWORD` - PostgreSQL password
- `MEILI_MASTER_KEY` - Meilisearch API key
- `REDIS_PASSWORD` - Redis password

---

## 🔧 Command Line Options

```bash
meilibridge [OPTIONS] [COMMAND]

OPTIONS:
    -c, --config <FILE>      Configuration file path
    -l, --log-level <LEVEL>  Log level (trace/debug/info/warn/error)
    -h, --help              Print help information
    -V, --version           Print version information

COMMANDS:
    run             Run the synchronization service (default)
    validate        Validate configuration file
    generate-sample Generate sample configuration
    version         Show version information
```

---

## 🤝 Contributing

We love contributions! Whether you're fixing bugs, adding features, or improving documentation, we appreciate your help.

### Quick Contribution Guide

1. **Fork & Clone**
   ```bash
   git clone https://github.com/YOUR_USERNAME/meilibridge.git
   cd meilibridge
   ```

2. **Set Up Development Environment**
   ```bash
   # Install Rust
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Install dependencies
   cargo build
   
   # Run tests
   cargo test
   ```

3. **Make Your Changes**
   - Create a feature branch: `git checkout -b feature/amazing-feature`
   - Make your changes
   - Add tests if applicable
   - Ensure all tests pass: `cargo test`

4. **Submit Pull Request**
   - Commit your changes: `git commit -m 'Add amazing feature'`
   - Push to your fork: `git push origin feature/amazing-feature`
   - Open a pull request

### Development Resources

- 📖 [Architecture Overview](docs/configuration-architecture.md) - Understand the codebase
- 🔧 [API Development Guide](docs/api-development.md) - Add new API endpoints
- 🔄 [CDC Operations Guide](docs/cdc-operations.md) - Work with CDC components
- 🗺️ [Roadmap](tasks/roadmap.md) - See what's planned

### Code of Conduct

Please note that this project is released with a [Contributor Code of Conduct](CODE_OF_CONDUCT.md). By participating in this project you agree to abide by its terms.

---

## 📚 Documentation

- **[Getting Started Guide](docs/getting-started.md)** - Detailed installation and setup
- **[Configuration & Architecture](docs/configuration-architecture.md)** - Deep dive into configuration
- **[API Reference](docs/api-development.md)** - REST API documentation
- **[CDC Operations](docs/cdc-operations.md)** - CDC setup and troubleshooting
- **[Roadmap](tasks/roadmap.md)** - Future plans and features

---

## 🆘 Getting Help

- 📋 [GitHub Issues](https://github.com/binary-touch/meilibridge/issues) - Report bugs or request features
- 💬 [Discussions](https://github.com/binary-touch/meilibridge/discussions) - Ask questions and share ideas
- 📧 Email: support@meilibridge.dev

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

<div align="center">
Made with ❤️ in India
</div>
