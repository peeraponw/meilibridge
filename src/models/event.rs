use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Unique event ID
    pub id: EventId,

    /// Event type
    pub event_type: EventType,

    /// Source information
    pub source: EventSource,

    /// Event data
    pub data: EventData,

    /// Event metadata
    pub metadata: EventMetadata,

    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct EventId(pub String);

impl EventId {
    pub fn new() -> Self {
        EventId(uuid::Uuid::new_v4().to_string())
    }
}

impl Default for EventId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    Create,
    Update,
    Delete,
    Truncate,
    Schema,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSource {
    pub database: String,
    pub schema: String,
    pub table: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventData {
    /// Primary key value(s)
    pub key: serde_json::Value,

    /// Old values (for updates/deletes)
    pub old: Option<HashMap<String, serde_json::Value>>,

    /// New values (for creates/updates)
    pub new: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    /// Transaction ID
    pub transaction_id: Option<String>,

    /// Source position (LSN, binlog position, etc.)
    pub position: String,

    /// Custom metadata
    #[serde(flatten)]
    pub custom: HashMap<String, serde_json::Value>,
}

/// CDC Event that flows through the processing pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdcEvent {
    pub event_type: EventType,
    pub table: String,
    pub schema: String,
    pub data: HashMap<String, serde_json::Value>,
    pub timestamp: DateTime<Utc>,
    pub position: Option<super::Position>,
}

impl From<CdcEvent> for Event {
    fn from(cdc: CdcEvent) -> Self {
        let key = cdc
            .data
            .get("id")
            .or_else(|| cdc.data.get("_id"))
            .cloned()
            .unwrap_or(serde_json::Value::Null);

        let data = match cdc.event_type {
            EventType::Create => EventData {
                key: key.clone(),
                old: None,
                new: Some(cdc.data),
            },
            EventType::Update => EventData {
                key: key.clone(),
                old: None, // Will be populated by CDC adapter
                new: Some(cdc.data),
            },
            EventType::Delete => EventData {
                key: key.clone(),
                old: Some(cdc.data),
                new: None,
            },
            _ => EventData {
                key,
                old: None,
                new: None,
            },
        };

        Event {
            id: EventId::new(),
            event_type: cdc.event_type,
            source: EventSource {
                database: String::new(), // Will be populated by source
                schema: cdc.schema,
                table: cdc.table,
            },
            data,
            metadata: EventMetadata {
                transaction_id: None,
                position: String::new(), // Will be populated by CDC adapter
                custom: HashMap::new(),
            },
            timestamp: cdc.timestamp,
        }
    }
}
