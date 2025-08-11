use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Pipeline configuration for transformations and mappings
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct PipelineConfig {
    /// Filter configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<FilterConfig>,

    /// Transform configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transform: Option<TransformConfig>,

    /// Mapping configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mapping: Option<MappingConfig>,
}

/// Filter configuration
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct FilterConfig {
    /// Table filtering
    #[serde(default)]
    pub tables: TableFilter,

    /// Event type filtering
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_types: Option<Vec<String>>,

    /// Field conditions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conditions: Option<Vec<Condition>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct TableFilter {
    /// Tables to include
    #[serde(skip_serializing_if = "Option::is_none")]
    pub whitelist: Option<Vec<String>>,

    /// Tables to exclude
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blacklist: Option<Vec<String>>,
}

/// Transform configuration
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct TransformConfig {
    /// Field-specific transformations
    #[serde(default)]
    pub fields: HashMap<String, HashMap<String, FieldTransform>>,

    /// Global transformations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub global_transforms: Option<Vec<Transform>>,
}

/// Mapping configuration
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct MappingConfig {
    /// Table mappings
    #[serde(default)]
    pub tables: HashMap<String, TableMapping>,

    /// Strategy for unmapped fields: "include", "exclude", "prefix"
    #[serde(default = "default_unmapped_strategy")]
    pub unmapped_fields_strategy: String,

    /// Prefix for unmapped fields when strategy is "prefix"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unmapped_fields_prefix: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TableMapping {
    /// New table name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Field mappings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<HashMap<String, FieldMapping>>,
}

/// Field mapping types
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum FieldMapping {
    /// Simple field rename
    Simple(String),
    /// Complex mapping
    Complex {
        #[serde(skip_serializing_if = "Option::is_none")]
        to: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        default: Option<Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        sources: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        separator: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        path: Option<String>,
    },
}

/// Field transformation types
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FieldTransform {
    Rename { from: String, to: String },
    Convert { field: String, to_type: String },
    Extract { from: String, path: String, to: String },
    Compute { expression: String, to: String },
}

/// Global transformation types
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Transform {
    AddField { name: String, value: Value },
    RemoveField { name: String },
    AddTimestamp { field: String },
    Lowercase { fields: Vec<String> },
    Uppercase { fields: Vec<String> },
}

/// Filter condition types
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Condition {
    Equals { field: String, value: Value },
    NotEquals { field: String, value: Value },
    Contains { field: String, value: Value },
    GreaterThan { field: String, value: Value },
    LessThan { field: String, value: Value },
    In { field: String, values: Vec<Value> },
    IsNull { field: String },
    IsNotNull { field: String },
    And { conditions: Vec<Condition> },
    Or { conditions: Vec<Condition> },
}

fn default_unmapped_strategy() -> String {
    "include".to_string()
}