# MeiliBridge Demo

This demo showcases MeiliBridge's real-time data synchronization capabilities between PostgreSQL and Meilisearch using Change Data Capture (CDC).

## What This Demo Demonstrates

### Core Features
1. **Real-time CDC Streaming**: Watch live data changes flow from PostgreSQL to Meilisearch
2. **Full Table Sync**: Initial bulk data import from existing PostgreSQL tables
3. **Automatic Schema Mapping**: Database columns automatically mapped to Meilisearch fields
4. **Transaction Handling**: Proper handling of database transactions and rollbacks
5. **Error Recovery**: Automatic retry and dead letter queue for failed operations
6. **Performance Monitoring**: Prometheus metrics and health checks
7. **Dynamic Configuration**: Hot-reload configuration changes without restart

### Use Cases Demonstrated
- E-commerce product catalog sync
- Real-time inventory updates
- Price changes propagation
- Product additions and deletions
- Soft delete handling

## Prerequisites

- Docker and Docker Compose installed
- At least 4GB of available RAM
- Port availability: 5432 (PostgreSQL), 7700 (Meilisearch), 8080 (MeiliBridge API), 6379 (Redis)

## Quick Start

1. **Clone and navigate to demo directory**
   ```bash
   cd demo
   ```

2. **Start all services**
   ```bash
   ./start.sh
   # Or manually: docker compose up -d
   ```

3. **Wait for services to initialize** (about 30 seconds)
   ```bash
   # Check service health
   docker compose ps
   
   # View MeiliBridge logs
   docker compose logs -f meilibridge
   ```

4. **Verify the setup**
   ```bash
   # Check MeiliBridge health
   curl http://localhost:8080/health
   
   # Check sync task status
   curl http://localhost:8080/tasks
   ```

5. **Access the services**
   - MeiliBridge API: http://localhost:8080
   - Meilisearch API: http://localhost:7700 (API key: masterKey123)
   - Meilisearch UI: http://localhost:24900 (visual search interface)
   - PostgreSQL: `localhost:5432` (user: postgres, password: postgres)
   - Prometheus metrics: http://localhost:8080/metrics

## Demo Scenarios

### 1. Initial Data Load
The demo starts with 1000 sample products in PostgreSQL that are automatically synced to Meilisearch.

```bash
# Check products in Meilisearch
curl -X POST http://localhost:7700/indexes/products/search \
  -H 'Authorization: Bearer masterKey123' \
  -H 'Content-Type: application/json' \
  -d '{"q":"","offset":0,"limit":20}' | jq
```

### 2. Real-time Updates
Watch live updates as the data generator modifies products:

```bash
# Terminal 1: Watch MeiliBridge logs
docker compose logs -f meilibridge | grep -E "processed|synced"

# Terminal 2: Watch Meilisearch document count
watch -n 1 'curl -s http://localhost:7700/indexes/products/stats | jq .numberOfDocuments'
```

### 3. Manual Data Changes
Test CDC by making manual changes:

```bash
# Connect to PostgreSQL
docker compose exec postgres psql -U postgres -d demo

# Insert a new product
INSERT INTO products (name, description, price, category, in_stock, tags)
VALUES ('Demo Product', 'This is a test product', 99.99, 'Electronics', true, '["new", "featured"]'::jsonb);

# Update a product
UPDATE products SET price = 199.99 WHERE id = 1;

# Soft delete a product
UPDATE products SET deleted_at = NOW() WHERE id = 2;
```

### 4. Visual Search with Meilisearch UI
Explore your synchronized data visually:

1. Open http://localhost:24900 in your browser
2. Enter the Meilisearch URL: `http://localhost:7700`
3. Enter the API key: `masterKey123`
4. Select the `products` index
5. Try searching for:
   - Brand names: "Nike", "Apple", "Samsung"
   - Categories: "Electronics", "Sports", "Fashion"
   - Features: "wireless", "waterproof", "premium"
6. Use filters to narrow results by category, price range, or stock status
7. Watch real-time updates as pg_cron modifies data

### 5. Bulk Operations
Test batch processing:

```bash
# Run bulk insert script
docker compose exec postgres psql -U postgres -d demo -f /demo/sql/bulk_insert.sql
```

### 6. Configuration Hot-Reload
Modify sync behavior without restart:

```bash
# Edit config file (changes auto-reload)
# Adjust batch_size, sync_interval, or field_mapping in config/meilibridge.yaml
```

### 7. Error Handling
Simulate errors to see recovery:

```bash
# Stop Meilisearch temporarily
docker compose stop meilisearch

# Make changes in PostgreSQL (they'll queue up)
docker compose exec postgres psql -U postgres -d demo -c "UPDATE products SET price = price * 1.1;"

# Start Meilisearch again
docker compose start meilisearch

# Watch MeiliBridge recover and sync queued changes
docker compose logs -f meilibridge
```

## Monitoring

### Prometheus Metrics
View real-time metrics:
```bash
# Event processing rate
curl -s http://localhost:8080/metrics | grep meilibridge_events_processed_total

# Sync latency
curl -s http://localhost:8080/metrics | grep meilibridge_sync_latency

# Error rate
curl -s http://localhost:8080/metrics | grep meilibridge_errors_total
```

### Health Checks
```bash
# Overall health
curl http://localhost:8080/health

# Component health
curl http://localhost:8080/health/postgresql
curl http://localhost:8080/health/meilisearch
curl http://localhost:8080/health/redis
```

## Automated Data Generation with pg_cron

The demo uses PostgreSQL's pg_cron extension to automatically generate realistic data changes:

### Scheduled Jobs
- **New Products**: Inserts new products every minute
- **Price Updates**: Updates random product prices every minute
- **Stock Changes**: Modifies stock levels every minute
- **Tag Updates**: Adds trending tags every minute
- **Soft Deletes**: Removes old products every 2 minutes
- **Product Restoration**: Restores deleted products every minute
- **Category Sales**: Applies bulk discounts every 5 minutes
- **Bulk Transactions**: Simulates bundle purchases every 3 minutes

### Monitor pg_cron Jobs
```bash
# View scheduled jobs
docker compose exec postgres psql -U postgres -d demo -c "SELECT * FROM cron.job;"

# Check job execution stats
docker compose exec postgres psql -U postgres -d demo -c "SELECT * FROM cron_job_stats;"

# View demo statistics
docker compose exec postgres psql -U postgres -d demo -c "SELECT * FROM get_demo_stats();"
```

### Manage Jobs
```bash
# Pause a job
docker compose exec postgres psql -U postgres -d demo -c "SELECT cron.unschedule('insert-new-products');"

# Resume a job
docker compose exec postgres psql -U postgres -d demo -c "SELECT cron.schedule('insert-new-products', '* * * * *', 'SELECT insert_new_product();');"

# View job run history
docker compose exec postgres psql -U postgres -d demo -c "SELECT * FROM cron.job_run_details ORDER BY start_time DESC LIMIT 10;"
```

## Troubleshooting

### View logs
```bash
# All services
docker compose logs

# Specific service
docker compose logs meilibridge
docker compose logs postgres
```

### Reset demo
```bash
# Stop and remove all containers
docker compose down -v

# Start fresh
docker compose up -d
```

### Common Issues

1. **Port conflicts**: Ensure ports 5432, 7700, 8080, 6379 are available
2. **Memory issues**: Increase Docker memory limit to at least 4GB
3. **Slow initial sync**: Normal for first-time setup with 1000+ products

## Architecture

```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│   PostgreSQL    │────▶│   MeiliBridge    │────▶│   Meilisearch   │
│  (Source Data)  │ CDC │  (Sync Engine)   │     │ (Search Engine) │
└─────────────────┘     └──────────────────┘     └─────────────────┘
                               │
                               ▼
                        ┌─────────────────┐
                        │      Redis      │
                        │  (Checkpoints)  │
                        └─────────────────┘
```

## Stopping the Demo

To stop the demo:
```bash
./stop.sh
# Or manually: docker compose down
```

## Cleanup

Remove all demo resources:
```bash
docker compose down -v
rm -rf ./data
```

## Next Steps

1. **Customize Configuration**: Edit `config/meilibridge.yaml` for your needs
2. **Add More Tables**: Extend `init.sql` and configuration for multiple tables
3. **Production Setup**: Use the demo as a template for production deployment
4. **Monitor Performance**: Set up Grafana dashboards using the Prometheus metrics

## Learn More

- [MeiliBridge Documentation](https://github.com/binarytouch/meilibridge)
- [Meilisearch Documentation](https://docs.meilisearch.com)
- [PostgreSQL Logical Replication](https://www.postgresql.org/docs/current/logical-replication.html)