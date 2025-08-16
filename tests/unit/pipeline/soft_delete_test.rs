// Soft delete handler tests

use chrono::Utc;
use meilibridge::models::event::{
    Event, EventData, EventId, EventMetadata, EventSource, EventType,
};
use serde_json::json;
use std::collections::HashMap;

#[cfg(test)]
mod soft_delete_tests {
    use super::*;

    fn create_test_event(
        event_type: EventType,
        old: Option<HashMap<String, serde_json::Value>>,
        new: Option<HashMap<String, serde_json::Value>>,
    ) -> Event {
        Event {
            id: EventId::new(),
            event_type,
            source: EventSource {
                database: "testdb".to_string(),
                schema: "public".to_string(),
                table: "users".to_string(),
            },
            data: EventData {
                key: json!({"id": 1}),
                old,
                new,
            },
            metadata: EventMetadata {
                transaction_id: Some("123".to_string()),
                position: "0/1234567".to_string(),
                custom: HashMap::new(),
            },
            timestamp: Utc::now(),
        }
    }

    #[test]
    fn test_soft_delete_detection_timestamp() {
        // Test UPDATE that sets deleted_at
        let mut old_data = HashMap::new();
        old_data.insert("id".to_string(), json!(1));
        old_data.insert("name".to_string(), json!("John Doe"));
        old_data.insert("deleted_at".to_string(), json!(null));

        let mut new_data = HashMap::new();
        new_data.insert("id".to_string(), json!(1));
        new_data.insert("name".to_string(), json!("John Doe"));
        new_data.insert("deleted_at".to_string(), json!("2024-01-15T10:30:00Z"));

        let event = create_test_event(EventType::Update, Some(old_data), Some(new_data));

        // Check that deleted_at changed from null to a timestamp
        let old_deleted = event.data.old.as_ref().and_then(|d| d.get("deleted_at"));
        let new_deleted = event.data.new.as_ref().and_then(|d| d.get("deleted_at"));

        assert_eq!(old_deleted, Some(&json!(null)));
        assert!(new_deleted.is_some() && new_deleted != Some(&json!(null)));
    }

    #[test]
    fn test_soft_delete_detection_boolean() {
        // Test UPDATE that sets is_deleted to true
        let mut old_data = HashMap::new();
        old_data.insert("id".to_string(), json!(1));
        old_data.insert("is_deleted".to_string(), json!(false));

        let mut new_data = HashMap::new();
        new_data.insert("id".to_string(), json!(1));
        new_data.insert("is_deleted".to_string(), json!(true));

        let event = create_test_event(EventType::Update, Some(old_data), Some(new_data));

        let old_deleted = event
            .data
            .old
            .as_ref()
            .and_then(|d| d.get("is_deleted"))
            .and_then(|v| v.as_bool());
        let new_deleted = event
            .data
            .new
            .as_ref()
            .and_then(|d| d.get("is_deleted"))
            .and_then(|v| v.as_bool());

        assert_eq!(old_deleted, Some(false));
        assert_eq!(new_deleted, Some(true));
    }

    #[test]
    fn test_soft_delete_status_field() {
        // Test UPDATE that sets status to "deleted"
        let mut old_data = HashMap::new();
        old_data.insert("id".to_string(), json!(1));
        old_data.insert("status".to_string(), json!("active"));

        let mut new_data = HashMap::new();
        new_data.insert("id".to_string(), json!(1));
        new_data.insert("status".to_string(), json!("deleted"));

        let event = create_test_event(EventType::Update, Some(old_data), Some(new_data));

        let old_status = event
            .data
            .old
            .as_ref()
            .and_then(|d| d.get("status"))
            .and_then(|v| v.as_str());
        let new_status = event
            .data
            .new
            .as_ref()
            .and_then(|d| d.get("status"))
            .and_then(|v| v.as_str());

        assert_eq!(old_status, Some("active"));
        assert_eq!(new_status, Some("deleted"));
    }

    #[test]
    fn test_non_soft_delete_update() {
        // Regular UPDATE that doesn't touch soft delete fields
        let mut old_data = HashMap::new();
        old_data.insert("id".to_string(), json!(1));
        old_data.insert("email".to_string(), json!("old@example.com"));
        old_data.insert("deleted_at".to_string(), json!(null));

        let mut new_data = HashMap::new();
        new_data.insert("id".to_string(), json!(1));
        new_data.insert("email".to_string(), json!("new@example.com"));
        new_data.insert("deleted_at".to_string(), json!(null));

        let event = create_test_event(EventType::Update, Some(old_data), Some(new_data));

        // deleted_at remains null in both old and new
        let old_deleted = event.data.old.as_ref().and_then(|d| d.get("deleted_at"));
        let new_deleted = event.data.new.as_ref().and_then(|d| d.get("deleted_at"));

        assert_eq!(old_deleted, Some(&json!(null)));
        assert_eq!(new_deleted, Some(&json!(null)));
    }

    #[test]
    fn test_hard_delete_event() {
        // Real DELETE event
        let mut old_data = HashMap::new();
        old_data.insert("id".to_string(), json!(1));
        old_data.insert("name".to_string(), json!("John Doe"));

        let event = create_test_event(EventType::Delete, Some(old_data), None);

        assert_eq!(event.event_type, EventType::Delete);
        assert!(event.data.old.is_some());
        assert!(event.data.new.is_none());
    }

    #[test]
    fn test_soft_delete_undelete() {
        // Test UPDATE that clears deleted_at (undelete)
        let mut old_data = HashMap::new();
        old_data.insert("id".to_string(), json!(1));
        old_data.insert("deleted_at".to_string(), json!("2024-01-15T10:30:00Z"));

        let mut new_data = HashMap::new();
        new_data.insert("id".to_string(), json!(1));
        new_data.insert("deleted_at".to_string(), json!(null));

        let event = create_test_event(EventType::Update, Some(old_data), Some(new_data));

        // This is an undelete operation
        let old_deleted = event.data.old.as_ref().and_then(|d| d.get("deleted_at"));
        let new_deleted = event.data.new.as_ref().and_then(|d| d.get("deleted_at"));

        assert!(old_deleted.is_some() && old_deleted != Some(&json!(null)));
        assert_eq!(new_deleted, Some(&json!(null)));
    }

    #[test]
    fn test_insert_with_soft_delete_field() {
        // INSERT event with soft delete field
        let mut new_data = HashMap::new();
        new_data.insert("id".to_string(), json!(1));
        new_data.insert("name".to_string(), json!("New User"));
        new_data.insert("deleted_at".to_string(), json!(null));

        let event = create_test_event(EventType::Create, None, Some(new_data));

        assert_eq!(event.event_type, EventType::Create);
        assert!(event.data.new.is_some());

        let deleted_at = event.data.new.as_ref().and_then(|d| d.get("deleted_at"));
        assert_eq!(deleted_at, Some(&json!(null)));
    }

    #[test]
    fn test_multiple_soft_delete_fields() {
        // Test event with multiple potential soft delete indicators
        let mut old_data = HashMap::new();
        old_data.insert("id".to_string(), json!(1));
        old_data.insert("is_deleted".to_string(), json!(false));
        old_data.insert("deleted_at".to_string(), json!(null));
        old_data.insert("status".to_string(), json!("active"));

        let mut new_data = HashMap::new();
        new_data.insert("id".to_string(), json!(1));
        new_data.insert("is_deleted".to_string(), json!(true));
        new_data.insert("deleted_at".to_string(), json!("2024-01-15T10:30:00Z"));
        new_data.insert("status".to_string(), json!("deleted"));

        let event = create_test_event(EventType::Update, Some(old_data), Some(new_data));

        // All soft delete indicators changed
        assert_eq!(
            event.data.old.as_ref().and_then(|d| d.get("is_deleted")),
            Some(&json!(false))
        );
        assert_eq!(
            event.data.new.as_ref().and_then(|d| d.get("is_deleted")),
            Some(&json!(true))
        );
    }
}
