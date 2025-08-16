# Configuration & Architecture Reference

This document provides a comprehensive reference for MeiliBridge configuration and system architecture.

## Table of Contents
- [System Architecture](#system-architecture)
- [Configuration Reference](#configuration-reference)
- [Data Models](#data-models)
- [Performance Tuning](#performance-tuning)
- [Security](#security)
- [High Availability](#high-availability)

## System Architecture

### Overview

MeiliBridge is designed as a high-performance, async-first data synchronization service that streams changes from PostgreSQL to Meilisearch in real-time.

```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│   PostgreSQL    │────▶│   MeiliBridge    │────▶│   Meilisearch   │
│   (Source DB)   │ CDC │    Core Engine   │     │  (Search Index) │
└─────────────────┘     └──────────────────┘     └─────────────────┘
                               │      │
                               │      │
                        ┌──────▼──┐ ┌─▼───────┐
                        │  Redis  │ │   API   │
                        │ Storage │ │ Server  │
                        └─────────┘ └─────────┘
```

### Core Components

#### 1. Configuration Manager
- Hot-reloading support
- Environment variable interpolation
- Schema validation
- Multi-source configuration merging

#### 2. Source Adapter System
Trait-based adapter pattern for extensibility:
```rust
#[async_trait]
pub trait SourceAdapter: Send + Sync {
    async fn connect(&mut self) -> Result<()>;
    async fn get_changes(&mut self) -> Result<ChangeStream>;
    async fn get_full_data(&self, table: &str, batch_size: usize) -> Result<DataStream>;
    async fn get_current_position(&self) -> Result<Position>;
    async fn acknowledge(&mut self, position: Position) -> Result<()>;
}
```

#### 3. Event Processing Pipeline
Multi-stage pipeline with backpressure:
1. **Ingestion**: Receive raw CDC events
2. **Parsing**: Decode database-specific formats
3. **Transformation**: Apply field mappings and conversions
4. **Filtering**: Apply table and field filters
5. **Batching**: Group events for efficient processing
6. **Routing**: Send to appropriate Meilisearch index

#### 4. Meilisearch Client
- Connection pooling
- Automatic retry with exponential backoff
- Batch operations
- Index management
- Progress tracking

#### 5. Progress Tracker
- Redis-based for distributed deployments
- Atomic checkpoint updates
- Position recovery on restart
- Distributed locking for HA setups

#### 6. Dead Letter Queue
- Failed event storage
- Retry mechanisms
- Monitoring and alerting
- Manual intervention support

### Data Flow Patterns

#### CDC Event Flow
```
PostgreSQL WAL → Logical Replication → CDC Adapter → Event Queue
    → Transformer → Filter → Batcher → Meilisearch Client → Meilisearch
```

#### Full Sync Flow
```
PostgreSQL Table → Paginated Query → Data Stream → Transformer
    → Batch Processor → Temporary Index → Atomic Swap → Meilisearch
```

## Configuration Reference

### Complete Configuration Structure

```yaml
# Application metadata
app:
  name: "MeiliBridge"
  instance_id: "prod-01"  # Optional, auto-generated if not specified
  tags:
    environment: "production"
    region: "us-east-1"

# PostgreSQL source configuration
source:
  type: postgresql
  host: "localhost"
  port: 5432
  database: "myapp"
  username: "replication_user"    # Note: 'username' not 'user'
  password: "${POSTGRES_PASSWORD}"
  
  # Connection pool settings
  pool:
    max_size: 10
    min_idle: 1                   # Default is 1
    connection_timeout: 30        # Connection timeout (seconds)
    idle_timeout: 600             # Idle connection timeout (seconds)
  
  # Replication settings
  slot_name: "meilibridge_slot"   # Default: "meilibridge"
  publication: "meilibridge_pub"  # Default: "meilibridge_pub"
  
  # SSL/TLS configuration
  ssl:
    mode: "disable"               # disable, prefer, require, verify-ca, verify-full
    ca_cert: "/path/to/ca.crt"
    client_cert: "/path/to/client.crt"
    client_key: "/path/to/client.key"
  
  # Statement cache
  statement_cache:
    enabled: true                 # Enable prepared statement caching
    max_size: 100                 # Maximum cached statements

# Meilisearch destination configuration
meilisearch:
  url: "http://localhost:7700"
  api_key: "${MEILI_MASTER_KEY}"
  timeout: 30
  max_connections: 10             # Connection pool size
  batch_size: 1000                # Default batch size
  auto_create_index: true
  primary_key: "id"               # Default primary key if not specified per task
  
  # Index settings template
  index_settings:
    searchable_attributes: []     # Fields to search
    displayed_attributes: []      # Fields to return
    filterable_attributes: []     # Fields for filtering
    sortable_attributes: []       # Fields for sorting
    ranking_rules: []             # Custom ranking rules
    stop_words: []                # Stop words list
    synonyms: {}                  # Synonyms mapping
  
  # Circuit breaker configuration
  circuit_breaker:
    enabled: true                 # Enable circuit breaker
    error_rate: 0.5               # Open at 50% error rate
    min_request_count: 10         # Min requests before evaluation
    consecutive_failures: 5       # Or 5 consecutive failures
    timeout_secs: 60              # Time before recovery attempt

# Redis configuration for state management
redis:
  url: "redis://localhost:6379"
  password: "${REDIS_PASSWORD}"
  database: 0
  key_prefix: "meilibridge"
  
  pool:
    max_size: 10
    min_idle: 2
    connection_timeout: 5
    idle_timeout: 60

# Sync task definitions
sync_tasks:
  - id: "users_sync"
    table: "public.users"
    index: "users"
    primary_key: "id"
    
    # Start with full sync
    full_sync_on_start: true
    
    # Auto-start this task
    auto_start: true
    
    # Filter configuration (optional)
    filter:
      tables:
        whitelist: ["public.users"]
      event_types: ["create", "update", "delete"]
      conditions:
        - op: not_equals
          field: "deleted"
          value: true
    
    # Transform configuration (optional)
    transform:
      fields:
        public.users:
          email:
            type: lowercase
            fields: ["email"]
          full_name:
            type: compute
            expression: "concat(first_name, last_name)"
            to: "full_name"
    
    # Field mapping (optional)
    mapping:
      tables:
        public.users:
          name: "users"  # Rename table
          fields:
            user_id: "id"  # Rename field
            created_at: "created_timestamp"
      
      unmapped_fields_strategy: "include"  # include, exclude, or prefix
      unmapped_fields_prefix: "_"
    
    # Sync options
    options:
      batch_size: 1000
      batch_timeout_ms: 1000
      deduplicate: false
      
      retry:
        max_retries: 3
        initial_delay: 1000
        max_delay: 60000
        multiplier: 2.0

# API server configuration
api:
  enabled: true
  host: "0.0.0.0"
  port: 7708
  
  # CORS settings
  cors:
    enabled: true
    origins: ["http://localhost:3000"]
    methods: ["GET", "POST", "PUT", "DELETE"]
    headers: ["Content-Type", "Authorization"]
  
  # Authentication
  auth:
    enabled: true
    type: "bearer"
    tokens:
      - name: "admin"
        token: "${API_ADMIN_TOKEN}"
        role: "admin"
      - name: "readonly"
        token: "${API_READONLY_TOKEN}"
        role: "read"

# Logging configuration
logging:
  level: "info"  # trace, debug, info, warn, error
  format: "pretty"  # pretty, json, compact
  
  # File output
  file:
    enabled: true
    path: "/var/log/meilibridge/app.log"
    rotation: "daily"  # daily, size, never
    max_size: "100MB"
    max_age: 7
    max_backups: 5
    compress: true
  
  # Optional: send logs to external service
  export:
    enabled: false
    endpoint: "https://logs.example.com"
    api_key: "${LOG_API_KEY}"
    batch_size: 100
    flush_interval: 10

# Feature flags
features:
  auto_recovery: true      # Automatically recover from failures
  health_checks: true      # Enable health check endpoints
  metrics_export: true     # Export Prometheus metrics
  distributed_mode: false  # Enable distributed mode with Redis
  dead_letter_queue: true  # Enable DLQ for failed events

# Plugin configuration (optional)
plugins:
  directory: "./plugins"          # Plugin directory path
  enabled: []                     # List of enabled plugins

# At-least-once delivery with deduplication
# Note: Named 'exactly_once_delivery' for backward compatibility
exactly_once_delivery:
  enabled: true                   # Enable at-least-once delivery
  deduplication_window: 10000     # Events to track for deduplication
  transaction_timeout_secs: 30    # Transaction timeout
  two_phase_commit: true          # Use two-phase commit protocol
  checkpoint_before_write: true   # Atomic checkpoint before write

# Error handling configuration
error_handling:
  retry:
    enabled: true
    max_attempts: 3
    initial_backoff_ms: 100
    max_backoff_ms: 30000
    backoff_multiplier: 2.0
    jitter_factor: 0.1
  
  dead_letter_queue:
    enabled: true
    storage: "memory"             # memory or redis
    max_entries_per_task: 10000
    retention_hours: 24
    auto_reprocess_interval_minutes: 0  # 0 = disabled
  
  circuit_breaker:
    enabled: false                # Global circuit breaker
    failure_threshold_percent: 50
    min_requests: 10
    reset_timeout_seconds: 60
    half_open_max_requests: 3
```

### Environment Variables

All configuration values support environment variable substitution using `${VAR_NAME}` syntax.

Common environment variables:
```bash
# Database credentials
export POSTGRES_PASSWORD=secret
export POSTGRES_HOST=db.example.com

# Meilisearch
export MEILI_MASTER_KEY=masterkey
export MEILI_URL=http://meilisearch:7700

# Redis
export REDIS_URL=redis://redis:6379
export REDIS_PASSWORD=redispass

# API
export API_ADMIN_TOKEN=admin-secret
export API_READONLY_TOKEN=readonly-secret

# Logging
export LOG_LEVEL=info
export LOG_FORMAT=json
```

## Data Models

### Core Models

#### Event Model
```rust
pub struct Event {
    pub id: EventId,
    pub event_type: EventType,
    pub source: EventSource,
    pub data: EventData,
    pub metadata: EventMetadata,
    pub timestamp: DateTime<Utc>,
}

pub enum EventType {
    Create,
    Update,
    Delete,
    Truncate,
    Schema,
}

pub struct EventData {
    pub key: serde_json::Value,
    pub old: Option<HashMap<String, serde_json::Value>>,
    pub new: Option<HashMap<String, serde_json::Value>>,
}
```

#### Stream Event Model
```rust
pub enum Event {
    Cdc(CdcEvent),
    FullSync { table: String, data: Value },
    Insert { table: String, new_data: Value, position: Option<Position> },
    Update { table: String, old_data: Value, new_data: Value, position: Option<Position> },
    Delete { table: String, old_data: Value, position: Option<Position> },
    Checkpoint(Position),
    Heartbeat,
}
```

#### Position Tracking
```rust
pub enum Position {
    PostgreSQL { lsn: String, xid: Option<u64> },
    MySQL { file: String, position: u64, gtid: Option<String> },
    MongoDB { resume_token: String },
}
```

## Performance Tuning

### Connection Pool Optimization

#### PostgreSQL Pool
```yaml
source:
  type: postgresql
  pool:
    max_size: 20              # Maximum connections
    min_idle: 5               # Minimum idle connections
    connection_timeout: 30    # Seconds to wait for connection
    idle_timeout: 600         # Seconds before closing idle connection
```

#### Redis Pool
```yaml
redis:
  pool:
    max_size: 20
    min_idle: 5
    connection_timeout: 5
    idle_timeout: 60
```

### Batching Strategies

#### Time-based Batching
```yaml
options:
  batch_size: 1000
  batch_timeout_ms: 1000  # Flush every second
```

#### Memory-based Batching
```yaml
options:
  batch_size: 5000
  batch_memory_limit: "50MB"
  batch_timeout_ms: 5000
```

### Memory Management

1. **Event Buffer Size**:
```yaml
pipeline:
  event_buffer_size: 10000
  backpressure_threshold: 0.8
```

2. **Garbage Collection Tuning**:
```bash
RUST_MIN_STACK=8388608  # 8MB stack size
RUST_BACKTRACE=1        # Enable backtraces
```

### CPU Optimization

1. **Worker Threads**:
```yaml
runtime:
  worker_threads: 8  # Default: CPU cores
  blocking_threads: 16
```

2. **Tokio Runtime**:
```yaml
runtime:
  type: "multi_thread"
  worker_threads: 8
  thread_name: "meilibridge-worker"
  thread_stack_size: 2097152  # 2MB
```

### Parallel Event Processing

MeiliBridge supports parallel processing of events to maximize throughput for high-volume CDC streams.

#### Configuration

```yaml
performance:
  parallel_processing:
    enabled: true                    # Enable parallel processing
    workers_per_table: 4            # Number of worker threads per table
    max_concurrent_events: 1000     # Maximum events processed concurrently
    work_stealing: true             # Enable work stealing between tables
    work_steal_interval_ms: 100     # Work stealing check interval
    work_steal_threshold: 50        # Minimum queue size difference for stealing
  
  batch_processing:
    default_batch_size: 100         # Default batch size
    max_batch_size: 1000            # Maximum batch size
    min_batch_size: 10              # Minimum batch size
    batch_timeout_ms: 5000          # Batch timeout
    adaptive_batching: true         # Dynamic batch sizing
    
    adaptive_config:
      target_latency_ms: 1000       # Target processing time
      adjustment_factor: 0.2        # Adjustment aggressiveness (0-1)
      metric_window_size: 10        # Metrics to average
      adjustment_interval_ms: 5000  # Min time between adjustments
      memory_pressure_threshold: 80.0  # Memory % to reduce batch
      per_table_optimization: true  # Per-table batch sizing
  
  connection_pool:
    max_connections: 20             # Max connections
    min_connections: 5              # Min connections
    connection_timeout: 30          # Timeout (seconds)
    idle_timeout: 600               # Idle timeout (seconds)
```

#### How It Works

1. **Per-Table Parallelism**: Each table gets its own set of worker threads
2. **Event Distribution**: Incoming CDC events are distributed to table-specific queues
3. **Parallel Processing**: Workers process events concurrently while maintaining order
4. **Work Stealing**: Idle workers can steal work from busy tables for better load balancing

#### Benefits

- **Higher Throughput**: Process multiple events simultaneously
- **Better CPU Utilization**: Leverage multi-core systems effectively
- **Automatic Load Balancing**: Work stealing prevents bottlenecks
- **Configurable Parallelism**: Adjust workers based on workload

#### Monitoring

Monitor parallel processing performance:

```bash
# Check parallel processing status
curl http://localhost:7701/api/parallel/status

# View queue sizes
curl http://localhost:7701/api/parallel/queues

# Prometheus metrics
meilibridge_parallel_worker_events_total
meilibridge_parallel_queue_size
meilibridge_work_stealing_operations_total
```

#### Best Practices

1. **Worker Count**: Set `workers_per_table` to 2-4x CPU cores for I/O-bound workloads
2. **Queue Monitoring**: Monitor queue sizes to detect processing bottlenecks
3. **Work Stealing**: Enable for workloads with uneven table activity
4. **Memory Usage**: Each worker maintains its own processing buffer

#### Example: High-Volume Setup

```yaml
performance:
  parallel_processing:
    enabled: true
    workers_per_table: 8            # For high-volume tables
    max_concurrent_events: 5000     # Increased concurrency
    work_stealing: true
    work_steal_interval_ms: 50      # More aggressive stealing
    
  batch_processing:
    default_batch_size: 1000
    max_batch_size: 5000
    batch_timeout_ms: 100           # Faster batching for parallel workers
```

## Security

### Authentication Methods

#### API Key Authentication
```yaml
api:
  auth:
    enabled: true
    type: "bearer"
    tokens:
      - name: "admin"
        token: "${API_ADMIN_TOKEN}"
        role: "admin"
```

### TLS Configuration

#### PostgreSQL TLS
```yaml
source:
  type: postgresql
  ssl:
    mode: "require"  # disable, prefer, require, verify-ca, verify-full
    ca_cert: "/path/to/ca.crt"
    client_cert: "/path/to/client.crt"
    client_key: "/path/to/client.key"
```

#### API TLS
```yaml
api:
  tls:
    enabled: true
    cert: "/path/to/server.crt"
    key: "/path/to/server.key"
    client_auth: "optional"  # none, optional, required
```

### Secrets Management

1. **Environment Variables**: Use `${VAR_NAME}` in configuration
2. **File-based Secrets**: Reference files with `file://path/to/secret`
3. **Vault Integration**: Use HashiCorp Vault for dynamic secrets

## High Availability

### Single Instance HA

```yaml
ha:
  mode: "single"
  recovery:
    enabled: true
    checkpoint_interval: 60
    max_recovery_time: 300
```

### Active-Passive HA

```yaml
ha:
  mode: "active_passive"
  leader_election:
    backend: "redis"
    ttl: 30
    renew_interval: 10
  
  failover:
    automatic: true
    grace_period: 60
```

### Active-Active HA

```yaml
ha:
  mode: "active_active"
  coordination:
    backend: "redis"
    partition_strategy: "hash"  # hash, range, custom
  
  load_balancing:
    algorithm: "round_robin"  # round_robin, least_loaded, consistent_hash
```

### Health Checks

```yaml
health:
  liveness:
    endpoint: "/health/live"
    interval: 10
    timeout: 5
    failure_threshold: 3
  
  readiness:
    endpoint: "/health/ready"
    interval: 5
    timeout: 3
    success_threshold: 2
```

## Monitoring & Observability

### Metrics Export

```yaml
metrics:
  enabled: true
  endpoint: "/metrics"
  namespace: "meilibridge"
  
  exporters:
    prometheus:
      enabled: true
      port: 7709
    
    statsd:
      enabled: false
      host: "localhost"
      port: 8125
```

### Distributed Tracing

```yaml
tracing:
  enabled: true
  sampler: "always"  # always, never, probabilistic
  sample_rate: 0.1
  
  exporters:
    jaeger:
      enabled: true
      endpoint: "http://jaeger:14268/api/traces"
    
    zipkin:
      enabled: false
      endpoint: "http://zipkin:9411/api/v2/spans"
```

## Circuit Breaker Protection

MeiliBridge uses a sophisticated circuit breaker implementation to protect against cascading failures and provide graceful degradation.

### Implementation Details

The circuit breaker is implemented using the `circuit_breaker` crate and tracks:
- Success/failure rates over configurable windows
- Consecutive failure counts
- Response time percentiles
- Error categorization (timeout, connection, server errors)

### States and Transitions

1. **Closed** (Normal Operation)
   - All requests pass through
   - Monitors failure rate and latency
   - Transitions to Open on threshold breach

2. **Open** (Failure Protection)
   - Rejects requests immediately with `CircuitBreakerOpen` error
   - No load on failing service
   - Waits for timeout before testing

3. **Half-Open** (Recovery Testing)
   - Allows limited requests through
   - Monitors success rate
   - Returns to Closed on success or Open on failure

### Configuration

```yaml
meilisearch:
  circuit_breaker:
    enabled: true
    
    # Failure thresholds
    error_threshold: 0.5         # Open if 50% requests fail
    timeout_threshold: 0.8       # Open if 80% requests timeout
    consecutive_failures: 5      # Or 5 consecutive failures
    
    # Timing configuration
    window_size_secs: 60        # Evaluation window
    timeout_secs: 60            # Time in Open state
    half_open_requests: 3       # Test requests in Half-Open
    
    # Request configuration
    min_request_count: 10       # Minimum requests to evaluate
    request_timeout_ms: 5000    # Individual request timeout
    
    # Advanced settings
    failure_categories:         # Which errors trigger circuit
      - "timeout"
      - "connection_refused"
      - "internal_server_error"
    exclude_status_codes: [429] # Don't count rate limits
```

### Metrics and Monitoring

```bash
# Request metrics
meilibridge_circuit_breaker_requests_total{state="closed",result="success"}
meilibridge_circuit_breaker_requests_total{state="open",result="rejected"}
meilibridge_circuit_breaker_error_rate{window="1m"}

# State tracking
meilibridge_circuit_breaker_state{name="meilisearch"} # 0=closed, 1=open, 2=half-open
meilibridge_circuit_breaker_state_duration_seconds{state="open"}
meilibridge_circuit_breaker_transitions_total{from="closed",to="open"}

# Performance metrics
meilibridge_circuit_breaker_response_time_ms{quantile="0.99"}
meilibridge_circuit_breaker_timeout_ratio
```

### Integration with DLQ

When the circuit breaker opens:
1. Failed events are sent to the Dead Letter Queue
2. Events are tagged with `circuit_breaker_open` reason
3. Automatic retry is disabled until circuit closes
4. Manual intervention may be required for recovery

### Benefits

- **Fail Fast**: Prevents wasting resources on operations likely to fail
- **Automatic Recovery**: Periodically tests if service has recovered
- **Reduced Load**: Gives failing services time to recover
- **Better User Experience**: Quick failures instead of timeouts

## Advanced Configuration

### Custom Transformers

```yaml
transformers:
  - name: "email_hasher"
    type: "custom"
    script: |
      function transform(event) {
        if (event.data.email) {
          event.data.email_hash = hash_sha256(event.data.email);
          delete event.data.email;
        }
        return event;
      }
```

### Conditional Routing

```yaml
routing:
  rules:
    - condition: "event.type == 'create' && event.table == 'users'"
      index: "new_users"
    
    - condition: "event.data.vip == true"
      index: "vip_users"
    
    - condition: "default"
      index: "users"
```

### Rate Limiting

```yaml
rate_limiting:
  enabled: true
  
  limits:
    - name: "api_requests"
      max: 1000
      window: 60  # seconds
      
    - name: "cdc_events"
      max: 10000
      window: 1
```

This comprehensive reference covers all aspects of MeiliBridge configuration and architecture. For specific implementation details, see the [CDC Implementation & Operations](./cdc-operations.md) guide.