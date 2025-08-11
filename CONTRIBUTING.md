# Contributing to MeiliBridge

First off, thank you for considering contributing to MeiliBridge! It's people like you that make MeiliBridge such a great tool. We welcome contributions from everyone, regardless of their experience level.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [How Can I Contribute?](#how-can-i-contribute)
- [Development Setup](#development-setup)
- [Development Workflow](#development-workflow)
- [Style Guidelines](#style-guidelines)
- [Testing](#testing)
- [Documentation](#documentation)
- [Community](#community)

## Code of Conduct

This project and everyone participating in it is governed by our [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code. Please report unacceptable behavior to [conduct@meilibridge.dev](mailto:conduct@meilibridge.dev).

## Getting Started

Before you begin:
- Have you read the [documentation](docs/)?
- Check if your issue/idea already exists in [GitHub Issues](https://github.com/binary-touch/meilibridge/issues)
- For major changes, open an issue first to discuss what you would like to change

## How Can I Contribute?

### Reporting Bugs

Before creating bug reports, please check existing issues as you might find out that you don't need to create one. When you are creating a bug report, please include as many details as possible:

**Bug Report Template:**
```markdown
**Describe the bug**
A clear and concise description of what the bug is.

**To Reproduce**
Steps to reproduce the behavior:
1. Configure '...'
2. Run command '....'
3. See error

**Expected behavior**
What you expected to happen.

**Environment:**
 - OS: [e.g. Ubuntu 22.04]
 - MeiliBridge version: [e.g. 0.1.0]
 - PostgreSQL version: [e.g. 14.5]
 - Meilisearch version: [e.g. 1.3.0]

**Logs**
```
Paste relevant logs here
```

**Additional context**
Add any other context about the problem here.
```

### Suggesting Enhancements

Enhancement suggestions are tracked as GitHub issues. When creating an enhancement suggestion, please include:

- **Use case** - Explain why this enhancement would be useful
- **Proposed solution** - Describe the solution you'd like
- **Alternatives** - Describe alternatives you've considered
- **Additional context** - Add any other context or screenshots

### Your First Code Contribution

Unsure where to begin? You can start by looking through these issues:

- [Good first issues](https://github.com/binary-touch/meilibridge/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22) - issues which should only require a few lines of code
- [Help wanted issues](https://github.com/binary-touch/meilibridge/issues?q=is%3Aissue+is%3Aopen+label%3A%22help+wanted%22) - issues which need extra attention

## Development Setup

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- Docker and Docker Compose (for running dependencies)
- Git

### Setting Up Your Development Environment

1. **Fork and clone the repository**
   ```bash
   git clone https://github.com/YOUR_USERNAME/meilibridge.git
   cd meilibridge
   ```

2. **Install Rust dependencies**
   ```bash
   cargo build
   ```

3. **Start development dependencies**
   ```bash
   # Start PostgreSQL, Meilisearch, and Redis
   make docker-up
   
   # Or using docker-compose directly
   docker-compose -f docker/docker-compose.dev.yml up -d
   ```

4. **Copy the example configuration**
   ```bash
   cp config.example.yaml config.yaml
   ```

5. **Run the tests**
   ```bash
   cargo test
   ```

6. **Run the application**
   ```bash
   cargo run
   ```

### Useful Make Commands

```bash
make help          # Show all available commands
make build         # Build the project
make test          # Run all tests
make fmt           # Format code
make lint          # Run clippy linter
make docker-up     # Start development dependencies
make docker-down   # Stop development dependencies
make clean         # Clean build artifacts
```

## Development Workflow

### 1. Create a Feature Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/issue-number-description
```

### 2. Make Your Changes

- Write clean, idiomatic Rust code
- Follow the existing code style
- Add tests for new functionality
- Update documentation as needed

### 3. Test Your Changes

```bash
# Run unit tests
cargo test

# Run integration tests
cargo test --test '*' -- --test-threads=1

# Run specific test
cargo test test_name

# Run with logging
RUST_LOG=debug cargo test

# Check for common mistakes
cargo clippy -- -D warnings

# Format your code
cargo fmt
```

### 4. Commit Your Changes

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```bash
# Features
git commit -m "feat: add support for MySQL source adapter"

# Bug fixes
git commit -m "fix: handle connection timeout in CDC reader"

# Documentation
git commit -m "docs: update configuration examples"

# Performance improvements
git commit -m "perf: optimize batch processing for large datasets"

# Tests
git commit -m "test: add integration tests for checkpoint recovery"

# Refactoring
git commit -m "refactor: extract common retry logic to utils"
```

### 5. Push and Create a Pull Request

```bash
git push origin feature/your-feature-name
```

Then create a Pull Request on GitHub with:
- Clear title following conventional commits
- Description of what changed and why
- Link to related issue(s)
- Screenshots/logs if applicable

## Style Guidelines

### Rust Code Style

- Use `cargo fmt` to format code
- Use `cargo clippy` to catch common mistakes
- Follow [Rust naming conventions](https://rust-lang.github.io/api-guidelines/naming.html)
- Write descriptive variable names
- Add comments for complex logic
- Use `Result<T, Error>` for error handling
- Prefer `&str` over `String` for function parameters
- Use `Arc<RwLock<T>>` for shared mutable state

### Example Code Style

```rust
use crate::error::{MeiliBridgeError, Result};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Processes events from the source adapter
/// 
/// # Arguments
/// * `events` - Vector of events to process
/// * `config` - Processing configuration
/// 
/// # Returns
/// * `Result<ProcessedBatch>` - Processed events or error
pub async fn process_events(
    events: Vec<Event>,
    config: &ProcessConfig,
) -> Result<ProcessedBatch> {
    // Validate input
    if events.is_empty() {
        return Err(MeiliBridgeError::Validation(
            "No events to process".to_string()
        ));
    }
    
    // Process events
    let processed = events
        .into_iter()
        .filter(|e| should_process(e, config))
        .map(|e| transform_event(e, config))
        .collect::<Result<Vec<_>>>()?;
    
    Ok(ProcessedBatch::new(processed))
}
```

### Documentation Style

- Add module-level documentation
- Document public APIs with examples
- Use markdown formatting in doc comments
- Include error conditions in documentation

```rust
//! Event processing module
//! 
//! This module handles the transformation and filtering of CDC events
//! before they are sent to the destination.

/// Transforms an event according to the configuration
/// 
/// # Example
/// ```rust
/// let event = Event::Insert { 
///     table: "users".to_string(),
///     data: json!({"id": 1, "name": "Alice"})
/// };
/// let transformed = transform_event(event, &config)?;
/// ```
/// 
/// # Errors
/// Returns `MeiliBridgeError::Transform` if transformation fails
pub fn transform_event(event: Event, config: &Config) -> Result<Event> {
    // Implementation
}
```

## Testing

### Test Organization

```
tests/
├── unit/           # Unit tests for individual components
├── integration/    # Integration tests with real dependencies
└── common/         # Shared test utilities
```

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_event_filter() {
        let filter = EventFilter::new(config);
        let event = create_test_event();
        
        assert!(filter.should_process(&event));
    }
    
    #[tokio::test]
    async fn test_async_processing() {
        let processor = Processor::new();
        let result = processor.process(event).await;
        
        assert!(result.is_ok());
    }
}
```

### Test Coverage

- Aim for >80% test coverage
- Test both success and error paths
- Include edge cases
- Test with realistic data

## Documentation

### Code Documentation

- Document all public APIs
- Include examples in documentation
- Document error conditions
- Keep documentation up-to-date with code

### Project Documentation

When adding new features, update:
- API documentation in `docs/api-development.md`
- Configuration examples in `docs/configuration-architecture.md`
- Getting started guide if needed
- CHANGELOG.md with your changes

### README Updates

Update README.md if you:
- Add new configuration options
- Change installation process
- Add new features
- Update system requirements

## Pull Request Process

1. **Before Submitting**
   - Ensure all tests pass
   - Run `cargo fmt` and `cargo clippy`
   - Update documentation
   - Add entries to CHANGELOG.md

2. **PR Requirements**
   - Clear, descriptive title
   - Link to related issue(s)
   - Description of changes
   - Test results/screenshots if applicable

3. **Review Process**
   - At least one maintainer approval required
   - All CI checks must pass
   - Address review feedback promptly
   - Keep PR updated with main branch

4. **After Merge**
   - Delete your feature branch
   - Update your local main branch
   - Celebrate! 🎉

## Community

### Getting Help

- 💬 [GitHub Discussions](https://github.com/binary-touch/meilibridge/discussions) - Ask questions
- 📧 [Email](mailto:support@meilibridge.dev) - General inquiries
- 🐛 [Issues](https://github.com/binary-touch/meilibridge/issues) - Bug reports and features

### Staying Updated

- Watch the repository for updates
- Join our community discussions
- Follow the [roadmap](tasks/roadmap.md)

## Recognition

Contributors are recognized in:
- The git history
- Release notes
- Special thanks in major releases

## Questions?

Feel free to:
- Open an issue for clarification
- Ask in GitHub Discussions
- Reach out to maintainers

Thank you for contributing to MeiliBridge! 🚀