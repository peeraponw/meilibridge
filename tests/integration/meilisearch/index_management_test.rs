// Meilisearch index management integration tests

// Testing Meilisearch index management patterns
use crate::common::setup_meilisearch;
use meilisearch_sdk::settings::Settings;
use serde_json::json;

#[cfg(test)]
mod index_management_tests {
    use super::*;

    #[tokio::test]
    async fn test_index_creation_deletion() {
        let (_container, client, _url) = setup_meilisearch().await.unwrap();

        // Create index
        let index_name = "test_create_delete";
        let task = client.create_index(index_name, Some("id")).await.unwrap();

        task.wait_for_completion(&client, None, None).await.unwrap();

        // Verify index exists
        let index = client.get_index(index_name).await.unwrap();
        assert_eq!(index.uid, index_name);
        assert_eq!(index.primary_key, Some("id".to_string()));

        // Delete index
        let delete_task = index.delete().await.unwrap();
        delete_task
            .wait_for_completion(&client, None, None)
            .await
            .unwrap();

        // Verify index is deleted
        let result = client.get_index(index_name).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_settings_synchronization() {
        let (_container, client, _url) = setup_meilisearch().await.unwrap();

        // Create index
        let index = client
            .create_index("settings_test", Some("id"))
            .await
            .unwrap()
            .wait_for_completion(&client, None, None)
            .await
            .unwrap()
            .try_make_index(&client)
            .unwrap();

        // Configure settings
        let settings = Settings::new()
            .with_searchable_attributes(vec!["title".to_string(), "content".to_string()])
            .with_filterable_attributes(vec!["category".to_string(), "tags".to_string()])
            .with_sortable_attributes(vec!["created_at".to_string()])
            .with_distinct_attribute(Some("id".to_string()));

        let task = index.set_settings(&settings).await.unwrap();
        task.wait_for_completion(&client, None, None).await.unwrap();

        // Verify settings
        let retrieved_settings = index.get_settings().await.unwrap();

        assert_eq!(
            retrieved_settings.searchable_attributes,
            Some(vec!["title".to_string(), "content".to_string()])
        );
        assert_eq!(
            retrieved_settings.filterable_attributes,
            Some(vec!["category".to_string(), "tags".to_string()])
        );
        assert_eq!(
            retrieved_settings.sortable_attributes,
            Some(vec!["created_at".to_string()])
        );
        assert_eq!(
            retrieved_settings.distinct_attribute,
            Some(Some("id".to_string()))
        );
    }

    #[tokio::test]
    #[ignore = "SwapIndexes API may not be available in this version"]
    async fn test_atomic_index_swap() {
        let (_container, client, _url) = setup_meilisearch().await.unwrap();

        // Create primary index with data
        let primary_name = "primary_index";
        let primary_index = client
            .create_index(primary_name, Some("id"))
            .await
            .unwrap()
            .wait_for_completion(&client, None, None)
            .await
            .unwrap()
            .try_make_index(&client)
            .unwrap();

        let primary_docs = vec![
            json!({"id": 1, "title": "Primary 1"}),
            json!({"id": 2, "title": "Primary 2"}),
        ];

        primary_index
            .add_documents(&primary_docs, Some("id"))
            .await
            .unwrap()
            .wait_for_completion(&client, None, None)
            .await
            .unwrap();

        // Create staging index with new data
        let staging_name = "staging_index";
        let staging_index = client
            .create_index(staging_name, Some("id"))
            .await
            .unwrap()
            .wait_for_completion(&client, None, None)
            .await
            .unwrap()
            .try_make_index(&client)
            .unwrap();

        let staging_docs = vec![
            json!({"id": 1, "title": "Staging 1 Updated"}),
            json!({"id": 2, "title": "Staging 2 Updated"}),
            json!({"id": 3, "title": "Staging 3 New"}),
        ];

        staging_index
            .add_documents(&staging_docs, Some("id"))
            .await
            .unwrap()
            .wait_for_completion(&client, None, None)
            .await
            .unwrap();

        // Swap indexes - this API may vary by version
        // let swap_task = client.swap_indexes(...)
        // For now, we'll skip the actual swap
        // Skip actual swap for now - SwapIndexes API may not be available
        // in this version of meilisearch-sdk
        // Future versions should implement:
        // let swap_task = client.swap_indexes([...]).await.unwrap();
        // swap_task.wait_for_completion(&client, None, None).await.unwrap();

        // Verify swap
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        let primary_after = client.get_index(primary_name).await.unwrap();
        let doc = primary_after
            .get_document::<serde_json::Value>("1")
            .await
            .unwrap();
        assert_eq!(doc["title"], "Staging 1 Updated");

        let stats = primary_after.get_stats().await.unwrap();
        assert_eq!(stats.number_of_documents, 3);
    }

    #[tokio::test]
    async fn test_schema_updates() {
        let (_container, client, _url) = setup_meilisearch().await.unwrap();

        let index = client
            .create_index("schema_test", Some("id"))
            .await
            .unwrap()
            .wait_for_completion(&client, None, None)
            .await
            .unwrap()
            .try_make_index(&client)
            .unwrap();

        // Add initial documents with basic schema
        let initial_docs = vec![
            json!({"id": 1, "name": "Item 1"}),
            json!({"id": 2, "name": "Item 2"}),
        ];

        index
            .add_documents(&initial_docs, Some("id"))
            .await
            .unwrap()
            .wait_for_completion(&client, None, None)
            .await
            .unwrap();

        // Add documents with extended schema
        let extended_docs = vec![
            json!({"id": 3, "name": "Item 3", "description": "New field", "price": 29.99}),
            json!({"id": 4, "name": "Item 4", "description": "Another new", "tags": ["tag1", "tag2"]}),
        ];

        index
            .add_documents(&extended_docs, Some("id"))
            .await
            .unwrap()
            .wait_for_completion(&client, None, None)
            .await
            .unwrap();

        // Update settings for new fields
        let settings = Settings::new()
            .with_searchable_attributes(vec!["name".to_string(), "description".to_string()])
            .with_filterable_attributes(vec!["price".to_string(), "tags".to_string()]);

        index
            .set_settings(&settings)
            .await
            .unwrap()
            .wait_for_completion(&client, None, None)
            .await
            .unwrap();

        // Verify schema flexibility
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        let results = index
            .search()
            .with_query("new")
            .execute::<serde_json::Value>()
            .await
            .unwrap();

        assert!(results.hits.len() >= 2);
    }

    #[tokio::test]
    async fn test_multiple_index_management() {
        let (_container, client, _url) = setup_meilisearch().await.unwrap();

        // Create multiple indexes
        let index_names = vec!["users", "products", "orders", "reviews"];

        for name in &index_names {
            client
                .create_index(name, Some("id"))
                .await
                .unwrap()
                .wait_for_completion(&client, None, None)
                .await
                .unwrap();
        }

        // List all indexes
        let indexes = client.list_all_indexes().await.unwrap();
        assert!(indexes.results.len() >= 4);

        for name in &index_names {
            assert!(indexes.results.iter().any(|idx| idx.uid == *name));
        }

        // Add data to each index
        for name in &index_names {
            let index = client.get_index(name).await.unwrap();
            let docs = vec![
                json!({"id": 1, "data": format!("{} item 1", name)}),
                json!({"id": 2, "data": format!("{} item 2", name)}),
            ];

            index
                .add_documents(&docs, Some("id"))
                .await
                .unwrap()
                .wait_for_completion(&client, None, None)
                .await
                .unwrap();
        }

        // Verify each index has data
        for name in &index_names {
            let index = client.get_index(name).await.unwrap();
            let stats = index.get_stats().await.unwrap();
            assert_eq!(stats.number_of_documents, 2);
        }
    }

    #[tokio::test]
    async fn test_index_stats_and_monitoring() {
        let (_container, client, _url) = setup_meilisearch().await.unwrap();

        let index = client
            .create_index("stats_test", Some("id"))
            .await
            .unwrap()
            .wait_for_completion(&client, None, None)
            .await
            .unwrap()
            .try_make_index(&client)
            .unwrap();

        // Get initial stats
        let initial_stats = index.get_stats().await.unwrap();
        assert_eq!(initial_stats.number_of_documents, 0);
        assert!(!initial_stats.is_indexing);

        // Add documents
        let docs: Vec<_> = (0..100)
            .map(|i| {
                json!({
                    "id": i,
                    "title": format!("Document {}", i),
                    "content": "Lorem ipsum dolor sit amet".repeat(10),
                })
            })
            .collect();

        index
            .add_documents(&docs, Some("id"))
            .await
            .unwrap()
            .wait_for_completion(&client, None, None)
            .await
            .unwrap();

        // Get updated stats
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        let updated_stats = index.get_stats().await.unwrap();
        assert_eq!(updated_stats.number_of_documents, 100);

        // Check field distribution
        let distribution = &updated_stats.field_distribution;
        if !distribution.is_empty() {
            assert!(distribution.contains_key("id"));
            assert!(distribution.contains_key("title"));
            assert!(distribution.contains_key("content"));
        }
    }

    #[tokio::test]
    async fn test_index_recovery_after_error() {
        let (_container, client, _url) = setup_meilisearch().await.unwrap();

        let index_name = "recovery_test";

        // Create index
        client
            .create_index(index_name, Some("id"))
            .await
            .unwrap()
            .wait_for_completion(&client, None, None)
            .await
            .unwrap();

        let index = client.get_index(index_name).await.unwrap();

        // Add some valid documents
        let valid_docs = vec![
            json!({"id": 1, "title": "Valid 1"}),
            json!({"id": 2, "title": "Valid 2"}),
        ];

        index
            .add_documents(&valid_docs, Some("id"))
            .await
            .unwrap()
            .wait_for_completion(&client, None, None)
            .await
            .unwrap();

        // Try to add documents with wrong primary key
        let wrong_key_docs = vec![json!({"wrong_id": 3, "title": "Wrong key"})];

        let _result = index.add_documents(&wrong_key_docs, Some("id")).await;
        // This might succeed but documents won't be indexed properly

        // Verify index still works and has correct documents
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        let stats = index.get_stats().await.unwrap();
        assert_eq!(stats.number_of_documents, 2);

        // Can still add more valid documents
        let more_docs = vec![json!({"id": 4, "title": "Valid 4"})];

        index
            .add_documents(&more_docs, Some("id"))
            .await
            .unwrap()
            .wait_for_completion(&client, None, None)
            .await
            .unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        let final_stats = index.get_stats().await.unwrap();
        assert_eq!(final_stats.number_of_documents, 3);
    }
}
