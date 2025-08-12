use crate::config::{FilterConfig, MappingConfig, TransformConfig};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SyncTaskConfig {
    /// Unique identifier for the task
    #[serde(default = "generate_id")]
    pub id: String,

    /// Source name (for multi-source support)
    #[serde(default = "default_source_name")]
    pub source_name: String,

    /// Source table name
    pub table: String,

    /// Target Meilisearch index
    pub index: String,

    /// Primary key field
    #[serde(default = "default_pk")]
    pub primary_key: String,

    /// Filter configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<FilterConfig>,

    /// Transform configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transform: Option<TransformConfig>,

    /// Mapping configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mapping: Option<MappingConfig>,

    /// Perform full sync on startup
    #[serde(default)]
    pub full_sync_on_start: Option<bool>,

    /// Sync options
    #[serde(default)]
    pub options: SyncOptions,

    /// Whether to auto-start this task
    #[serde(default = "default_true")]
    pub auto_start: Option<bool>,

    /// Soft delete configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub soft_delete: Option<SoftDeleteConfig>,
}

fn default_true() -> Option<bool> {
    Some(true)
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct FieldFilters {
    /// Include only these fields
    pub include: Option<Vec<String>>,

    /// Exclude these fields
    pub exclude: Option<Vec<String>>,

    /// Custom filter expressions
    pub expressions: Option<Vec<FilterExpression>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FilterExpression {
    pub field: String,
    pub operator: FilterOperator,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterOperator {
    Eq,
    Ne,
    Gt,
    Gte,
    Lt,
    Lte,
    In,
    NotIn,
    Like,
    Regex,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct SyncOptions {
    /// Batch size for inserts
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,

    /// Batch timeout in milliseconds
    #[serde(default = "default_batch_timeout")]
    pub batch_timeout_ms: u64,

    /// Enable deduplication
    #[serde(default)]
    pub deduplicate: bool,

    /// Retry configuration
    #[serde(default)]
    pub retry: SyncRetryConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SyncRetryConfig {
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    #[serde(default = "default_initial_delay")]
    pub initial_delay: u64,

    #[serde(default = "default_max_delay")]
    pub max_delay: u64,

    #[serde(default = "default_multiplier")]
    pub multiplier: f64,
}

impl Default for SyncRetryConfig {
    fn default() -> Self {
        Self {
            max_retries: default_max_retries(),
            initial_delay: default_initial_delay(),
            max_delay: default_max_delay(),
            multiplier: default_multiplier(),
        }
    }
}

fn default_pk() -> String {
    "id".to_string()
}
fn default_batch_size() -> usize {
    1000
}
fn default_batch_timeout() -> u64 {
    1000 // 1 second
}
fn generate_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

fn default_source_name() -> String {
    "primary".to_string()
}

/// Soft delete configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SoftDeleteConfig {
    /// Field name to check for soft delete status
    pub field: String,

    /// Values that indicate a soft delete
    pub delete_values: Vec<serde_json::Value>,

    /// Whether to handle soft deletes during full sync
    #[serde(default = "default_true_bool")]
    pub handle_on_full_sync: bool,

    /// Whether to handle soft deletes during CDC
    #[serde(default = "default_true_bool")]
    pub handle_on_cdc: bool,
}

fn default_true_bool() -> bool {
    true
}
fn default_max_retries() -> u32 {
    3
}
fn default_initial_delay() -> u64 {
    1
}
fn default_max_delay() -> u64 {
    60
}
fn default_multiplier() -> f64 {
    2.0
}
