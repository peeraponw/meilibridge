//! Exactly-once delivery implementation for MeiliBridge
//! 
//! This module provides transaction-based checkpointing and event deduplication
//! to guarantee exactly-once delivery semantics.

pub mod checkpoint;
pub mod deduplication;
pub mod transaction;

pub use checkpoint::TransactionalCheckpoint;
pub use deduplication::{EventDeduplicator, DeduplicationKey};
pub use transaction::{TwoPhaseCommit, TransactionCoordinator};

use crate::error::Result;
use crate::models::Position;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Configuration for exactly-once delivery
#[derive(Debug, Clone)]
pub struct ExactlyOnceConfig {
    /// Enable exactly-once delivery guarantees
    pub enabled: bool,
    
    /// Deduplication window size (number of events to track)
    pub deduplication_window: usize,
    
    /// Transaction timeout in seconds
    pub transaction_timeout_secs: u64,
    
    /// Enable two-phase commit protocol
    pub two_phase_commit: bool,
    
    /// Checkpoint before write (atomic with event processing)
    pub checkpoint_before_write: bool,
}

impl Default for ExactlyOnceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            deduplication_window: 10000,
            transaction_timeout_secs: 30,
            two_phase_commit: true,
            checkpoint_before_write: true,
        }
    }
}

/// Manages exactly-once delivery guarantees
pub struct ExactlyOnceManager {
    config: ExactlyOnceConfig,
    deduplicator: Arc<RwLock<EventDeduplicator>>,
    pub transaction_coordinator: Arc<TransactionCoordinator>,
}

impl ExactlyOnceManager {
    pub fn new(config: ExactlyOnceConfig) -> Self {
        let deduplicator = Arc::new(RwLock::new(
            EventDeduplicator::new(config.deduplication_window)
        ));
        
        let transaction_coordinator = Arc::new(
            TransactionCoordinator::new(config.transaction_timeout_secs)
        );
        
        Self {
            config,
            deduplicator,
            transaction_coordinator,
        }
    }
    
    /// Check if an event has already been processed
    pub async fn is_duplicate(&self, key: &DeduplicationKey) -> Result<bool> {
        if !self.config.enabled {
            return Ok(false);
        }
        
        let dedup = self.deduplicator.read().await;
        Ok(dedup.contains(key))
    }
    
    /// Mark an event as processed
    pub async fn mark_processed(&self, key: DeduplicationKey) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }
        
        let mut dedup = self.deduplicator.write().await;
        dedup.add(key);
        Ok(())
    }
    
    /// Begin a new transaction
    pub async fn begin_transaction(&self, task_id: &str) -> Result<String> {
        if !self.config.two_phase_commit {
            return Ok(String::new());
        }
        
        self.transaction_coordinator.begin(task_id).await
    }
    
    /// Prepare phase of two-phase commit
    pub async fn prepare(&self, transaction_id: &str, position: Position) -> Result<bool> {
        if !self.config.two_phase_commit {
            return Ok(true);
        }
        
        self.transaction_coordinator.prepare(transaction_id, position).await
    }
    
    /// Commit phase of two-phase commit
    pub async fn commit(&self, transaction_id: &str) -> Result<()> {
        if !self.config.two_phase_commit {
            return Ok(());
        }
        
        self.transaction_coordinator.commit(transaction_id).await
    }
    
    /// Rollback a transaction
    pub async fn rollback(&self, transaction_id: &str) -> Result<()> {
        if !self.config.two_phase_commit {
            return Ok(());
        }
        
        self.transaction_coordinator.rollback(transaction_id).await
    }
}