use crate::health::HealthRegistry;
use crate::pipeline::PipelineOrchestrator;
use crate::sync::SyncTaskManager;
use crate::source::postgres::StatementCache;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Shared state for the API server
#[derive(Clone)]
pub struct ApiState {
    pub orchestrator: Arc<RwLock<PipelineOrchestrator>>,
    pub task_manager: Arc<RwLock<SyncTaskManager>>,
    pub health_registry: Option<Arc<HealthRegistry>>,
    pub postgres_cache: Option<Arc<StatementCache>>,
}

impl ApiState {
    pub fn new(
        orchestrator: Arc<RwLock<PipelineOrchestrator>>,
        task_manager: Arc<RwLock<SyncTaskManager>>,
    ) -> Self {
        Self {
            orchestrator,
            task_manager,
            health_registry: None,
            postgres_cache: None,
        }
    }
    
    pub fn with_health_registry(mut self, registry: Arc<HealthRegistry>) -> Self {
        self.health_registry = Some(registry);
        self
    }
    
    pub fn with_postgres_cache(mut self, cache: Arc<StatementCache>) -> Self {
        self.postgres_cache = Some(cache);
        self
    }
    
    pub fn health_registry(&self) -> Option<&Arc<HealthRegistry>> {
        self.health_registry.as_ref()
    }
}