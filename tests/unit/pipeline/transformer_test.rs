// Pipeline transformer tests

use chrono::Utc;
use meilibridge::models::event::{
    Event, EventData, EventId, EventMetadata, EventSource, EventType,
};
use serde_json::json;
use std::collections::HashMap;

#[cfg(test)]
mod transformer_tests {
    use super::*;

    fn create_test_event() -> Event {
        let mut data = HashMap::new();
        data.insert("id".to_string(), json!(1));
        data.insert("name".to_string(), json!("Test User"));
        data.insert("email".to_string(), json!("test@example.com"));

        Event {
            id: EventId::new(),
            event_type: EventType::Create,
            source: EventSource {
                database: "testdb".to_string(),
                schema: "public".to_string(),
                table: "users".to_string(),
            },
            data: EventData {
                key: json!({"id": 1}),
                old: None,
                new: Some(data),
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
    fn test_event_creation() {
        let event = create_test_event();
        assert_eq!(event.event_type, EventType::Create);
        assert_eq!(event.source.table, "users");
        assert!(event.data.new.is_some());
        assert!(event.data.old.is_none());
    }

    #[test]
    fn test_update_event() {
        let mut old_data = HashMap::new();
        old_data.insert("id".to_string(), json!(1));
        old_data.insert("name".to_string(), json!("Old Name"));

        let mut new_data = HashMap::new();
        new_data.insert("id".to_string(), json!(1));
        new_data.insert("name".to_string(), json!("New Name"));

        let event = Event {
            id: EventId::new(),
            event_type: EventType::Update,
            source: EventSource {
                database: "testdb".to_string(),
                schema: "public".to_string(),
                table: "users".to_string(),
            },
            data: EventData {
                key: json!({"id": 1}),
                old: Some(old_data),
                new: Some(new_data),
            },
            metadata: EventMetadata {
                transaction_id: Some("124".to_string()),
                position: "0/1234568".to_string(),
                custom: HashMap::new(),
            },
            timestamp: Utc::now(),
        };

        assert_eq!(event.event_type, EventType::Update);
        assert!(event.data.old.is_some());
        assert!(event.data.new.is_some());
    }

    #[test]
    fn test_delete_event() {
        let mut old_data = HashMap::new();
        old_data.insert("id".to_string(), json!(1));
        old_data.insert("name".to_string(), json!("Deleted User"));

        let event = Event {
            id: EventId::new(),
            event_type: EventType::Delete,
            source: EventSource {
                database: "testdb".to_string(),
                schema: "public".to_string(),
                table: "users".to_string(),
            },
            data: EventData {
                key: json!({"id": 1}),
                old: Some(old_data),
                new: None,
            },
            metadata: EventMetadata {
                transaction_id: Some("125".to_string()),
                position: "0/1234569".to_string(),
                custom: HashMap::new(),
            },
            timestamp: Utc::now(),
        };

        assert_eq!(event.event_type, EventType::Delete);
        assert!(event.data.old.is_some());
        assert!(event.data.new.is_none());
    }

    #[test]
    fn test_event_with_nested_data() {
        let mut data = HashMap::new();
        data.insert("id".to_string(), json!(1));
        data.insert(
            "profile".to_string(),
            json!({
                "name": "Test User",
                "age": 30,
                "settings": {
                    "theme": "dark",
                    "notifications": true
                }
            }),
        );

        let event = Event {
            id: EventId::new(),
            event_type: EventType::Create,
            source: EventSource {
                database: "testdb".to_string(),
                schema: "public".to_string(),
                table: "users".to_string(),
            },
            data: EventData {
                key: json!({"id": 1}),
                old: None,
                new: Some(data.clone()),
            },
            metadata: EventMetadata {
                transaction_id: None,
                position: "0/1234570".to_string(),
                custom: HashMap::new(),
            },
            timestamp: Utc::now(),
        };

        let profile = event.data.new.as_ref().unwrap().get("profile").unwrap();
        assert!(profile.is_object());
        assert_eq!(profile["name"], json!("Test User"));
    }

    #[test]
    fn test_event_metadata() {
        let mut custom_metadata = HashMap::new();
        custom_metadata.insert("source_version".to_string(), json!("14.5"));
        custom_metadata.insert("replication_slot".to_string(), json!("test_slot"));

        let event = Event {
            id: EventId::new(),
            event_type: EventType::Create,
            source: EventSource {
                database: "testdb".to_string(),
                schema: "public".to_string(),
                table: "users".to_string(),
            },
            data: EventData {
                key: json!({"id": 1}),
                old: None,
                new: Some(HashMap::new()),
            },
            metadata: EventMetadata {
                transaction_id: Some("126".to_string()),
                position: "0/1234571".to_string(),
                custom: custom_metadata,
            },
            timestamp: Utc::now(),
        };

        assert_eq!(event.metadata.transaction_id, Some("126".to_string()));
        assert_eq!(
            event.metadata.custom.get("source_version"),
            Some(&json!("14.5"))
        );
    }
}
