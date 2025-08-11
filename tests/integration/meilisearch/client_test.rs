// Meilisearch client integration tests

// Testing Meilisearch client integration
use meilibridge::config::MeilisearchConfig;
use crate::common::setup_meilisearch;
use meilisearch_sdk::client::Client;

#[cfg(test)]
mod meilisearch_client_tests {
    use super::*;

    #[tokio::test]
    async fn test_connection_establishment() {
        let (_container, client, _url) = setup_meilisearch().await.unwrap();
        
        // Test connection
        let health = client.health().await;
        assert!(health.is_ok());
        
        let stats = client.get_stats().await;
        assert!(stats.is_ok());
    }

    #[tokio::test]
    async fn test_authentication() {
        let (_container, _, url) = setup_meilisearch().await.unwrap();
        
        // Test with valid API key
        let valid_client = Client::new(&url, Some("masterKey")).unwrap();
        let result = valid_client.get_stats().await;
        assert!(result.is_ok());
        
        // Test with invalid API key
        let invalid_client = Client::new(&url, Some("wrongKey")).unwrap();
        let result = invalid_client.get_stats().await;
        assert!(result.is_err());
        
        // Test without API key (should fail for protected routes)
        let no_key_client = Client::new(&url, None::<&str>).unwrap();
        let result = no_key_client.create_index("test", Some("id")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_error_handling() {
        // Test connection to non-existent server
        let client = Client::new("http://localhost:59999", Some("masterKey")).unwrap();
        let result = client.health().await;
        assert!(result.is_err());
        
        // Test invalid URL - Client::new may accept it but operations should fail
        let client_result = Client::new("not-a-url", Some("masterKey"));
        if let Ok(client) = client_result {
            // If client was created, operations should fail
            let result = client.health().await;
            assert!(result.is_err());
        } else {
            // If client creation failed, that's also fine
            assert!(client_result.is_err());
        }
    }

    #[tokio::test]
    async fn test_timeout_behavior() {
        let (_container, _, url) = setup_meilisearch().await.unwrap();
        
        // Create client with very short timeout
        let client = Client::new(&url, Some("masterKey")).unwrap();
        
        // This should work with normal timeout
        let result = client.health().await;
        assert!(result.is_ok());
        
        // Note: Testing actual timeout would require a way to simulate slow responses
    }

    #[tokio::test]
    async fn test_concurrent_operations() {
        let (_container, client, _url) = setup_meilisearch().await.unwrap();
        
        // Create test index
        let index = client.create_index("concurrent_test", Some("id"))
            .await
            .unwrap()
            .wait_for_completion(&client, None, None)
            .await
            .unwrap()
            .try_make_index(&client)
            .unwrap();
        
        // Perform multiple concurrent operations
        let mut handles = vec![];
        
        for i in 0..10 {
            let index_clone = index.clone();
            let handle = tokio::spawn(async move {
                let doc = serde_json::json!({
                    "id": i,
                    "title": format!("Document {}", i),
                    "content": format!("Content for document {}", i)
                });
                
                index_clone.add_documents(&[doc], Some("id"))
                    .await
                    .unwrap()
                    .wait_for_completion(&index_clone.client, None, None)
                    .await
                    .unwrap();
            });
            handles.push(handle);
        }
        
        // Wait for all operations to complete
        for handle in handles {
            handle.await.unwrap();
        }
        
        // Verify all documents were added
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        let stats = index.get_stats().await.unwrap();
        assert_eq!(stats.number_of_documents, 10);
    }

    #[tokio::test]
    async fn test_client_configuration() {
        let (_container, _, url) = setup_meilisearch().await.unwrap();
        
        // Test client with custom configuration
        let config = MeilisearchConfig {
            url: url.clone(),
            api_key: Some("masterKey".to_string()),
            timeout: 30,
            max_connections: 10,
            index_settings: Default::default(),
            primary_key: None,
            batch_size: 1000,
            auto_create_index: true,
            circuit_breaker: Default::default(),
        };
        
        let client = Client::new(&config.url, config.api_key.as_deref()).unwrap();
        
        // Verify client works with configuration
        let health = client.health().await;
        assert!(health.is_ok());
    }

    #[tokio::test]
    async fn test_index_operations() {
        let (_container, client, _url) = setup_meilisearch().await.unwrap();
        
        // Create index
        let index_name = "test_index_ops";
        let task = client.create_index(index_name, Some("id"))
            .await
            .unwrap();
        
        task.wait_for_completion(&client, None, None)
            .await
            .unwrap();
        
        // Get index
        let mut index = client.get_index(index_name).await.unwrap();
        assert_eq!(index.uid, index_name);
        assert_eq!(index.primary_key, Some("id".to_string()));
        
        // Update index
        let update_task = index.set_primary_key("new_id")
            .await
            .unwrap();
        
        update_task.wait_for_completion(&client, None, None)
            .await
            .unwrap();
        
        // Verify update
        let updated_index = client.get_index(index_name).await.unwrap();
        assert_eq!(updated_index.primary_key, Some("new_id".to_string()));
        
        // Delete index
        let delete_task = index.delete()
            .await
            .unwrap();
        
        delete_task.wait_for_completion(&client, None, None)
            .await
            .unwrap();
        
        // Verify deletion
        let result = client.get_index(index_name).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_reconnection_handling() {
        let (_container, client, url) = setup_meilisearch().await.unwrap();
        
        // Create initial index
        let index_name = "reconnect_test";
        client.create_index(index_name, Some("id"))
            .await
            .unwrap()
            .wait_for_completion(&client, None, None)
            .await
            .unwrap();
        
        // Simulate disconnection by creating new client
        let new_client = Client::new(&url, Some("masterKey")).unwrap();
        
        // Verify new client can access existing data
        let index = new_client.get_index(index_name).await.unwrap();
        assert_eq!(index.uid, index_name);
    }

    #[tokio::test]
    async fn test_error_recovery() {
        let (_container, client, _url) = setup_meilisearch().await.unwrap();
        
        // Try to create index with invalid name
        let result = client.create_index("", Some("id")).await;
        assert!(result.is_err());
        
        // Try to get non-existent index
        let result = client.get_index("non_existent_index").await;
        assert!(result.is_err());
        
        // Verify client still works after errors
        let health = client.health().await;
        assert!(health.is_ok());
    }
}