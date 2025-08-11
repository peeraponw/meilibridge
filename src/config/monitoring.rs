use serde::{Deserialize, Serialize};

/// Monitoring configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MonitoringConfig {
    /// Enable metrics collection
    #[serde(default = "default_true")]
    pub metrics_enabled: bool,
    
    /// Metrics export interval in seconds
    #[serde(default = "default_metrics_interval")]
    pub metrics_interval_seconds: u64,
    
    /// Enable health checks
    #[serde(default = "default_true")]
    pub health_checks_enabled: bool,
    
    /// Health check interval in seconds
    #[serde(default = "default_health_check_interval")]
    pub health_check_interval_seconds: u64,
    
    /// Enable distributed tracing
    #[serde(default)]
    pub tracing_enabled: bool,
    
    /// Tracing backend (jaeger, zipkin, otlp)
    #[serde(default = "default_tracing_backend")]
    pub tracing_backend: String,
    
    /// Tracing endpoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracing_endpoint: Option<String>,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            metrics_enabled: default_true(),
            metrics_interval_seconds: default_metrics_interval(),
            health_checks_enabled: default_true(),
            health_check_interval_seconds: default_health_check_interval(),
            tracing_enabled: false,
            tracing_backend: default_tracing_backend(),
            tracing_endpoint: None,
        }
    }
}

fn default_true() -> bool { true }
fn default_metrics_interval() -> u64 { 60 }
fn default_health_check_interval() -> u64 { 30 }
fn default_tracing_backend() -> String { "otlp".to_string() }