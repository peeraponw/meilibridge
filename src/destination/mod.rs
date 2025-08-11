pub mod adapter;
pub mod meilisearch;
pub mod circuit_breaker;

pub use adapter::{DestinationAdapter, SyncResult};
pub use circuit_breaker::{MeilisearchCircuitBreaker, CircuitBreakerError, CircuitState, CircuitBreakerBuilder};