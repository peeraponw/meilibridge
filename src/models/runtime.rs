use crate::config::{SourceConfig, SyncTaskConfig};
use crate::models::Progress;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug)]
pub struct RuntimeState {
    /// Active sources
    pub sources: Arc<RwLock<HashMap<String, SourceState>>>,

    /// Active sync tasks
    pub tasks: Arc<RwLock<HashMap<String, TaskState>>>,

    /// System metrics
    pub metrics: Arc<Metrics>,
}

impl RuntimeState {
    pub fn new() -> Self {
        Self {
            sources: Arc::new(RwLock::new(HashMap::new())),
            tasks: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(Metrics::new()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SourceState {
    pub id: String,
    pub config: SourceConfig,
    pub status: SourceStatus,
    pub connected_at: Option<DateTime<Utc>>,
    pub last_error: Option<ErrorInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SourceStatus {
    Connecting,
    Connected,
    Disconnected,
    Error,
}

#[derive(Debug, Clone)]
pub struct TaskState {
    pub id: String,
    pub config: SyncTaskConfig,
    pub status: TaskStatus,
    pub progress: Progress,
    pub metrics: TaskMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Starting,
    Running,
    Paused,
    Stopping,
    Stopped,
    Error,
}

#[derive(Debug, Clone, Default)]
pub struct TaskMetrics {
    pub events_per_second: f64,
    pub lag_seconds: f64,
    pub error_rate: f64,
    pub last_update: DateTime<Utc>,
}

impl TaskMetrics {
    pub fn new() -> Self {
        Self {
            last_update: Utc::now(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorInfo {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub timestamp: DateTime<Utc>,
}

impl ErrorInfo {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
            timestamp: Utc::now(),
        }
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

#[derive(Debug)]
pub struct Metrics {
    pub total_events: std::sync::atomic::AtomicU64,
    pub total_errors: std::sync::atomic::AtomicU64,
    pub total_bytes: std::sync::atomic::AtomicU64,
    pub start_time: DateTime<Utc>,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            total_events: std::sync::atomic::AtomicU64::new(0),
            total_errors: std::sync::atomic::AtomicU64::new(0),
            total_bytes: std::sync::atomic::AtomicU64::new(0),
            start_time: Utc::now(),
        }
    }

    pub fn record_event(&self, bytes: u64) {
        self.total_events
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.total_bytes
            .fetch_add(bytes, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn record_error(&self) {
        self.total_errors
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn uptime_seconds(&self) -> i64 {
        (Utc::now() - self.start_time).num_seconds()
    }
}
