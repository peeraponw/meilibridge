pub mod adapter;
pub mod circuit_breaker;
pub mod meilisearch;

pub use adapter::{DestinationAdapter, SyncResult};
pub use circuit_breaker::{
    CircuitBreakerBuilder, CircuitBreakerError, CircuitState, MeilisearchCircuitBreaker,
};
