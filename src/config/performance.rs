use serde::{Deserialize, Serialize};

/// Performance configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PerformanceConfig {
    /// Connection pool configuration
    #[serde(default)]
    pub connection_pool: ConnectionPoolConfig,
    
    /// Batch processing configuration
    #[serde(default)]
    pub batch_processing: BatchProcessingConfig,
    
    /// Parallel processing configuration
    #[serde(default)]
    pub parallel_processing: ParallelProcessingConfig,
    
    /// Memory limits
    #[serde(default)]
    pub memory: MemoryConfig,
    
    /// Rate limiting
    #[serde(default)]
    pub rate_limits: RateLimitConfig,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            connection_pool: ConnectionPoolConfig::default(),
            batch_processing: BatchProcessingConfig::default(),
            parallel_processing: ParallelProcessingConfig::default(),
            memory: MemoryConfig::default(),
            rate_limits: RateLimitConfig::default(),
        }
    }
}

/// Connection pool settings
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConnectionPoolConfig {
    /// Maximum connections in the pool
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    
    /// Minimum connections to maintain
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,
    
    /// Connection timeout in seconds
    #[serde(default = "default_connection_timeout")]
    pub connection_timeout: u64,
    
    /// Idle timeout in seconds
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout: u64,
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            max_connections: default_max_connections(),
            min_connections: default_min_connections(),
            connection_timeout: default_connection_timeout(),
            idle_timeout: default_idle_timeout(),
        }
    }
}

/// Batch processing settings
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BatchProcessingConfig {
    /// Default batch size
    #[serde(default = "default_batch_size")]
    pub default_batch_size: usize,
    
    /// Maximum batch size
    #[serde(default = "default_max_batch_size")]
    pub max_batch_size: usize,
    
    /// Minimum batch size
    #[serde(default = "default_min_batch_size")]
    pub min_batch_size: usize,
    
    /// Batch timeout in milliseconds
    #[serde(default = "default_batch_timeout_ms")]
    pub batch_timeout_ms: u64,
    
    /// Enable adaptive batching
    #[serde(default = "default_true")]
    pub adaptive_batching: bool,
    
    /// Adaptive batching configuration
    #[serde(default)]
    pub adaptive_config: AdaptiveBatchingConfig,
}

impl Default for BatchProcessingConfig {
    fn default() -> Self {
        Self {
            default_batch_size: default_batch_size(),
            max_batch_size: default_max_batch_size(),
            min_batch_size: default_min_batch_size(),
            batch_timeout_ms: default_batch_timeout_ms(),
            adaptive_batching: default_true(),
            adaptive_config: AdaptiveBatchingConfig::default(),
        }
    }
}

/// Adaptive batching configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AdaptiveBatchingConfig {
    /// Target latency in milliseconds
    #[serde(default = "default_target_latency_ms")]
    pub target_latency_ms: u64,
    
    /// Adjustment factor (0.0 to 1.0)
    #[serde(default = "default_adjustment_factor")]
    pub adjustment_factor: f64,
    
    /// Window size for averaging metrics
    #[serde(default = "default_metric_window_size")]
    pub metric_window_size: usize,
    
    /// Minimum time between adjustments (ms)
    #[serde(default = "default_adjustment_interval_ms")]
    pub adjustment_interval_ms: u64,
    
    /// Memory pressure threshold (percentage)
    #[serde(default = "default_memory_pressure_threshold")]
    pub memory_pressure_threshold: f64,
    
    /// Enable per-table optimization
    #[serde(default = "default_true")]
    pub per_table_optimization: bool,
}

impl Default for AdaptiveBatchingConfig {
    fn default() -> Self {
        Self {
            target_latency_ms: default_target_latency_ms(),
            adjustment_factor: default_adjustment_factor(),
            metric_window_size: default_metric_window_size(),
            adjustment_interval_ms: default_adjustment_interval_ms(),
            memory_pressure_threshold: default_memory_pressure_threshold(),
            per_table_optimization: default_true(),
        }
    }
}

/// Parallel processing configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ParallelProcessingConfig {
    /// Enable parallel processing
    #[serde(default = "default_true")]
    pub enabled: bool,
    
    /// Number of worker threads per table
    #[serde(default = "default_workers_per_table")]
    pub workers_per_table: usize,
    
    /// Maximum concurrent events across all workers
    #[serde(default = "default_max_concurrent_events")]
    pub max_concurrent_events: usize,
    
    /// Enable work stealing between tables
    #[serde(default = "default_true")]
    pub work_stealing: bool,
    
    /// Work stealing check interval in milliseconds
    #[serde(default = "default_steal_interval_ms")]
    pub work_steal_interval_ms: u64,
    
    /// Queue size threshold for work stealing
    #[serde(default = "default_steal_threshold")]
    pub work_steal_threshold: usize,
}

impl Default for ParallelProcessingConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            workers_per_table: default_workers_per_table(),
            max_concurrent_events: default_max_concurrent_events(),
            work_stealing: default_true(),
            work_steal_interval_ms: default_steal_interval_ms(),
            work_steal_threshold: default_steal_threshold(),
        }
    }
}

/// Memory configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MemoryConfig {
    /// Maximum memory for event queue (MB)
    #[serde(default = "default_max_queue_memory_mb")]
    pub max_queue_memory_mb: usize,
    
    /// Maximum memory for checkpoint cache (MB)
    #[serde(default = "default_max_checkpoint_memory_mb")]
    pub max_checkpoint_memory_mb: usize,
    
    /// Enable memory pressure monitoring
    #[serde(default = "default_true")]
    pub monitor_memory_pressure: bool,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            max_queue_memory_mb: default_max_queue_memory_mb(),
            max_checkpoint_memory_mb: default_max_checkpoint_memory_mb(),
            monitor_memory_pressure: default_true(),
        }
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RateLimitConfig {
    /// Enable rate limiting
    #[serde(default)]
    pub enabled: bool,
    
    /// Maximum events per second (0 = unlimited)
    #[serde(default)]
    pub max_events_per_second: u64,
    
    /// Maximum API requests per second
    #[serde(default = "default_max_api_requests")]
    pub max_api_requests_per_second: u64,
    
    /// Burst capacity
    #[serde(default = "default_burst_capacity")]
    pub burst_capacity: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_events_per_second: 0,
            max_api_requests_per_second: default_max_api_requests(),
            burst_capacity: default_burst_capacity(),
        }
    }
}

// Default value functions
fn default_max_connections() -> u32 { 20 }
fn default_min_connections() -> u32 { 5 }
fn default_connection_timeout() -> u64 { 30 }
fn default_idle_timeout() -> u64 { 600 }
fn default_batch_size() -> usize { 100 }
fn default_max_batch_size() -> usize { 1000 }
fn default_min_batch_size() -> usize { 10 }
fn default_batch_timeout_ms() -> u64 { 5000 }
fn default_true() -> bool { true }
fn default_workers_per_table() -> usize { 4 }
fn default_max_concurrent_events() -> usize { 1000 }
fn default_steal_interval_ms() -> u64 { 100 }
fn default_steal_threshold() -> usize { 50 }
fn default_max_queue_memory_mb() -> usize { 512 }
fn default_max_checkpoint_memory_mb() -> usize { 128 }
fn default_max_api_requests() -> u64 { 1000 }
fn default_burst_capacity() -> u64 { 100 }
fn default_target_latency_ms() -> u64 { 1000 }
fn default_adjustment_factor() -> f64 { 0.2 }
fn default_metric_window_size() -> usize { 10 }
fn default_adjustment_interval_ms() -> u64 { 5000 }
fn default_memory_pressure_threshold() -> f64 { 80.0 }