use super::{CdcEvent, Position};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Events that flow through the streaming pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    /// CDC change event
    Cdc(CdcEvent),
    
    /// Full sync data event
    FullSync {
        table: String,
        data: Value,
    },
    
    /// Insert event
    Insert {
        table: String,
        new_data: Value,
        position: Option<Position>,
    },
    
    /// Update event
    Update {
        table: String,
        old_data: Value,
        new_data: Value,
        position: Option<Position>,
    },
    
    /// Delete event
    Delete {
        table: String,
        old_data: Value,
        position: Option<Position>,
    },
    
    /// Checkpoint event
    Checkpoint(Position),
    
    /// Heartbeat event
    Heartbeat,
}

/// Extended CDC event with position tracking
#[derive(Debug, Clone)]
pub struct CdcEventWithPosition {
    pub event: CdcEvent,
    pub position: Option<Position>,
}

impl CdcEventWithPosition {
    pub fn new(event: CdcEvent, position: Option<Position>) -> Self {
        Self { event, position }
    }
}