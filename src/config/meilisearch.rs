use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MeilisearchConfig {
    /// API URL
    pub url: String,

    /// API key for authentication
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,

    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,

    /// Connection pool size
    #[serde(default = "default_connections")]
    pub max_connections: usize,

    /// Index settings template
    #[serde(default)]
    pub index_settings: IndexSettings,
    
    /// Default primary key field name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_key: Option<String>,
    
    /// Batch size for bulk operations
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
    
    /// Auto-create indexes if they don't exist
    #[serde(default = "default_auto_create_index")]
    pub auto_create_index: bool,
    
    /// Circuit breaker configuration
    #[serde(default)]
    pub circuit_breaker: CircuitBreakerConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CircuitBreakerConfig {
    /// Enable circuit breaker
    #[serde(default = "default_circuit_breaker_enabled")]
    pub enabled: bool,
    
    /// Error rate threshold (0.0 - 1.0)
    #[serde(default = "default_error_rate")]
    pub error_rate: f64,
    
    /// Minimum request count before evaluating
    #[serde(default = "default_min_request_count")]
    pub min_request_count: u64,
    
    /// Consecutive failures to open circuit
    #[serde(default = "default_consecutive_failures")]
    pub consecutive_failures: u64,
    
    /// Timeout before half-open state (seconds)
    #[serde(default = "default_circuit_timeout")]
    pub timeout_secs: u64,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            enabled: default_circuit_breaker_enabled(),
            error_rate: default_error_rate(),
            min_request_count: default_min_request_count(),
            consecutive_failures: default_consecutive_failures(),
            timeout_secs: default_circuit_timeout(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct IndexSettings {
    /// Searchable attributes
    pub searchable_attributes: Option<Vec<String>>,

    /// Displayed attributes
    pub displayed_attributes: Option<Vec<String>>,

    /// Filterable attributes
    pub filterable_attributes: Option<Vec<String>>,

    /// Sortable attributes
    pub sortable_attributes: Option<Vec<String>>,

    /// Ranking rules
    pub ranking_rules: Option<Vec<String>>,

    /// Stop words
    pub stop_words: Option<Vec<String>>,

    /// Synonyms
    pub synonyms: Option<HashMap<String, Vec<String>>>,
}

fn default_timeout() -> u64 {
    30
}

fn default_connections() -> usize {
    10
}

fn default_batch_size() -> usize {
    1000
}

fn default_auto_create_index() -> bool {
    true
}

fn default_circuit_breaker_enabled() -> bool {
    true
}

fn default_error_rate() -> f64 {
    0.5
}

fn default_min_request_count() -> u64 {
    10
}

fn default_consecutive_failures() -> u64 {
    5
}

fn default_circuit_timeout() -> u64 {
    60
}
