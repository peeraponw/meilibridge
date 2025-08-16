// pgoutput protocol parser tests

use bytes::{BufMut, BytesMut};
use meilibridge::source::postgres::pgoutput::*;

#[cfg(test)]
mod pgoutput_tests {
    use super::*;

    fn create_test_parser() -> PgOutputParser {
        PgOutputParser::new()
    }

    #[test]
    fn test_begin_message_parsing() {
        let mut parser = create_test_parser();

        // Create a BEGIN message
        let mut msg = BytesMut::new();
        msg.extend_from_slice(b"B"); // Message type
        msg.extend_from_slice(&1234567890u64.to_be_bytes()); // Final LSN
        msg.extend_from_slice(&1640995200000000i64.to_be_bytes()); // Timestamp (2022-01-01)
        msg.extend_from_slice(&42u32.to_be_bytes()); // XID

        let result = parser.parse_message(&msg);
        assert!(result.is_ok());

        match result.unwrap() {
            Some(PgOutputMessage::Begin(begin)) => {
                assert_eq!(begin.final_lsn, 1234567890);
                assert_eq!(begin.xid, 42);
                assert!(begin.timestamp > 0);
            }
            _ => panic!("Expected BEGIN message"),
        }
    }

    #[test]
    fn test_commit_message_parsing() {
        let mut parser = create_test_parser();

        // Create a COMMIT message
        let mut msg = BytesMut::new();
        msg.extend_from_slice(b"C"); // Message type
        msg.put_u8(0); // Flags
        msg.extend_from_slice(&1234567890u64.to_be_bytes()); // Commit LSN
        msg.extend_from_slice(&1234567890u64.to_be_bytes()); // End LSN
        msg.extend_from_slice(&1640995200000000i64.to_be_bytes()); // Timestamp

        let result = parser.parse_message(&msg);
        assert!(result.is_ok());

        match result.unwrap() {
            Some(PgOutputMessage::Commit(commit)) => {
                assert_eq!(commit.commit_lsn, 1234567890);
                assert_eq!(commit.end_lsn, 1234567890);
                assert!(commit.timestamp > 0);
            }
            _ => panic!("Expected COMMIT message"),
        }
    }

    #[test]
    fn test_relation_message_parsing() {
        let mut parser = create_test_parser();

        // Create a RELATION message
        let mut msg = BytesMut::new();
        msg.extend_from_slice(b"R"); // Message type
        msg.extend_from_slice(&123u32.to_be_bytes()); // Relation ID
        msg.extend_from_slice(b"public\0"); // Namespace
        msg.extend_from_slice(b"users\0"); // Relation name
        msg.put_u8(b'd'); // Replica identity
        msg.extend_from_slice(&3u16.to_be_bytes()); // Number of columns

        // Column 1: id (integer)
        msg.put_u8(1); // Flags (part of key)
        msg.extend_from_slice(b"id\0"); // Column name
        msg.extend_from_slice(&23u32.to_be_bytes()); // Type OID (int4)
        msg.extend_from_slice(&(-1i32).to_be_bytes()); // Type modifier

        // Column 2: name (text)
        msg.put_u8(0); // Flags
        msg.extend_from_slice(b"name\0"); // Column name
        msg.extend_from_slice(&25u32.to_be_bytes()); // Type OID (text)
        msg.extend_from_slice(&(-1i32).to_be_bytes()); // Type modifier

        // Column 3: active (boolean)
        msg.put_u8(0); // Flags
        msg.extend_from_slice(b"active\0"); // Column name
        msg.extend_from_slice(&16u32.to_be_bytes()); // Type OID (bool)
        msg.extend_from_slice(&(-1i32).to_be_bytes()); // Type modifier

        let result = parser.parse_message(&msg);
        assert!(result.is_ok());

        match result.unwrap() {
            Some(PgOutputMessage::Relation(relation)) => {
                assert_eq!(relation.relation_id, 123);
                assert_eq!(relation.namespace, "public");
                assert_eq!(relation.relation_name, "users");
                assert_eq!(relation.columns.len(), 3);
                assert_eq!(relation.columns[0].name, "id");
                assert_eq!(relation.columns[1].name, "name");
                assert_eq!(relation.columns[2].name, "active");
                assert_eq!(relation.columns[0].flags, 1); // Key column
                assert_eq!(relation.columns[1].flags, 0);
            }
            _ => panic!("Expected RELATION message"),
        }
    }

    #[test]
    fn test_insert_message_parsing() {
        let mut parser = create_test_parser();

        // First, register a relation
        register_test_relation(&mut parser);

        // Create an INSERT message
        let mut msg = BytesMut::new();
        msg.extend_from_slice(b"I"); // Message type
        msg.extend_from_slice(&123u32.to_be_bytes()); // Relation ID
        msg.put_u8(b'N'); // New tuple follows

        // Tuple data
        msg.extend_from_slice(&3u16.to_be_bytes()); // Number of columns

        // Column 1: id = 1
        msg.put_u8(b't'); // Text format
        msg.extend_from_slice(&1u32.to_be_bytes()); // Length
        msg.extend_from_slice(b"1"); // Value

        // Column 2: name = "Alice"
        msg.put_u8(b't'); // Text format
        msg.extend_from_slice(&5u32.to_be_bytes()); // Length
        msg.extend_from_slice(b"Alice"); // Value

        // Column 3: active = true
        msg.put_u8(b't'); // Text format
        msg.extend_from_slice(&1u32.to_be_bytes()); // Length
        msg.extend_from_slice(b"t"); // Value

        let result = parser.parse_message(&msg);
        assert!(result.is_ok());

        match result.unwrap() {
            Some(PgOutputMessage::Insert(insert)) => {
                assert_eq!(insert.relation_id, 123);
                assert_eq!(insert.tuple.columns.len(), 3);
                assert_eq!(insert.tuple.columns[0], Some(b"1".to_vec()));
                assert_eq!(insert.tuple.columns[1], Some(b"Alice".to_vec()));
                assert_eq!(insert.tuple.columns[2], Some(b"t".to_vec()));
            }
            _ => panic!("Expected INSERT message"),
        }
    }

    #[test]
    fn test_update_message_parsing() {
        let mut parser = create_test_parser();
        register_test_relation(&mut parser);

        // Create an UPDATE message with old and new tuples
        let mut msg = BytesMut::new();
        msg.extend_from_slice(b"U"); // Message type
        msg.extend_from_slice(&123u32.to_be_bytes()); // Relation ID

        // Old tuple
        msg.put_u8(b'O'); // Old tuple follows
        msg.extend_from_slice(&3u16.to_be_bytes()); // Number of columns

        // Old values
        msg.put_u8(b't');
        msg.extend_from_slice(&1u32.to_be_bytes());
        msg.extend_from_slice(b"1");

        msg.put_u8(b't');
        msg.extend_from_slice(&5u32.to_be_bytes());
        msg.extend_from_slice(b"Alice");

        msg.put_u8(b't');
        msg.extend_from_slice(&1u32.to_be_bytes());
        msg.extend_from_slice(b"t");

        // New tuple
        msg.put_u8(b'N'); // New tuple follows
        msg.extend_from_slice(&3u16.to_be_bytes()); // Number of columns

        // New values
        msg.put_u8(b't');
        msg.extend_from_slice(&1u32.to_be_bytes());
        msg.extend_from_slice(b"1");

        msg.put_u8(b't');
        msg.extend_from_slice(&3u32.to_be_bytes());
        msg.extend_from_slice(b"Bob");

        msg.put_u8(b't');
        msg.extend_from_slice(&1u32.to_be_bytes());
        msg.extend_from_slice(b"f");

        let result = parser.parse_message(&msg);
        assert!(result.is_ok());

        match result.unwrap() {
            Some(PgOutputMessage::Update(update)) => {
                assert_eq!(update.relation_id, 123);
                assert!(update.old_tuple.is_some());
                assert_eq!(update.new_tuple.columns.len(), 3);
                assert_eq!(update.new_tuple.columns[1], Some(b"Bob".to_vec()));
                assert_eq!(update.new_tuple.columns[2], Some(b"f".to_vec()));
            }
            _ => panic!("Expected UPDATE message"),
        }
    }

    #[test]
    fn test_delete_message_parsing() {
        let mut parser = create_test_parser();
        register_test_relation(&mut parser);

        // Create a DELETE message
        let mut msg = BytesMut::new();
        msg.extend_from_slice(b"D"); // Message type
        msg.extend_from_slice(&123u32.to_be_bytes()); // Relation ID

        // Old tuple (key only)
        msg.put_u8(b'K'); // Key tuple follows
        msg.extend_from_slice(&1u16.to_be_bytes()); // Number of columns (key only)

        // Key value
        msg.put_u8(b't');
        msg.extend_from_slice(&1u32.to_be_bytes());
        msg.extend_from_slice(b"1");

        let result = parser.parse_message(&msg);
        assert!(result.is_ok());

        match result.unwrap() {
            Some(PgOutputMessage::Delete(delete)) => {
                assert_eq!(delete.relation_id, 123);
                assert_eq!(delete.old_tuple.columns.len(), 1);
                assert_eq!(delete.old_tuple.columns[0], Some(b"1".to_vec()));
            }
            _ => panic!("Expected DELETE message"),
        }
    }

    #[test]
    fn test_keepalive_message_handling() {
        let mut parser = create_test_parser();

        // Create a keepalive message
        let mut msg = BytesMut::new();
        msg.extend_from_slice(b"k"); // Message type
        msg.extend_from_slice(&1234567890u64.to_be_bytes()); // WAL end
        msg.extend_from_slice(&1640995200000000i64.to_be_bytes()); // Timestamp
        msg.put_u8(1); // Reply requested

        let result = parser.parse_message(&msg);
        assert!(result.is_ok());

        match result.unwrap() {
            Some(PgOutputMessage::Keepalive(keepalive)) => {
                assert_eq!(keepalive.wal_end, 1234567890);
                assert!(keepalive.reply_requested);
            }
            _ => panic!("Expected KEEPALIVE message"),
        }
    }

    #[test]
    fn test_type_message_parsing() {
        let mut parser = create_test_parser();

        // Create a TYPE message
        let mut msg = BytesMut::new();
        msg.extend_from_slice(b"Y"); // Message type
        msg.extend_from_slice(&456u32.to_be_bytes()); // Type ID
        msg.extend_from_slice(b"public\0"); // Namespace
        msg.extend_from_slice(b"user_status\0"); // Type name

        let result = parser.parse_message(&msg);
        assert!(result.is_ok());

        match result.unwrap() {
            Some(PgOutputMessage::Type(typ)) => {
                assert_eq!(typ.type_id, 456);
                assert_eq!(typ.namespace, "public");
                assert_eq!(typ.name, "user_status");
            }
            _ => panic!("Expected TYPE message"),
        }
    }

    #[test]
    fn test_null_value_handling() {
        let mut parser = create_test_parser();
        register_test_relation(&mut parser);

        // Create an INSERT with NULL values
        let mut msg = BytesMut::new();
        msg.extend_from_slice(b"I");
        msg.extend_from_slice(&123u32.to_be_bytes());
        msg.put_u8(b'N');
        msg.extend_from_slice(&3u16.to_be_bytes());

        // id = 1
        msg.put_u8(b't');
        msg.extend_from_slice(&1u32.to_be_bytes());
        msg.extend_from_slice(b"1");

        // name = NULL
        msg.put_u8(b'n'); // Null indicator

        // active = true
        msg.put_u8(b't');
        msg.extend_from_slice(&1u32.to_be_bytes());
        msg.extend_from_slice(b"t");

        let result = parser.parse_message(&msg);
        assert!(result.is_ok());

        match result.unwrap() {
            Some(PgOutputMessage::Insert(insert)) => {
                assert_eq!(insert.tuple.columns.len(), 3);
                assert_eq!(insert.tuple.columns[0], Some(b"1".to_vec()));
                assert_eq!(insert.tuple.columns[1], None); // NULL
                assert_eq!(insert.tuple.columns[2], Some(b"t".to_vec()));
            }
            _ => panic!("Expected INSERT message"),
        }
    }

    #[test]
    fn test_truncate_message_parsing() {
        let mut parser = create_test_parser();

        // Create a TRUNCATE message
        let mut msg = BytesMut::new();
        msg.extend_from_slice(b"T"); // Message type
        msg.extend_from_slice(&2u32.to_be_bytes()); // Number of relations
        msg.put_u8(0b00000001); // Options (CASCADE)
        msg.extend_from_slice(&123u32.to_be_bytes()); // Relation ID 1
        msg.extend_from_slice(&456u32.to_be_bytes()); // Relation ID 2

        let result = parser.parse_message(&msg);
        assert!(result.is_ok());

        match result.unwrap() {
            Some(PgOutputMessage::Truncate(truncate)) => {
                assert_eq!(truncate.relation_count, 2);
                assert_eq!(truncate.relation_ids.len(), 2);
                assert_eq!(truncate.relation_ids[0], 123);
                assert_eq!(truncate.relation_ids[1], 456);
                assert_eq!(truncate.options, 1); // CASCADE flag
            }
            _ => panic!("Expected TRUNCATE message"),
        }
    }

    #[test]
    fn test_empty_message_handling() {
        let mut parser = create_test_parser();
        let result = parser.parse_message(&[]);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_unknown_message_type() {
        let mut parser = create_test_parser();

        // Unknown message type 'X'
        let mut msg = BytesMut::new();
        msg.extend_from_slice(b"X");
        msg.extend_from_slice(&[0u8; 10]); // Random data

        let result = parser.parse_message(&msg);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none()); // Parser skips unknown messages
    }

    #[test]
    fn test_get_relation() {
        let mut parser = create_test_parser();

        // Register a relation
        register_test_relation(&mut parser);

        // Should be able to retrieve it
        let relation = parser.get_relation(123);
        assert!(relation.is_some());
        let rel = relation.unwrap();
        assert_eq!(rel.relation_name, "users");
        assert_eq!(rel.columns.len(), 3);

        // Non-existent relation
        assert!(parser.get_relation(999).is_none());
    }

    // Helper function to register a test relation
    fn register_test_relation(parser: &mut PgOutputParser) {
        let mut msg = BytesMut::new();
        msg.extend_from_slice(b"R");
        msg.extend_from_slice(&123u32.to_be_bytes());
        msg.extend_from_slice(b"public\0");
        msg.extend_from_slice(b"users\0");
        msg.put_u8(b'd');
        msg.extend_from_slice(&3u16.to_be_bytes());

        // Column 1: id
        msg.put_u8(1); // Flags
        msg.extend_from_slice(b"id\0");
        msg.extend_from_slice(&23u32.to_be_bytes());
        msg.extend_from_slice(&(-1i32).to_be_bytes());

        // Column 2: name
        msg.put_u8(0);
        msg.extend_from_slice(b"name\0");
        msg.extend_from_slice(&25u32.to_be_bytes());
        msg.extend_from_slice(&(-1i32).to_be_bytes());

        // Column 3: active
        msg.put_u8(0);
        msg.extend_from_slice(b"active\0");
        msg.extend_from_slice(&16u32.to_be_bytes());
        msg.extend_from_slice(&(-1i32).to_be_bytes());

        let _ = parser.parse_message(&msg);
    }
}
