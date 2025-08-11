# MeiliBridge Test Suite

This directory contains all tests for the MeiliBridge project, organized into three main categories:

## Test Organization

### Unit Tests (`unit/`)
Fast, isolated tests for individual components without external dependencies.

### Integration Tests (`integration/`)
Tests that verify integration with external services like PostgreSQL, Meilisearch, and Redis.

### End-to-End Tests (`e2e/`)
Complete production scenarios testing the full data flow from source to destination.

### Common Utilities (`common/`)
Shared test utilities, fixtures, and mock implementations.

## Running Tests

```bash
# Run all tests
cargo test

# Run only unit tests
cargo test --test unit

# Run only integration tests (requires Docker)
cargo test --test integration

# Run only e2e tests (requires full environment)
cargo test --test e2e

# Run specific test file
cargo test --test unit::api::sync_task_test

# Run with coverage
cargo tarpaulin --out Html
```

## Test Guidelines

1. **Naming Convention**: Test functions should clearly describe what they test
   - Good: `test_sync_task_creation_with_valid_config`
   - Bad: `test_1` or `test_sync_task`

2. **Test Independence**: Each test should be independent and not rely on others

3. **Resource Cleanup**: Always clean up resources (containers, files, connections)

4. **Assertions**: Use descriptive assertion messages for easier debugging

5. **Documentation**: Document complex test scenarios and setup requirements

See `/docs/test_plan.md` for the comprehensive test plan and coverage requirements.