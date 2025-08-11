use serde::{Deserialize, Serialize};

/// Error handling configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ErrorHandlingConfig {
    /// Retry configuration
    #[serde(default)]
    pub retry: RetryConfig,
    
    /// Dead letter queue configuration
    #[serde(default)]
    pub dead_letter_queue: DeadLetterQueueConfig,
    
    /// Circuit breaker configuration
    #[serde(default)]
    pub circuit_breaker: ErrorHandlingCircuitBreakerConfig,
}

impl Default for ErrorHandlingConfig {
    fn default() -> Self {
        Self {
            retry: RetryConfig::default(),
            dead_letter_queue: DeadLetterQueueConfig::default(),
            circuit_breaker: ErrorHandlingCircuitBreakerConfig::default(),
        }
    }
}

/// Retry configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RetryConfig {
    /// Enable retries
    #[serde(default = "default_true")]
    pub enabled: bool,
    
    /// Maximum retry attempts
    #[serde(default = "default_max_retries")]
    pub max_attempts: u32,
    
    /// Initial backoff in milliseconds
    #[serde(default = "default_initial_backoff_ms")]
    pub initial_backoff_ms: u64,
    
    /// Maximum backoff in milliseconds
    #[serde(default = "default_max_backoff_ms")]
    pub max_backoff_ms: u64,
    
    /// Backoff multiplier
    #[serde(default = "default_backoff_multiplier")]
    pub backoff_multiplier: f64,
    
    /// Jitter factor (0.0 to 1.0)
    #[serde(default = "default_jitter_factor")]
    pub jitter_factor: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            max_attempts: default_max_retries(),
            initial_backoff_ms: default_initial_backoff_ms(),
            max_backoff_ms: default_max_backoff_ms(),
            backoff_multiplier: default_backoff_multiplier(),
            jitter_factor: default_jitter_factor(),
        }
    }
}

/// Dead letter queue configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeadLetterQueueConfig {
    /// Enable dead letter queue
    #[serde(default = "default_true")]
    pub enabled: bool,
    
    /// Storage backend (memory, redis)
    #[serde(default = "default_dlq_storage")]
    pub storage: String,
    
    /// Maximum entries per task
    #[serde(default = "default_max_entries")]
    pub max_entries_per_task: usize,
    
    /// Retention period in hours
    #[serde(default = "default_retention_hours")]
    pub retention_hours: u64,
    
    /// Auto-reprocess interval in minutes (0 = disabled)
    #[serde(default)]
    pub auto_reprocess_interval_minutes: u64,
}

impl Default for DeadLetterQueueConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            storage: default_dlq_storage(),
            max_entries_per_task: default_max_entries(),
            retention_hours: default_retention_hours(),
            auto_reprocess_interval_minutes: 0,
        }
    }
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ErrorHandlingCircuitBreakerConfig {
    /// Enable circuit breaker
    #[serde(default)]
    pub enabled: bool,
    
    /// Failure threshold percentage (0-100)
    #[serde(default = "default_failure_threshold")]
    pub failure_threshold_percent: u8,
    
    /// Minimum number of requests before evaluation
    #[serde(default = "default_min_requests")]
    pub min_requests: u32,
    
    /// Reset timeout in seconds
    #[serde(default = "default_reset_timeout")]
    pub reset_timeout_seconds: u64,
    
    /// Half-open max requests
    #[serde(default = "default_half_open_requests")]
    pub half_open_max_requests: u32,
}

impl Default for ErrorHandlingCircuitBreakerConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            failure_threshold_percent: default_failure_threshold(),
            min_requests: default_min_requests(),
            reset_timeout_seconds: default_reset_timeout(),
            half_open_max_requests: default_half_open_requests(),
        }
    }
}

// Default value functions
fn default_true() -> bool { true }
fn default_max_retries() -> u32 { 3 }
fn default_initial_backoff_ms() -> u64 { 100 }
fn default_max_backoff_ms() -> u64 { 30000 }
fn default_backoff_multiplier() -> f64 { 2.0 }
fn default_jitter_factor() -> f64 { 0.1 }
fn default_dlq_storage() -> String { "memory".to_string() }
fn default_max_entries() -> usize { 10000 }
fn default_retention_hours() -> u64 { 24 }
fn default_failure_threshold() -> u8 { 50 }
fn default_min_requests() -> u32 { 10 }
fn default_reset_timeout() -> u64 { 60 }
fn default_half_open_requests() -> u32 { 5 }