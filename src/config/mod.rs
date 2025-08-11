use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod api;
mod loader;
mod logging;
mod meilisearch;
mod redis;
mod source;
mod sync_task;
mod validation;
mod performance;
mod monitoring;
mod error_handling;
pub mod pipeline;

pub use api::*;
pub use loader::*;
pub use logging::*;
pub use meilisearch::*;
pub use redis::*;
pub use source::*;
pub use sync_task::*;
pub use pipeline::*;
pub use validation::*;
pub use performance::*;
pub use monitoring::*;
pub use error_handling::*;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Application metadata
    #[serde(default)]
    pub app: AppConfig,

    /// Data source configuration (single source for backward compatibility)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceConfig>,
    
    /// Multiple data sources configuration
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<NamedSourceConfig>,

    /// Meilisearch configuration
    pub meilisearch: MeilisearchConfig,

    /// Sync task definitions
    pub sync_tasks: Vec<SyncTaskConfig>,

    /// Redis configuration for state management
    pub redis: RedisConfig,

    /// API server configuration
    #[serde(default)]
    pub api: ApiConfig,

    /// Logging configuration
    #[serde(default)]
    pub logging: LoggingConfig,
    
    /// Monitoring configuration
    #[serde(default)]
    pub monitoring: MonitoringConfig,
    
    /// Error handling configuration
    #[serde(default)]
    pub error_handling: ErrorHandlingConfig,

    /// Plugin configurations
    #[serde(default)]
    pub plugins: PluginConfig,

    /// Feature flags
    #[serde(default)]
    pub features: FeatureFlags,
    
    /// Performance configuration
    #[serde(default)]
    pub performance: PerformanceConfig,
    
    /// Exactly-once delivery configuration
    #[serde(default)]
    pub exactly_once_delivery: ExactlyOnceDeliveryConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct AppConfig {
    #[serde(default = "default_name")]
    pub name: String,

    #[serde(default = "default_instance_id")]
    pub instance_id: String,

    #[serde(default)]
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct PluginConfig {
    #[serde(default)]
    pub directory: Option<String>,

    #[serde(default)]
    pub enabled: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct FeatureFlags {
    #[serde(default = "default_true")]
    pub auto_recovery: bool,

    #[serde(default = "default_true")]
    pub health_checks: bool,

    #[serde(default = "default_true")]
    pub metrics_export: bool,

    #[serde(default)]
    pub distributed_mode: bool,
}

fn default_name() -> String {
    "meilibridge".to_string()
}

fn default_instance_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExactlyOnceDeliveryConfig {
    /// Enable exactly-once delivery guarantees
    #[serde(default = "default_true")]
    pub enabled: bool,
    
    /// Size of the deduplication window (number of events to track)
    #[serde(default = "default_deduplication_window")]
    pub deduplication_window: usize,
    
    /// Transaction timeout in seconds
    #[serde(default = "default_transaction_timeout")]
    pub transaction_timeout_secs: u64,
    
    /// Enable two-phase commit protocol
    #[serde(default = "default_true")]
    pub two_phase_commit: bool,
    
    /// Save checkpoint before writing to Meilisearch (atomic)
    #[serde(default = "default_true")]
    pub checkpoint_before_write: bool,
}

impl Default for ExactlyOnceDeliveryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            deduplication_window: default_deduplication_window(),
            transaction_timeout_secs: default_transaction_timeout(),
            two_phase_commit: true,
            checkpoint_before_write: true,
        }
    }
}

fn default_deduplication_window() -> usize {
    10000
}

fn default_transaction_timeout() -> u64 {
    30
}
