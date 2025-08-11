# Common Test Utilities

This module provides shared utilities for integration tests to avoid code duplication and ensure consistent test setup.

## Module Structure

- **`setup.rs`** - Test environment setup with container management
- **`api_helpers.rs`** - API testing helpers and assertions
- **`test_data.rs`** - Test data generators and SQL helpers
- **`containers.rs`** - Container definitions and lifecycle management
- **`fixtures.rs`** - Configuration fixtures and test setup helpers
- **`helpers.rs`** - Additional utility functions

## Key Components

### TestEnvironment

Provides a builder pattern for setting up test services:

```rust
let env = TestEnvironment::new()
    .with_postgres().await?
    .with_redis().await?
    .with_meilisearch().await?;
```

### Quick Setup Functions

For simple test scenarios:

```rust
// Setup individual services
let (container, client, url) = setup_postgres_cdc().await?;
let (container, client, url) = setup_redis().await?;
let (container, client, url) = setup_meilisearch().await?;

// Setup all services
let env = setup_all_services().await?;
```

### TestApiServer

Simplified API testing:

```rust
let server = TestApiServer::new(config).await;
let response = server.get("/health").await;
let response = server.post_json("/tasks", &task_config).await;
server.shutdown();
```

### Test Data Generators

Generate realistic test data:

```rust
let users = generate_users(100);
let products = generate_products(50);
let orders = generate_orders(200, user_count, product_count);
let checkpoint = create_checkpoint("task_id", "0/1234567");
```

## Usage Example

See `full_integration_example.rs` for a complete example demonstrating all utilities.

## Benefits

1. **Reduced Duplication**: Common setup code is centralized
2. **Consistent Container Management**: Uses static Docker client to keep containers alive
3. **Type Safety**: Strongly typed helpers reduce runtime errors
4. **Easy Service Composition**: Mix and match services as needed
5. **Built-in Assertions**: Common test assertions are provided

## Migration Guide

To update existing tests:

1. Replace local `setup_*` functions with imports from `common`
2. Use `TestEnvironment` for multi-service tests
3. Replace manual API setup with `TestApiServer`
4. Use test data generators instead of manual data creation