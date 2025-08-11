use meilibridge::pipeline::cdc_coordinator::CdcCoordinator;
use meilibridge::models::stream_event::Event;
use meilibridge::models::{CdcEvent, EventType, Position};
use meilibridge::source::adapter::{SourceAdapter, EventStream, DataStream};
use meilibridge::error::{MeiliBridgeError, Result};
use async_trait::async_trait;
use futures::stream;
use tokio::sync::{mpsc, watch};
use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json::json;

// Mock source adapter for testing
struct MockSourceAdapter {
    events: Arc<Mutex<Vec<Event>>>,
    stream_created: Arc<Mutex<bool>>,
}

impl MockSourceAdapter {
    fn new(events: Vec<Event>) -> Self {
        Self {
            events: Arc::new(Mutex::new(events)),
            stream_created: Arc::new(Mutex::new(false)),
        }
    }
}

#[async_trait]
impl SourceAdapter for MockSourceAdapter {
    async fn connect(&mut self) -> Result<()> {
        Ok(())
    }
    
    async fn get_changes(&mut self) -> Result<EventStream> {
        *self.stream_created.lock().await = true;
        let events = self.events.lock().await.clone();
        Ok(Box::pin(stream::iter(events.into_iter().map(Ok))))
    }
    
    async fn get_full_data(&self, _table: &str, _batch_size: usize) -> Result<DataStream> {
        Ok(Box::pin(stream::empty()))
    }
    
    async fn get_current_position(&self) -> Result<Position> {
        Ok(Position::PostgreSQL { lsn: "0/0".to_string() })
    }
    
    async fn acknowledge(&mut self, _position: Position) -> Result<()> {
        Ok(())
    }
    
    fn is_connected(&self) -> bool {
        true
    }
    
    async fn disconnect(&mut self) -> Result<()> {
        Ok(())
    }
    
    fn clone_box(&self) -> Box<dyn SourceAdapter> {
        Box::new(MockSourceAdapter {
            events: self.events.clone(),
            stream_created: self.stream_created.clone(),
        })
    }
}

fn create_test_cdc_event(table: &str, schema: &str) -> Event {
    use chrono::Utc;
    use std::collections::HashMap;
    
    let mut data = HashMap::new();
    data.insert("id".to_string(), json!(1));
    
    Event::Cdc(CdcEvent {
        event_type: EventType::Create,
        table: table.to_string(),
        schema: schema.to_string(),
        data,
        timestamp: Utc::now(),
        position: Some(Position::PostgreSQL { lsn: "0/0".to_string() }),
    })
}

#[tokio::test]
async fn test_coordinator_creation() {
    let adapter = Box::new(MockSourceAdapter::new(vec![]));
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    
    let coordinator = CdcCoordinator::new(adapter, shutdown_rx);
    assert!(!coordinator.is_paused().await);
    
    drop(shutdown_tx);
}

#[tokio::test]
async fn test_register_and_unregister_task() {
    let adapter = Box::new(MockSourceAdapter::new(vec![]));
    let (_shutdown_tx, shutdown_rx) = watch::channel(false);
    
    let mut coordinator = CdcCoordinator::new(adapter, shutdown_rx);
    let (tx, _rx) = mpsc::channel(10);
    
    // Register task
    coordinator.register_task("users".to_string(), tx.clone());
    
    // Unregister task
    coordinator.unregister_task("users");
}

#[tokio::test]
async fn test_pause_and_resume_all() {
    let adapter = Box::new(MockSourceAdapter::new(vec![]));
    let (_shutdown_tx, shutdown_rx) = watch::channel(false);
    
    let coordinator = CdcCoordinator::new(adapter, shutdown_rx);
    
    // Initially not paused
    assert!(!coordinator.is_paused().await);
    
    // Pause all
    coordinator.pause_all().await;
    assert!(coordinator.is_paused().await);
    
    // Resume all
    coordinator.resume_all().await;
    assert!(!coordinator.is_paused().await);
}

#[tokio::test]
async fn test_pause_and_resume_specific_table() {
    let adapter = Box::new(MockSourceAdapter::new(vec![]));
    let (_shutdown_tx, shutdown_rx) = watch::channel(false);
    
    let coordinator = CdcCoordinator::new(adapter, shutdown_rx);
    
    // Initially no tables paused
    assert_eq!(coordinator.get_paused_tables().await.len(), 0);
    
    // Pause specific table
    coordinator.pause_table("users").await;
    let paused = coordinator.get_paused_tables().await;
    assert_eq!(paused.len(), 1);
    assert!(paused.contains(&"users".to_string()));
    
    // Resume table
    coordinator.resume_table("users").await;
    assert_eq!(coordinator.get_paused_tables().await.len(), 0);
}

#[tokio::test]
async fn test_event_distribution_to_registered_task() {
    let event = create_test_cdc_event("users", "public");
    let adapter = Box::new(MockSourceAdapter::new(vec![event.clone()]));
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    
    let mut coordinator = CdcCoordinator::new(adapter, shutdown_rx);
    let (tx, mut rx) = mpsc::channel(10);
    
    // Register task for "users" table
    coordinator.register_task("users".to_string(), tx);
    
    // Start coordinator in background
    let coordinator_handle = tokio::spawn(async move {
        let _ = coordinator.run().await;
    });
    
    // Wait for event to be received
    tokio::time::timeout(std::time::Duration::from_millis(100), async {
        let received = rx.recv().await.unwrap().unwrap();
        match received {
            Event::Cdc(cdc_event) => {
                assert_eq!(cdc_event.table, "users");
                assert_eq!(cdc_event.schema, "public");
            }
            _ => panic!("Expected CDC event"),
        }
    }).await.expect("Should receive event");
    
    // Shutdown
    shutdown_tx.send(true).unwrap();
    let _ = tokio::time::timeout(std::time::Duration::from_millis(100), coordinator_handle).await;
}

#[tokio::test]
async fn test_event_distribution_with_schema_prefix() {
    let event = create_test_cdc_event("users", "public");
    let adapter = Box::new(MockSourceAdapter::new(vec![event.clone()]));
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    
    let mut coordinator = CdcCoordinator::new(adapter, shutdown_rx);
    let (tx, mut rx) = mpsc::channel(10);
    
    // Register task for "public.users" table
    coordinator.register_task("public.users".to_string(), tx);
    
    // Start coordinator in background
    let coordinator_handle = tokio::spawn(async move {
        let _ = coordinator.run().await;
    });
    
    // Wait for event to be received
    tokio::time::timeout(std::time::Duration::from_millis(100), async {
        let received = rx.recv().await.unwrap().unwrap();
        match received {
            Event::Cdc(cdc_event) => {
                assert_eq!(cdc_event.table, "users");
                assert_eq!(cdc_event.schema, "public");
            }
            _ => panic!("Expected CDC event"),
        }
    }).await.expect("Should receive event");
    
    // Shutdown
    shutdown_tx.send(true).unwrap();
    let _ = tokio::time::timeout(std::time::Duration::from_millis(100), coordinator_handle).await;
}

#[tokio::test]
async fn test_event_not_distributed_when_paused() {
    let event = create_test_cdc_event("users", "public");
    let adapter = Box::new(MockSourceAdapter::new(vec![event.clone()]));
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    
    let mut coordinator = CdcCoordinator::new(adapter, shutdown_rx);
    let (tx, mut rx) = mpsc::channel(10);
    
    // Pause all events
    coordinator.pause_all().await;
    
    // Register task
    coordinator.register_task("users".to_string(), tx);
    
    // Start coordinator in background
    let coordinator_handle = tokio::spawn(async move {
        let _ = coordinator.run().await;
    });
    
    // Should not receive any events
    let result = tokio::time::timeout(std::time::Duration::from_millis(50), rx.recv()).await;
    assert!(result.is_err(), "Should not receive event when paused");
    
    // Shutdown
    shutdown_tx.send(true).unwrap();
    let _ = tokio::time::timeout(std::time::Duration::from_millis(100), coordinator_handle).await;
}

#[tokio::test]
async fn test_event_not_distributed_for_paused_table() {
    let event = create_test_cdc_event("users", "public");
    let adapter = Box::new(MockSourceAdapter::new(vec![event.clone()]));
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    
    let mut coordinator = CdcCoordinator::new(adapter, shutdown_rx);
    let (tx, mut rx) = mpsc::channel(10);
    
    // Pause specific table
    coordinator.pause_table("users").await;
    
    // Register task
    coordinator.register_task("users".to_string(), tx);
    
    // Start coordinator in background
    let coordinator_handle = tokio::spawn(async move {
        let _ = coordinator.run().await;
    });
    
    // Should not receive any events
    let result = tokio::time::timeout(std::time::Duration::from_millis(50), rx.recv()).await;
    assert!(result.is_err(), "Should not receive event for paused table");
    
    // Shutdown
    shutdown_tx.send(true).unwrap();
    let _ = tokio::time::timeout(std::time::Duration::from_millis(100), coordinator_handle).await;
}

#[tokio::test]
async fn test_coordinator_handles_adapter_errors() {
    // Create adapter that returns error
    struct ErrorAdapter;
    
    #[async_trait]
    impl SourceAdapter for ErrorAdapter {
        async fn connect(&mut self) -> Result<()> {
            Ok(())
        }
        
        async fn get_changes(&mut self) -> Result<EventStream> {
            Ok(Box::pin(stream::once(async {
                Err(MeiliBridgeError::Source("Test error".to_string()))
            })))
        }
        
        async fn get_full_data(&self, _table: &str, _batch_size: usize) -> Result<DataStream> {
            Ok(Box::pin(stream::empty()))
        }
        
        async fn get_current_position(&self) -> Result<Position> {
            Ok(Position::PostgreSQL { lsn: "0/0".to_string() })
        }
        
        async fn acknowledge(&mut self, _position: Position) -> Result<()> {
            Ok(())
        }
        
        fn is_connected(&self) -> bool {
            true
        }
        
        async fn disconnect(&mut self) -> Result<()> {
            Ok(())
        }
        
        fn clone_box(&self) -> Box<dyn SourceAdapter> {
            Box::new(ErrorAdapter)
        }
    }
    
    let adapter = Box::new(ErrorAdapter);
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    
    let mut coordinator = CdcCoordinator::new(adapter, shutdown_rx);
    
    // Start coordinator in background
    let coordinator_handle = tokio::spawn(async move {
        // Should handle error gracefully
        let result = coordinator.run().await;
        assert!(result.is_ok());
    });
    
    // Let it process the error
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    
    // Shutdown
    shutdown_tx.send(true).unwrap();
    let _ = tokio::time::timeout(std::time::Duration::from_millis(100), coordinator_handle).await;
}

#[tokio::test]
async fn test_non_cdc_event_ignored() {
    // Create a non-CDC event
    let event = Event::FullSync {
        table: "users".to_string(),
        data: json!({"id": 1}),
    };
    
    let adapter = Box::new(MockSourceAdapter::new(vec![event]));
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    
    let mut coordinator = CdcCoordinator::new(adapter, shutdown_rx);
    let (tx, mut rx) = mpsc::channel(10);
    
    // Register task
    coordinator.register_task("users".to_string(), tx);
    
    // Start coordinator in background
    let coordinator_handle = tokio::spawn(async move {
        let _ = coordinator.run().await;
    });
    
    // Should not receive any events (non-CDC events are ignored)
    let result = tokio::time::timeout(std::time::Duration::from_millis(50), rx.recv()).await;
    assert!(result.is_err(), "Should not receive non-CDC event");
    
    // Shutdown
    shutdown_tx.send(true).unwrap();
    let _ = tokio::time::timeout(std::time::Duration::from_millis(100), coordinator_handle).await;
}