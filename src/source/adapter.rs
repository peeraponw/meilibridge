use crate::error::Result;
use crate::models::{Event, Position};
use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;

/// A stream of events from the source
pub type EventStream = Pin<Box<dyn Stream<Item = Result<Event>> + Send>>;

/// A stream of data for full sync
pub type DataStream = Pin<Box<dyn Stream<Item = Result<serde_json::Value>> + Send>>;

/// Trait for source adapters that provide CDC and full sync capabilities
#[async_trait]
pub trait SourceAdapter: Send + Sync {
    /// Connect to the source database
    async fn connect(&mut self) -> Result<()>;
    
    /// Get a stream of change events (CDC)
    async fn get_changes(&mut self) -> Result<EventStream>;
    
    /// Get full data for initial sync
    async fn get_full_data(&self, table: &str, batch_size: usize) -> Result<DataStream>;
    
    /// Get the current replication position
    async fn get_current_position(&self) -> Result<Position>;
    
    /// Acknowledge events up to the given position
    async fn acknowledge(&mut self, position: Position) -> Result<()>;
    
    /// Check if the adapter is connected
    fn is_connected(&self) -> bool;
    
    /// Disconnect from the source
    async fn disconnect(&mut self) -> Result<()>;
    
    /// Clone the adapter into a boxed trait object
    /// This should share the underlying connection pool to avoid duplicate connections
    fn clone_box(&self) -> Box<dyn SourceAdapter>;
}