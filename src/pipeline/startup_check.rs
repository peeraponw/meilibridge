use crate::config::{Config, SyncTaskConfig};
use crate::error::{MeiliBridgeError, Result};
use tracing::{info, warn};

/// Performs startup checks for the pipeline
pub struct StartupChecker {
    config: Config,
}

impl StartupChecker {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Perform all startup checks
    pub async fn check(&self) -> Result<()> {
        info!("Performing startup checks...");

        // Check CDC-only tables
        self.check_cdc_only_tables().await?;

        // Check Meilisearch connectivity
        self.check_meilisearch_connectivity().await?;

        info!("✅ Startup checks completed successfully");
        Ok(())
    }

    /// Check CDC-only tables and ensure proper configuration
    async fn check_cdc_only_tables(&self) -> Result<()> {
        let mut cdc_only_tables = Vec::new();

        for task in &self.config.sync_tasks {
            if task.is_cdc_only() {
                cdc_only_tables.push(task);
            }
        }

        if cdc_only_tables.is_empty() {
            return Ok(());
        }

        info!(
            "Found {} CDC-only tables (no initial full sync):",
            cdc_only_tables.len()
        );

        for task in &cdc_only_tables {
            info!("  - Table '{}' -> Index '{}'", task.table, task.index);

            // Warn if auto_create_index is disabled
            if !self.config.meilisearch.auto_create_index {
                warn!(
                    "    ⚠️  auto_create_index is disabled. Index '{}' must exist in Meilisearch \
                    or the first CDC event will fail.",
                    task.index
                );
            } else {
                info!(
                    "    ✓ auto_create_index is enabled. Index '{}' will be created on first event.",
                    task.index
                );
            }

            // Check if primary key is configured
            if !task.has_primary_key_configured(self.config.meilisearch.primary_key.as_ref()) {
                warn!(
                    "    ⚠️  No primary key configured for table '{}'. Delete operations will fail.",
                    task.table
                );
            }
        }

        // If auto_create_index is disabled, suggest enabling it
        if !self.config.meilisearch.auto_create_index && !cdc_only_tables.is_empty() {
            warn!(
                "💡 Consider enabling 'meilisearch.auto_create_index' in your configuration \
                to automatically create indexes for CDC-only tables."
            );
        }

        Ok(())
    }

    /// Check Meilisearch connectivity and version
    async fn check_meilisearch_connectivity(&self) -> Result<()> {
        info!("Checking Meilisearch connectivity...");

        use crate::destination::meilisearch::protected_client::ProtectedMeilisearchClient;

        // Create a protected client to test connectivity
        let client = ProtectedMeilisearchClient::new(self.config.meilisearch.clone())?;

        // Test the connection
        match client.test_connection().await {
            Ok(_) => {
                info!("✓ Successfully connected to Meilisearch");

                // Try to get version info for better diagnostics
                match client.get_version().await {
                    Ok(version) => {
                        info!("  Meilisearch version: {}", version);
                    }
                    Err(e) => {
                        warn!("  Could not retrieve Meilisearch version: {}", e);
                    }
                }
            }
            Err(e) => {
                return Err(MeiliBridgeError::Meilisearch(format!(
                    "Failed to connect to Meilisearch at {}: {}",
                    self.config.meilisearch.url, e
                )));
            }
        }

        // Validate API key permissions if configured
        if self.config.meilisearch.api_key.is_some() {
            info!("  API key configured - permissions will be validated on first use");
        } else {
            warn!("  No API key configured - ensure Meilisearch allows anonymous access");
        }

        // For CDC-only tables with auto_create_index enabled,
        // we could pre-create indexes here, but it's better to let
        // them be created on-demand to avoid empty indexes

        Ok(())
    }
}

/// Extension trait for sync task configuration
trait SyncTaskExt {
    fn is_cdc_only(&self) -> bool;
    fn has_primary_key_configured(&self, global_pk: Option<&String>) -> bool;
}

impl SyncTaskExt for SyncTaskConfig {
    fn is_cdc_only(&self) -> bool {
        !self.full_sync_on_start.unwrap_or(false)
    }

    fn has_primary_key_configured(&self, global_pk: Option<&String>) -> bool {
        !self.primary_key.is_empty() || global_pk.map(|s| !s.is_empty()).unwrap_or(false)
    }
}
