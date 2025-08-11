# CDC Implementation & Operations

This guide covers the technical implementation of Change Data Capture (CDC) with PostgreSQL, debugging procedures, monitoring, and operational best practices.

## Table of Contents
- [PostgreSQL CDC Implementation](#postgresql-cdc-implementation)
- [CDC Configuration](#cdc-configuration)
- [Monitoring & Debugging](#monitoring--debugging)
- [Troubleshooting Guide](#troubleshooting-guide)
- [Performance Optimization](#performance-optimization)
- [Operational Best Practices](#operational-best-practices)

## PostgreSQL CDC Implementation

### Overview

MeiliBridge uses PostgreSQL's logical replication feature to capture changes in real-time. The implementation uses the `pgoutput` plugin, which is the standard logical replication protocol in PostgreSQL 10+. The CDC system is fully implemented with support for all DML operations (INSERT, UPDATE, DELETE) and handles complex data types including arrays and JSONB.

### Architecture

```
┌─────────────────────┐
│   PostgreSQL WAL    │
│  (Write-Ahead Log)  │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  Logical Decoding   │
│   (pgoutput)        │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│ Replication Slot    │
│ (meilibridge_slot)  │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  MeiliBridge CDC    │
│     Consumer        │
└─────────────────────┘
```

### Setting Up Logical Replication

#### 1. Enable Logical Replication

In `postgresql.conf`:
```ini
# Logical replication settings
wal_level = logical
max_replication_slots = 10
max_wal_senders = 10

# Optional: tune for performance
wal_sender_timeout = 60s
wal_receiver_timeout = 60s
```

#### 2. Create Replication User

```sql
-- Create dedicated replication user
CREATE USER meilibridge_repl WITH REPLICATION LOGIN PASSWORD 'secure_password';

-- Grant necessary permissions
GRANT USAGE ON SCHEMA public TO meilibridge_repl;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO meilibridge_repl;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT ON TABLES TO meilibridge_repl;
```

#### 3. Create Publication

```sql
-- Create publication for all tables
CREATE PUBLICATION meilibridge_pub FOR ALL TABLES;

-- Or create for specific tables
CREATE PUBLICATION meilibridge_pub FOR TABLE 
    public.users,
    public.orders,
    public.products;

-- Add tables to existing publication
ALTER PUBLICATION meilibridge_pub ADD TABLE public.new_table;
```

#### 4. Set Replica Identity

For UPDATE and DELETE operations to include all column values:
```sql
-- Set for individual table
ALTER TABLE public.users REPLICA IDENTITY FULL;

-- Set for all tables in a schema
DO $$
DECLARE
    r RECORD;
BEGIN
    FOR r IN SELECT tablename FROM pg_tables WHERE schemaname = 'public'
    LOOP
        EXECUTE 'ALTER TABLE public.' || quote_ident(r.tablename) || ' REPLICA IDENTITY FULL';
    END LOOP;
END $$;
```

### CDC Event Processing

#### Event Types and Structure

1. **INSERT Event**:
```json
{
  "action": "I",
  "schema": "public",
  "table": "users",
  "columns": [
    {"name": "id", "type": "integer", "value": 123},
    {"name": "email", "type": "text", "value": "user@example.com"},
    {"name": "created_at", "type": "timestamp", "value": "2024-01-01T00:00:00Z"}
  ]
}
```

2. **UPDATE Event**:
```json
{
  "action": "U",
  "schema": "public",
  "table": "users",
  "identity": [
    {"name": "id", "type": "integer", "value": 123}
  ],
  "columns": [
    {"name": "email", "type": "text", "value": "newemail@example.com"}
  ]
}
```

3. **DELETE Event**:
```json
{
  "action": "D",
  "schema": "public",
  "table": "users",
  "identity": [
    {"name": "id", "type": "integer", "value": 123}
  ]
}
```

### Implementation Details

#### Replication Architecture

MeiliBridge implements a complete CDC pipeline with these components:

1. **ReplicationConsumer**: Manages the replication connection and protocol
2. **PgOutputDecoder**: Binary decoder for pgoutput format messages
3. **WALConsumer**: Processes decoded messages into events
4. **EventStream**: Delivers events to the processing pipeline

#### Core Implementation

```rust
pub struct ReplicationConsumer {
    config: PostgreSQLConfig,
    slot_manager: ReplicationSlotManager,
    event_sender: mpsc::Sender<Event>,
    checkpoint_manager: Arc<CheckpointManager>,
    last_lsn: Arc<Mutex<Option<PgLsn>>>,
}

// Automatic slot management
pub struct ReplicationSlotManager {
    slot_name: String,
    publication: String,
    create_if_not_exists: bool,
}

// Binary protocol decoder
pub struct PgOutputDecoder {
    relations: HashMap<u32, RelationInfo>,
    type_map: HashMap<Oid, Type>,
}
```

#### Message Processing Pipeline

```rust
// Binary message decoding
impl PgOutputDecoder {
    pub fn decode_message(&mut self, buffer: &[u8]) -> Result<Option<DecodedMessage>> {
        let message_type = buffer[0];
        match message_type {
            b'B' => self.decode_begin(buffer),      // Transaction begin
            b'C' => self.decode_commit(buffer),     // Transaction commit
            b'I' => self.decode_insert(buffer),     // Insert operation
            b'U' => self.decode_update(buffer),     // Update operation
            b'D' => self.decode_delete(buffer),     // Delete operation
            b'R' => self.decode_relation(buffer),   // Relation info
            b'Y' => self.decode_type(buffer),       // Type info
            _ => Ok(None)
        }
    }
}

// Event transformation
impl WALConsumer {
    pub async fn process_decoded_message(&mut self, msg: DecodedMessage) -> Result<()> {
        let event = match msg {
            DecodedMessage::Insert { relation, tuple } => {
                self.create_insert_event(relation, tuple)
            }
            DecodedMessage::Update { relation, old_tuple, new_tuple } => {
                self.create_update_event(relation, old_tuple, new_tuple)
            }
            DecodedMessage::Delete { relation, old_tuple } => {
                self.create_delete_event(relation, old_tuple)
            }
            DecodedMessage::Commit { lsn, timestamp } => {
                self.checkpoint_manager.update_position(Position::PostgreSQL { lsn, xid: None }).await?;
                Event::Checkpoint(Position::PostgreSQL { lsn, xid: None })
            }
            _ => return Ok(())
        };
        
        self.event_sender.send(event).await?;
        Ok(())
    }
}
```

## CDC Configuration

### Basic CDC Configuration

```yaml
source:
  type: postgresql
  postgresql:
    # Connection settings
    host: "localhost"
    port: 5432
    database: "myapp"
    user: "meilibridge_repl"
    password: "${POSTGRES_REPL_PASSWORD}"
    
    # Connection pool configuration
    pool:
      max_size: 10
      min_idle: 2
      acquire_timeout: 30
      idle_timeout: 600
    
    # Replication settings
    replication:
      slot_name: "meilibridge_slot"
      publication: "meilibridge_pub"
      create_slot: true          # Auto-create slot if missing
      temporary_slot: false      # Persistent slot
      
      # Protocol settings
      proto_version: 1           # pgoutput protocol version
      streaming: true            # Enable streaming mode
      
      # Keepalive settings
      wal_sender_timeout: 60     # Match PostgreSQL setting
      standby_message_timeout: 10 # Status update interval
```

### Advanced CDC Options

```yaml
# Sync task CDC configuration
sync_tasks:
  - id: "users_sync"
    source:
      table: "public.users"
    
    # CDC-specific settings
    cdc:
      enabled: true
      auto_start: true
      
      # Event filtering
      event_types: ["insert", "update", "delete"]
      
      # Column filtering (only sync these columns)
      columns: ["id", "email", "name", "updated_at"]
      
      # Start position
      start_position: "current"  # current, beginning, or specific LSN
      
      # Batch processing
      batch_size: 100
      batch_timeout_ms: 1000
      
      # Transaction handling
      transaction_mode: "individual"  # individual or batch
      
    # Full sync configuration
    full_sync:
      enabled: true
      on_start: true
      page_size: 1000
      order_by: "id"
      
    # Soft delete detection
    soft_delete:
      enabled: true
      field: "deleted_at"
      strategy: "mark"  # mark or remove
```

## Monitoring & Debugging

### Key Metrics to Monitor

#### 1. Replication Lag

Monitor the lag between PostgreSQL and MeiliBridge:
```sql
-- Check replication slot lag
SELECT 
    slot_name,
    active,
    pg_size_pretty(pg_wal_lsn_diff(pg_current_wal_lsn(), restart_lsn)) as lag_size,
    pg_wal_lsn_diff(pg_current_wal_lsn(), restart_lsn) as lag_bytes
FROM pg_replication_slots
WHERE slot_name = 'meilibridge_slot';
```

#### 2. WAL Size

Monitor WAL accumulation:
```sql
-- Check WAL size
SELECT 
    pg_size_pretty(sum(size)) as total_wal_size
FROM pg_ls_waldir();
```

#### 3. Publication Status

```sql
-- Check publication tables
SELECT 
    schemaname,
    tablename
FROM pg_publication_tables
WHERE pubname = 'meilibridge_pub';
```

### Prometheus Metrics

MeiliBridge exposes comprehensive CDC metrics:

```
# Core CDC Metrics
meilibridge_cdc_events_processed_total{table="users",event_type="insert"}
meilibridge_cdc_events_failed_total{table="users",reason="decode_error"}
meilibridge_cdc_lag_seconds{slot="meilibridge_slot"}
meilibridge_cdc_lag_bytes{slot="meilibridge_slot"}

# Replication Connection
meilibridge_replication_connection_status{slot="meilibridge_slot"} # 1=connected, 0=disconnected
meilibridge_replication_reconnects_total
meilibridge_replication_last_lsn{slot="meilibridge_slot"}

# Performance Metrics
meilibridge_cdc_decode_duration_seconds_histogram
meilibridge_cdc_batch_size_histogram
meilibridge_cdc_processing_duration_seconds_histogram

# PostgreSQL Metrics
meilibridge_postgres_wal_size_bytes
meilibridge_postgres_replication_slot_active{slot="meilibridge_slot"}
meilibridge_postgres_publication_table_count{publication="meilibridge_pub"}
```

### Debug Logging and Tracing

Enable detailed CDC logging:
```yaml
logging:
  level: "info"
  format: "json"
  
  # Module-specific log levels
  tracing:
    enabled: true
    targets:
      - "meilibridge=info"
      - "meilibridge::source::postgres=debug"
      - "meilibridge::source::postgres::pgoutput=trace"  # Binary protocol details
      - "meilibridge::pipeline=debug"
      - "meilibridge::checkpoint=debug"
      - "tokio_postgres=warn"
```

### Real-time CDC Monitoring

Use the diagnostics API for real-time CDC inspection:
```bash
# Get current CDC status
curl http://localhost:7708/api/diagnostics/cdc

# Response:
{
  "status": "streaming",
  "current_lsn": "0/1634FA0",
  "lag_bytes": 1024,
  "events_per_second": 156.4,
  "last_event_time": "2024-01-15T10:30:45Z",
  "connected_since": "2024-01-15T09:00:00Z",
  "tables": {
    "public.users": {
      "inserts": 1234,
      "updates": 567,
      "deletes": 89
    }
  }
}
```

### CDC Event Inspection

MeiliBridge provides comprehensive CDC debugging tools:

#### Event Capture
```bash
# Start capturing CDC events
curl -X POST http://localhost:7708/api/diagnostics/cdc/capture \
  -H "Content-Type: application/json" \
  -d '{
    "enabled": true,
    "tables": ["public.users"],
    "event_types": ["insert", "update"],
    "max_events": 100
  }'

# Get captured events
curl http://localhost:7708/api/diagnostics/cdc/events

# Response:
{
  "events": [
    {
      "timestamp": "2024-01-15T10:30:45Z",
      "lsn": "0/1634FA0",
      "table": "public.users",
      "event_type": "insert",
      "data": {
        "id": 123,
        "email": "user@example.com"
      }
    }
  ],
  "total_captured": 42
}
```

#### Test Event Generation
```bash
# Generate test CDC event
curl -X POST http://localhost:7708/api/diagnostics/cdc/test-event \
  -H "Content-Type: application/json" \
  -d '{
    "table": "public.users",
    "event_type": "insert",
    "data": {"id": 999, "name": "Test User"}
  }'
```

## Troubleshooting Guide

### Common Issues and Solutions

#### 1. Replication Slot Already Exists

**Error**: `ERROR: replication slot "meilibridge_slot" already exists`

**Solution**:
```sql
-- Check existing slots
SELECT * FROM pg_replication_slots;

-- Drop old slot if needed
SELECT pg_drop_replication_slot('meilibridge_slot');
```

#### 2. No CDC Events Received

**Comprehensive Checklist**:

1. **Check replication slot status**:
```sql
-- Detailed slot information
SELECT 
    slot_name,
    plugin,
    slot_type,
    database,
    active,
    active_pid,
    restart_lsn,
    confirmed_flush_lsn,
    wal_status,
    safe_wal_size
FROM pg_replication_slots 
WHERE slot_name = 'meilibridge_slot';

-- Check if slot is lagging
SELECT 
    slot_name,
    pg_size_pretty(pg_wal_lsn_diff(pg_current_wal_lsn(), restart_lsn)) as replication_lag
FROM pg_replication_slots 
WHERE slot_name = 'meilibridge_slot';
```

2. Verify publication exists:
```sql
SELECT * FROM pg_publication WHERE pubname = 'meilibridge_pub';
```

3. **Check table configuration**:
```sql
-- Verify replica identity
SELECT 
    n.nspname AS schema_name,
    c.relname AS table_name,
    CASE c.relreplident
        WHEN 'd' THEN 'default (primary key)'
        WHEN 'n' THEN 'nothing'
        WHEN 'f' THEN 'full'
        WHEN 'i' THEN 'index'
    END AS replica_identity,
    c.relreplident
FROM pg_class c
JOIN pg_namespace n ON n.oid = c.relnamespace
WHERE c.relkind = 'r' 
AND n.nspname = 'public'
ORDER BY n.nspname, c.relname;

-- Check if table is in publication
SELECT 
    schemaname,
    tablename,
    pubname
FROM pg_publication_tables
WHERE pubname = 'meilibridge_pub'
ORDER BY schemaname, tablename;
```

4. Verify user permissions:
```sql
-- Check replication permission
SELECT rolreplication FROM pg_roles WHERE rolname = 'meilibridge_repl';

-- Check table permissions
SELECT has_table_privilege('meilibridge_repl', 'public.users', 'SELECT');
```

#### 3. High Replication Lag

**Symptoms**: 
- Increasing lag_bytes in replication slot
- Delayed event processing
- High memory usage
- Slow Meilisearch indexing

**Diagnosis**:
```sql
-- Check detailed lag information
SELECT 
    slot_name,
    active,
    pg_size_pretty(pg_wal_lsn_diff(pg_current_wal_lsn(), restart_lsn)) as lag_size,
    pg_wal_lsn_diff(pg_current_wal_lsn(), restart_lsn) / 1024.0 / 1024.0 as lag_mb,
    pg_current_wal_lsn() as current_lsn,
    restart_lsn,
    confirmed_flush_lsn
FROM pg_replication_slots 
WHERE slot_name = 'meilibridge_slot';
```

**Solutions**:

1. **Enable parallel processing**:
```yaml
performance:
  parallel_processing:
    enabled: true
    workers_per_table: 4
    work_stealing: true
```

2. **Optimize batching**:
```yaml
performance:
  adaptive_batching:
    enabled: true
    min_batch_size: 100
    max_batch_size: 5000
    target_batch_size: 1000
    latency_target_ms: 100
```

3. Optimize PostgreSQL:
```sql
-- Increase WAL sender timeout
ALTER SYSTEM SET wal_sender_timeout = '120s';

-- Increase max_wal_size
ALTER SYSTEM SET max_wal_size = '4GB';

-- Reload configuration
SELECT pg_reload_conf();
```

#### 4. Connection Drops

**Error**: `FATAL: terminating connection due to idle timeout`

**Solution**:
```yaml
postgresql:
  pool:
    idle_timeout: 0  # Disable idle timeout
    keepalive_interval: 30  # Send keepalive every 30s
    
  replication:
    keepalive_interval: 10  # More frequent keepalives
    status_interval: 5      # More frequent status updates
```

#### 5. Out of Memory

**Symptoms**: 
- Process killed by OOM killer
- Increasing memory usage

**Solutions**:
1. Limit batch sizes:
```yaml
options:
  batch_size: 1000
  batch_memory_limit: "100MB"
```

2. Enable backpressure:
```yaml
pipeline:
  backpressure_threshold: 0.8
  pause_on_backpressure: true
```

### Advanced Debugging

#### 1. PostgreSQL Replication Debugging

```sql
-- Enable detailed replication logging
ALTER SYSTEM SET log_replication_commands = on;
ALTER SYSTEM SET log_min_messages = 'debug1';
ALTER SYSTEM SET wal_debug = on;  -- Warning: verbose!

-- Enable logical decoding debugging
ALTER SYSTEM SET debug_logical_replication_streaming = on;

-- Apply changes
SELECT pg_reload_conf();

-- Check replication statistics
SELECT * FROM pg_stat_replication;
SELECT * FROM pg_stat_wal_receiver;
```

#### 2. MeiliBridge Debug Configuration

```yaml
# Enable comprehensive debugging
debug:
  # CDC debugging
  cdc:
    trace_enabled: true
    trace_tables: ["public.users", "public.orders"]
    log_binary_data: true      # Log raw pgoutput messages
    capture_events: true       # Store events for inspection
    event_buffer_size: 1000
    
  # Performance profiling
  profiling:
    enabled: true
    sample_rate: 0.1          # Sample 10% of operations
    
  # Memory tracking
  memory:
    track_allocations: true
    report_interval: 60       # seconds
```

#### 3. Test Replication Manually

```bash
# Test with pg_recvlogical
pg_recvlogical \
  -d myapp \
  -S test_slot \
  -P pgoutput \
  -o proto_version=1 \
  -o publication_names='meilibridge_pub' \
  --start \
  -f -
```

## Performance Optimization

### PostgreSQL Tuning for CDC

#### 1. WAL and Replication Configuration
```ini
# WAL settings optimized for CDC
wal_level = logical
wal_buffers = 32MB                    # Increase for high write load
wal_writer_delay = 200ms
wal_writer_flush_after = 1MB

# Checkpoint tuning
checkpoint_timeout = 30min
checkpoint_completion_target = 0.9
max_wal_size = 8GB                    # Increase for high change rate
min_wal_size = 2GB

# Replication settings
max_replication_slots = 10
max_wal_senders = 10
wal_sender_timeout = 120s             # Increase for slow consumers
wal_receiver_timeout = 120s
wal_receiver_status_interval = 10s

# Logical decoding
logical_decoding_work_mem = 256MB     # Memory per replication connection
max_slot_wal_keep_size = 10GB         # PostgreSQL 13+ WAL retention limit
```

#### 2. Replication Tuning
```ini
# Increase replication work memory
logical_decoding_work_mem = 256MB

# Tune sender/receiver
wal_sender_timeout = 120s
wal_receiver_timeout = 120s
```

### MeiliBridge Performance Tuning

#### 1. Optimized CDC Configuration
```yaml
# High-performance CDC settings
performance:
  # Connection pooling
  connection_pool_size: 20            # Increase for multiple tables
  statement_cache_size: 100           # Cache prepared statements
  
  # Event processing
  event_channel_size: 10000           # Internal event buffer
  max_pending_events: 50000           # Maximum in-flight events
  
  # Parallel processing
  parallel_processing:
    enabled: true
    workers_per_table: 4              # CPU cores / active tables
    work_stealing: true
    steal_ratio: 0.25                 # Steal when 25% idle
    
  # Adaptive batching
  adaptive_batching:
    enabled: true
    min_batch_size: 10
    max_batch_size: 5000
    memory_limit: "100MB"
    
  # Memory management
  memory_limit: "2GB"
  gc_interval: 60                     # Force GC every minute
  backpressure_threshold: 0.8         # Slow down at 80% memory
```

#### 2. Memory Management
```yaml
memory:
  # Event buffer limits
  max_event_buffer_mb: 512
  
  # GC tuning
  gc_interval: 60  # seconds
  
  # Circuit breaker
  memory_threshold_percent: 80
```

#### 3. Network and I/O Optimization
```yaml
postgresql:
  # TCP optimization
  tcp_keepalive: true
  tcp_keepalive_time: 60
  tcp_keepalive_interval: 10
  tcp_keepalive_count: 6
  tcp_nodelay: true                   # Disable Nagle's algorithm
  
  # Buffer tuning
  send_buffer_size: 4194304           # 4MB for high throughput
  receive_buffer_size: 4194304
  
  # Connection tuning
  connect_timeout: 30
  statement_timeout: 0                # No timeout for replication
  
  # SSL optimization (if used)
  ssl_mode: "prefer"
  ssl_compression: false              # CPU vs bandwidth tradeoff
```

## Operational Best Practices

### 1. Monitoring Setup

**Essential Alerts**:

```yaml
# Prometheus alerting rules
groups:
  - name: meilibridge_cdc
    rules:
      - alert: HighReplicationLag
        expr: meilibridge_cdc_lag_bytes > 1073741824  # 1GB
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High replication lag detected"
          
      - alert: CDCConnectionLost
        expr: meilibridge_replication_connection_status == 0
        for: 1m
        labels:
          severity: critical
          
      - alert: HighCDCErrorRate
        expr: rate(meilibridge_cdc_events_failed_total[5m]) > 0.1
        for: 5m
        labels:
          severity: warning
          
      - alert: ReplicationSlotInactive
        expr: meilibridge_postgres_replication_slot_active == 0
        for: 5m
        labels:
          severity: critical
```

**Grafana Dashboard**:
```json
{
  "dashboard": {
    "title": "MeiliBridge CDC Monitoring",
    "panels": [
      {
        "title": "Replication Lag",
        "targets": [
          {
            "expr": "meilibridge_cdc_lag_bytes"
          }
        ]
      },
      {
        "title": "Events Per Second",
        "targets": [
          {
            "expr": "rate(meilibridge_cdc_events_total[1m])"
          }
        ]
      }
    ]
  }
}
```

### 2. Backup and Recovery

**Checkpoint Management**:

MeiliBridge implements robust checkpoint persistence for reliable CDC recovery:

#### Features
- **Automatic Persistence**: Checkpoints saved after each transaction commit
- **Multi-Backend Support**: Redis (distributed) or in-memory (single instance)
- **Transaction Awareness**: Tracks both LSN and transaction IDs
- **Atomic Updates**: Ensures consistency during checkpoint updates
- **Automatic Recovery**: Resumes from last checkpoint on restart

#### Configuration
```yaml
# Redis-based checkpoints (recommended for production)
redis:
  url: "redis://localhost:6379"
  database: 0
  key_prefix: "meilibridge"
  
# Checkpoint behavior
checkpoint:
  flush_interval: 30          # Seconds between flushes
  flush_batch_size: 10        # Updates before forcing flush
  ttl: 604800                # 7 days retention
  
  # Recovery options
  recovery:
    verify_position: true     # Verify position exists in WAL
    fallback_to_start: false  # Start from beginning if checkpoint invalid
```

#### Manual Checkpoint Management
```bash
# Get current checkpoints
curl http://localhost:7708/api/checkpoints

# Set checkpoint for specific task
curl -X PUT http://localhost:7708/api/checkpoints/users_sync \
  -H "Content-Type: application/json" \
  -d '{"lsn": "0/1634FA0", "xid": 12345}'

# Delete checkpoint (force full resync)
curl -X DELETE http://localhost:7708/api/checkpoints/users_sync
```

**Backup and Export**:
```bash
# Export all checkpoints
curl http://localhost:7708/api/checkpoints/export > checkpoints_$(date +%Y%m%d).json

# Import checkpoints
curl -X POST http://localhost:7708/api/checkpoints/import \
  -H "Content-Type: application/json" \
  -d @checkpoints_20240115.json

# Backup to S3 (using AWS CLI)
aws s3 cp <(curl -s http://localhost:7708/api/checkpoints/export) \
  s3://backup-bucket/meilibridge/checkpoints/$(date +%Y%m%d_%H%M%S).json
```

**Slot Recreation**:
```sql
-- Backup slot position
SELECT slot_name, restart_lsn FROM pg_replication_slots;

-- Recreate slot at position
SELECT pg_create_logical_replication_slot('meilibridge_slot', 'pgoutput');
```

### 3. Maintenance Windows

**CDC Control Operations**:
```bash
# Pause specific sync task
curl -X POST http://localhost:7708/api/sync-tasks/users_sync/pause

# Resume specific sync task
curl -X POST http://localhost:7708/api/sync-tasks/users_sync/resume

# Pause all CDC operations
curl -X POST http://localhost:7708/api/cdc/pause-all

# Resume all CDC operations  
curl -X POST http://localhost:7708/api/cdc/resume-all

# Get pause/resume status
curl http://localhost:7708/api/cdc/status
```

**Graceful Restart**:
```bash
# Send graceful shutdown signal
kill -TERM $(pidof meilibridge)

# Or via systemd
systemctl stop meilibridge
```

### 4. Disaster Recovery

**Scenario: Lost Replication Slot**

1. Stop MeiliBridge
2. Create new slot:
```sql
SELECT pg_create_logical_replication_slot('meilibridge_slot_new', 'pgoutput');
```

3. Update configuration:
```yaml
replication:
  slot_name: "meilibridge_slot_new"
  start_lsn: "0/0"  # Start from beginning
```

4. Trigger full resync:
```bash
curl -X POST http://localhost:7708/api/v1/tasks/full-sync
```

### 5. Security Best Practices

#### Replication Security

1. **User Permissions** (Principle of Least Privilege):
```sql
-- Create dedicated replication user
CREATE USER meilibridge_repl WITH REPLICATION LOGIN PASSWORD 'StrongPassword123!';

-- Grant minimal required permissions
GRANT CONNECT ON DATABASE myapp TO meilibridge_repl;
GRANT USAGE ON SCHEMA public TO meilibridge_repl;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO meilibridge_repl;

-- Set default permissions for new tables
ALTER DEFAULT PRIVILEGES IN SCHEMA public 
    GRANT SELECT ON TABLES TO meilibridge_repl;
```

2. **Network Security**:
```yaml
# PostgreSQL SSL configuration
postgresql:
  ssl:
    enabled: true
    mode: "require"              # require, verify-ca, verify-full
    ca_cert: "/certs/ca.crt"
    client_cert: "/certs/client.crt"
    client_key: "/certs/client.key"
```

3. **Connection Restrictions** (pg_hba.conf):
```
# TYPE  DATABASE    USER              ADDRESS         METHOD
hostssl all        meilibridge_repl  10.0.0.0/24     md5
hostssl replication meilibridge_repl  10.0.0.0/24     md5
```

4. **Audit Configuration**:
```sql
-- Enable audit logging
ALTER SYSTEM SET log_connections = on;
ALTER SYSTEM SET log_disconnections = on;
ALTER SYSTEM SET log_replication_commands = on;
SELECT pg_reload_conf();
```

5. **Secrets Management**:
```yaml
# Use environment variables
postgresql:
  password: "${POSTGRES_REPL_PASSWORD}"
  
# Or use secret files
postgresql:
  password_file: "/run/secrets/postgres_password"
```

### 6. Production Checklist

Before deploying CDC to production:

- [ ] PostgreSQL WAL settings optimized
- [ ] Replication user created with minimal permissions
- [ ] SSL/TLS configured for replication connections
- [ ] Publication created with required tables
- [ ] Replica identity set appropriately
- [ ] Monitoring and alerting configured
- [ ] Checkpoint persistence enabled (Redis)
- [ ] Backup procedures documented
- [ ] Disaster recovery plan tested
- [ ] Performance baseline established

This comprehensive guide covers all aspects of CDC implementation, monitoring, debugging, and operations. For API usage and development, see the [API & Development Guide](./api-development.md).