use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RedisConfig {
    /// Redis URL
    pub url: String,

    /// Password for authentication
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,

    /// Database number
    #[serde(default)]
    pub database: u8,

    /// Key prefix for all operations
    #[serde(default = "default_key_prefix")]
    pub key_prefix: String,

    /// Connection pool settings
    #[serde(default)]
    pub pool: RedisPoolConfig,

    /// Checkpoint retention settings
    #[serde(default)]
    pub checkpoint_retention: CheckpointRetentionConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CheckpointRetentionConfig {
    /// Maximum number of checkpoints to keep per task
    #[serde(default = "default_max_checkpoints_per_task")]
    pub max_checkpoints_per_task: usize,

    /// Enable automatic cleanup on memory pressure
    #[serde(default = "default_cleanup_on_memory_pressure")]
    pub cleanup_on_memory_pressure: bool,

    /// Memory pressure threshold to trigger cleanup (percentage)
    #[serde(default = "default_memory_pressure_threshold")]
    pub memory_pressure_threshold: f64,
}

impl Default for CheckpointRetentionConfig {
    fn default() -> Self {
        Self {
            max_checkpoints_per_task: default_max_checkpoints_per_task(),
            cleanup_on_memory_pressure: default_cleanup_on_memory_pressure(),
            memory_pressure_threshold: default_memory_pressure_threshold(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RedisPoolConfig {
    #[serde(default = "default_redis_pool_size")]
    pub max_size: u32,

    #[serde(default = "default_redis_min_idle")]
    pub min_idle: u32,

    #[serde(default = "default_redis_timeout")]
    pub connection_timeout: u64,
}

impl Default for RedisPoolConfig {
    fn default() -> Self {
        Self {
            max_size: default_redis_pool_size(),
            min_idle: default_redis_min_idle(),
            connection_timeout: default_redis_timeout(),
        }
    }
}

fn default_key_prefix() -> String {
    "meilibridge".to_string()
}
fn default_redis_pool_size() -> u32 {
    10
}
fn default_redis_min_idle() -> u32 {
    1
}
fn default_redis_timeout() -> u64 {
    5
}
fn default_max_checkpoints_per_task() -> usize {
    10
}
fn default_cleanup_on_memory_pressure() -> bool {
    true
}
fn default_memory_pressure_threshold() -> f64 {
    80.0
}
