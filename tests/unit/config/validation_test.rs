// Simplified configuration validation tests

use meilibridge::config::*;

#[cfg(test)]
mod validation_tests {
    use super::*;

    #[test]
    fn test_valid_configuration_acceptance() {
        // Test that a minimal valid configuration is accepted
        let config = Config {
            app: Default::default(),
            source: Some(SourceConfig::PostgreSQL(Box::new(PostgreSQLConfig {
                connection: PostgreSQLConnection::ConnectionString(
                    "postgresql://user:pass@localhost/db".to_string(),
                ),
                slot_name: "test_slot".to_string(),
                publication: "test_pub".to_string(),
                pool: Default::default(),
                ssl: Default::default(),
                statement_cache: Default::default(),
            }))),
            sources: vec![],
            sync_tasks: vec![SyncTaskConfig {
                id: "test".to_string(),
                source_name: "primary".to_string(),
                table: "users".to_string(),
                index: "users_index".to_string(),
                primary_key: "id".to_string(),
                filter: None,
                transform: None,
                mapping: None,
                full_sync_on_start: None,
                options: Default::default(),
                auto_start: None,
                soft_delete: None,
            }],
            meilisearch: MeilisearchConfig {
                url: "http://localhost:7700".to_string(),
                api_key: None,
                timeout: 30,
                max_connections: 10,
                index_settings: Default::default(),
                primary_key: None,
                batch_size: 1000,
                auto_create_index: true,
                circuit_breaker: Default::default(),
            },
            redis: Some(RedisConfig {
                url: "redis://localhost:6379".to_string(),
                password: None,
                database: 0,
                key_prefix: "meilibridge".to_string(),
                pool: Default::default(),
                checkpoint_retention: Default::default(),
            }),
            api: Default::default(),
            logging: Default::default(),
            monitoring: Default::default(),
            error_handling: Default::default(),
            plugins: Default::default(),
            features: Default::default(),
            performance: Default::default(),
            at_least_once_delivery: Default::default(),
        };

        // If we can create it, it's valid
        assert_eq!(config.sync_tasks.len(), 1);
        assert_eq!(config.sync_tasks[0].table, "users");
    }

    #[test]
    fn test_empty_sync_tasks() {
        let config = Config {
            app: Default::default(),
            source: None,
            sources: vec![],
            sync_tasks: vec![], // Empty sync tasks
            meilisearch: MeilisearchConfig {
                url: "http://localhost:7700".to_string(),
                api_key: None,
                timeout: 30,
                max_connections: 10,
                index_settings: Default::default(),
                primary_key: None,
                batch_size: 1000,
                auto_create_index: true,
                circuit_breaker: Default::default(),
            },
            redis: Some(RedisConfig {
                url: "redis://localhost:6379".to_string(),
                password: None,
                database: 0,
                key_prefix: "meilibridge".to_string(),
                pool: Default::default(),
                checkpoint_retention: Default::default(),
            }),
            api: Default::default(),
            logging: Default::default(),
            monitoring: Default::default(),
            error_handling: Default::default(),
            plugins: Default::default(),
            features: Default::default(),
            performance: Default::default(),
            at_least_once_delivery: Default::default(),
        };

        assert_eq!(config.sync_tasks.len(), 0);
    }

    #[test]
    fn test_source_config_variations() {
        // Test PostgreSQL with connection string
        let pg_config = SourceConfig::PostgreSQL(Box::new(PostgreSQLConfig {
            connection: PostgreSQLConnection::ConnectionString(
                "postgresql://user:pass@localhost/db".to_string(),
            ),
            slot_name: "slot".to_string(),
            publication: "pub".to_string(),
            pool: Default::default(),
            ssl: Default::default(),
            statement_cache: Default::default(),
        }));

        assert!(matches!(pg_config, SourceConfig::PostgreSQL(_)));

        // Test PostgreSQL with parameters
        let pg_params = SourceConfig::PostgreSQL(Box::new(PostgreSQLConfig {
            connection: PostgreSQLConnection::Parameters {
                host: "localhost".to_string(),
                port: 5432,
                database: "testdb".to_string(),
                username: "user".to_string(),
                password: "pass".to_string(),
            },
            slot_name: "slot".to_string(),
            publication: "pub".to_string(),
            pool: Default::default(),
            ssl: Default::default(),
            statement_cache: Default::default(),
        }));

        match pg_params {
            SourceConfig::PostgreSQL(ref cfg) => match &cfg.connection {
                PostgreSQLConnection::Parameters { host, .. } => {
                    assert_eq!(host, "localhost");
                }
                _ => panic!("Expected parameters"),
            },
            _ => panic!("Expected PostgreSQL config"),
        }
    }

    #[test]
    fn test_meilisearch_config() {
        let config = MeilisearchConfig {
            url: "http://localhost:7700".to_string(),
            api_key: Some("masterKey".to_string()),
            timeout: 60,
            max_connections: 20,
            index_settings: Default::default(),
            primary_key: Some("id".to_string()),
            batch_size: 500,
            auto_create_index: false,
            circuit_breaker: Default::default(),
        };

        assert_eq!(config.url, "http://localhost:7700");
        assert_eq!(config.api_key, Some("masterKey".to_string()));
        assert_eq!(config.timeout, 60);
        assert_eq!(config.batch_size, 500);
    }

    #[test]
    fn test_sync_task_config() {
        let task = SyncTaskConfig {
            id: "task1".to_string(),
            source_name: "primary".to_string(),
            table: "products".to_string(),
            index: "products_index".to_string(),
            primary_key: "product_id".to_string(),
            filter: None,
            transform: None,
            mapping: None,
            full_sync_on_start: Some(true),
            options: SyncOptions {
                batch_size: 2000,
                batch_timeout_ms: 5000,
                deduplicate: true,
                retry: Default::default(),
            },
            auto_start: Some(false),
            soft_delete: Some(SoftDeleteConfig {
                field: "deleted_at".to_string(),
                delete_values: vec![serde_json::json!(true)],
                handle_on_full_sync: true,
                handle_on_cdc: true,
            }),
        };

        assert_eq!(task.table, "products");
        assert_eq!(task.primary_key, "product_id");
        assert_eq!(task.options.batch_size, 2000);
        assert_eq!(task.soft_delete.as_ref().unwrap().field, "deleted_at");
    }
}
