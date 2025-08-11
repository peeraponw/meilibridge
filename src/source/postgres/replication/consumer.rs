use crate::error::{MeiliBridgeError, Result};
use crate::models::{Event, EventType};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_postgres::Client;
use tracing::{debug, info, warn, error, trace};
use serde_json::json;

pub struct ReplicationConsumer {
    client: Arc<Client>,
    slot_name: String,
    event_tx: mpsc::Sender<Result<Event>>,
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

impl ReplicationConsumer {
    pub fn new(
        client: Client,
        slot_name: String,
        _publication_name: String,
        event_tx: mpsc::Sender<Result<Event>>,
    ) -> Self {
        Self {
            client: Arc::new(client),
            slot_name,
            event_tx,
            shutdown_tx: None,
        }
    }

    pub async fn start_replication(&mut self, start_lsn: Option<String>) -> Result<()> {
        // Check if slot exists and get its plugin type
        let check_slot_query = format!(
            "SELECT plugin FROM pg_replication_slots WHERE slot_name = '{}'",
            self.slot_name
        );
        
        match self.client.query_one(&check_slot_query, &[]).await {
            Ok(row) => {
                let plugin: String = row.get(0);
                info!("Found replication slot '{}' with plugin '{}'", self.slot_name, plugin);
                
                if plugin == "pgoutput" {
                    // Drop and recreate with test_decoding
                    warn!("Slot uses pgoutput plugin which requires binary protocol. Recreating with test_decoding...");
                    self.recreate_slot_with_test_decoding().await?;
                }
                
                self.start_test_decoding_polling(start_lsn).await
            }
            Err(_) => {
                // Slot doesn't exist, create it with test_decoding
                warn!("Replication slot '{}' not found, creating with test_decoding plugin", self.slot_name);
                self.create_slot_with_test_decoding().await?;
                self.start_test_decoding_polling(start_lsn).await
            }
        }
    }

    async fn recreate_slot_with_test_decoding(&self) -> Result<()> {
        // Drop the existing slot
        let drop_query = format!(
            "SELECT pg_drop_replication_slot('{}')",
            self.slot_name
        );
        
        match self.client.execute(&drop_query, &[]).await {
            Ok(_) => info!("Dropped existing replication slot '{}'", self.slot_name),
            Err(e) => warn!("Failed to drop slot: {}", e),
        }
        
        // Create new slot with test_decoding
        self.create_slot_with_test_decoding().await
    }

    async fn create_slot_with_test_decoding(&self) -> Result<()> {
        let query = format!(
            "SELECT pg_create_logical_replication_slot('{}', 'test_decoding')",
            self.slot_name
        );
        
        self.client.execute(&query, &[]).await
            .map_err(|e| MeiliBridgeError::Source(format!("Failed to create slot: {}", e)))?;
        
        info!("Created replication slot '{}' with test_decoding plugin", self.slot_name);
        Ok(())
    }

    async fn start_test_decoding_polling(&mut self, start_lsn: Option<String>) -> Result<()> {
        let lsn = start_lsn.unwrap_or_else(|| "0/0".to_string());
        info!("Starting test_decoding polling from LSN: {}", lsn);
        
        let client = self.client.clone();
        let tx = self.event_tx.clone();
        let slot_name = self.slot_name.clone();
        
        // Create shutdown channel
        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel::<()>();
        self.shutdown_tx = Some(shutdown_tx);
        
        tokio::spawn(async move {
            let mut consecutive_errors = 0;
            const MAX_CONSECUTIVE_ERRORS: u32 = 5;
            
            loop {
                // Check for shutdown signal
                if shutdown_rx.try_recv().is_ok() {
                    info!("Replication consumer received shutdown signal");
                    break;
                }
                
                // Use pg_logical_slot_get_changes for test_decoding
                let query = format!(
                    "SELECT lsn::text, xid::text, data FROM pg_logical_slot_get_changes('{}', NULL, NULL)",
                    slot_name
                );
                
                match client.query(&query, &[]).await {
                    Ok(rows) => {
                        consecutive_errors = 0; // Reset error counter on success
                        
                        if !rows.is_empty() {
                            info!("CDC: Found {} changes from replication slot", rows.len());
                            
                            for row in &rows {
                                let lsn: String = row.get(0);
                                let xid: String = row.get(1);
                                let data: String = row.get(2);
                                
                                debug!("CDC: Raw message - LSN: {}, XID: {}, Data: {}", lsn, xid, data);
                                
                                // Parse test_decoding format
                                if let Some(event) = parse_test_decoding_message(&data) {
                                    debug!("CDC: Successfully parsed event: {:?}", event);
                                    match tx.send(Ok(event)).await {
                                        Ok(_) => debug!("CDC: Event sent to coordinator"),
                                        Err(_) => {
                                            info!("CDC: Replication consumer stopping - channel closed");
                                            return;
                                        }
                                    }
                                } else {
                                    warn!("CDC: Failed to parse test_decoding message: {}", data);
                                }
                            }
                        } else {
                            trace!("CDC: No changes found in this poll cycle");
                        }
                    }
                    Err(e) => {
                        consecutive_errors += 1;
                        error!("Failed to get changes (attempt {}/{}): {}", 
                               consecutive_errors, MAX_CONSECUTIVE_ERRORS, e);
                        
                        // Check if it's a connection error or we've exceeded max errors
                        if e.to_string().contains("connection") || consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
                            error!("Connection lost or max errors reached, stopping replication consumer");
                            let _ = tx.send(Err(MeiliBridgeError::Source(
                                format!("Database connection lost after {} consecutive errors", consecutive_errors)
                            ))).await;
                            return;
                        }
                        
                        // Exponential backoff on errors
                        let backoff_secs = std::cmp::min(2u64.pow(consecutive_errors), 30);
                        warn!("Backing off for {} seconds before retry", backoff_secs);
                        tokio::time::sleep(tokio::time::Duration::from_secs(backoff_secs)).await;
                        continue;
                    }
                }
                
                // Check if channel is still open
                if tx.is_closed() {
                    info!("Replication consumer stopping - channel closed");
                    break;
                }
                
                // Wait before next poll
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        });
        
        Ok(())
    }
    
    pub fn stop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}

/// Parse test_decoding format messages
fn parse_test_decoding_message(data: &str) -> Option<Event> {
    // test_decoding format examples:
    // BEGIN 605
    // table public.users: INSERT: id[integer]:1 email[text]:'test@example.com' ...
    // COMMIT 605
    
    debug!("Parser: Processing message: {}", data);
    
    // Skip transaction control messages
    if data.starts_with("BEGIN") || data.starts_with("COMMIT") {
        debug!("Parser: Skipping transaction control message");
        return None;
    }
    
    // Parse table operations
    if !data.starts_with("table ") {
        debug!("Parser: Message does not start with 'table', skipping");
        return None;
    }
    
    let parts: Vec<&str> = data.splitn(3, ':').collect();
    if parts.len() < 3 {
        warn!("Invalid test_decoding message format: {}", data);
        return None;
    }
    
    // Parse table info
    let table_info = parts[0].trim();
    let table_parts: Vec<&str> = table_info[6..].split('.').collect();
    if table_parts.len() != 2 {
        return None;
    }
    
    let schema = table_parts[0];
    let table = table_parts[1];
    
    // Parse operation
    let operation = parts[1].trim();
    let event_type = match operation {
        "INSERT" => EventType::Create,
        "UPDATE" => EventType::Update,
        "DELETE" => EventType::Delete,
        _ => {
            warn!("Unknown operation type: {}", operation);
            return None;
        }
    };
    
    // Parse data fields
    let data_part = parts[2].trim();
    let data_map = parse_test_decoding_fields(data_part);
    
    let cdc_event = crate::models::CdcEvent {
        event_type: event_type.clone(),
        table: table.to_string(),
        schema: schema.to_string(),
        data: data_map.clone(),
        timestamp: Utc::now(),
        position: None,
    };
    
    debug!("Parser: Created CdcEvent - type: {:?}, schema: {}, table: {}, fields: {}", 
           event_type, schema, table, data_map.len());
    
    let event = Event::Cdc(cdc_event);
    debug!("Parser: Returning Event::Cdc");
    
    Some(event)
}

fn parse_test_decoding_fields(data: &str) -> HashMap<String, serde_json::Value> {
    let mut result = HashMap::new();
    let mut current_field = String::new();
    let mut current_type = String::new();
    let mut current_value = String::new();
    let mut state = ParseState::FieldName;
    let mut in_quotes = false;
    let mut escape_next = false;
    
    #[derive(Debug)]
    enum ParseState {
        FieldName,
        FieldType,
        FieldValue,
    }
    
    for ch in data.chars() {
        if escape_next {
            current_value.push(ch);
            escape_next = false;
            continue;
        }
        
        match state {
            ParseState::FieldName => {
                if ch == '[' {
                    state = ParseState::FieldType;
                } else if ch == ' ' && current_field.is_empty() {
                    // Skip leading spaces
                } else {
                    current_field.push(ch);
                }
            }
            ParseState::FieldType => {
                if ch == ']' {
                    state = ParseState::FieldValue;
                } else {
                    current_type.push(ch);
                }
            }
            ParseState::FieldValue => {
                if ch == ':' && current_value.is_empty() {
                    // Skip the colon separator
                } else if ch == '\'' && !in_quotes {
                    in_quotes = true;
                } else if ch == '\'' && in_quotes {
                    in_quotes = false;
                } else if ch == '\\' && in_quotes {
                    escape_next = true;
                } else if ch == ' ' && !in_quotes && !current_value.is_empty() {
                    // Field complete
                    let value = convert_value(&current_value, &current_type);
                    result.insert(current_field.clone(), value);
                    
                    // Reset for next field
                    current_field.clear();
                    current_type.clear();
                    current_value.clear();
                    state = ParseState::FieldName;
                } else {
                    current_value.push(ch);
                }
            }
        }
    }
    
    // Handle last field
    if !current_field.is_empty() && !current_value.is_empty() {
        let value = convert_value(&current_value, &current_type);
        result.insert(current_field, value);
    }
    
    result
}

fn convert_value(value: &str, type_name: &str) -> serde_json::Value {
    // Handle NULL values
    if value == "null" {
        return json!(null);
    }
    
    // Convert based on type name
    match type_name {
        "integer" | "bigint" | "smallint" => {
            value.parse::<i64>()
                .map(|n| json!(n))
                .unwrap_or_else(|_| json!(value))
        }
        "real" | "double precision" | "numeric" => {
            value.parse::<f64>()
                .map(|n| json!(n))
                .unwrap_or_else(|_| json!(value))
        }
        "boolean" => {
            json!(value == "t" || value == "true")
        }
        "json" | "jsonb" => {
            serde_json::from_str(value)
                .unwrap_or_else(|_| json!(value))
        }
        // Handle array types
        t if t.ends_with("[]") || t.contains("array") => {
            // Try to parse as PostgreSQL array format
            if value.starts_with('{') && value.ends_with('}') {
                parse_postgres_array(value, t)
            } else {
                json!(value)
            }
        }
        _ => {
            // For text and other types, return as string
            json!(value)
        }
    }
}

/// Parse PostgreSQL array string representation
fn parse_postgres_array(value: &str, type_name: &str) -> serde_json::Value {
    // Remove the curly braces
    let content = &value[1..value.len() - 1];
    if content.is_empty() {
        return json!([]);
    }
    
    // Simple parser for array elements
    let mut elements = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut escape_next = false;
    
    for ch in content.chars() {
        if escape_next {
            current.push(ch);
            escape_next = false;
            continue;
        }
        
        match ch {
            '\\' if in_quotes => escape_next = true,
            '"' => in_quotes = !in_quotes,
            ',' if !in_quotes => {
                elements.push(parse_array_element(&current, type_name));
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    
    // Don't forget the last element
    if !current.is_empty() {
        elements.push(parse_array_element(&current, type_name));
    }
    
    json!(elements)
}

/// Parse a single array element
fn parse_array_element(element: &str, array_type: &str) -> serde_json::Value {
    let trimmed = element.trim();
    
    // Handle NULL
    if trimmed == "NULL" {
        return json!(null);
    }
    
    // Remove quotes if present
    let unquoted = if trimmed.starts_with('"') && trimmed.ends_with('"') {
        &trimmed[1..trimmed.len() - 1]
    } else {
        trimmed
    };
    
    // Determine element type from array type
    let element_type = array_type.trim_end_matches("[]").trim_end_matches(" array");
    
    // Convert the element based on its type
    match element_type {
        "integer" | "bigint" | "smallint" => {
            unquoted.parse::<i64>()
                .map(|n| json!(n))
                .unwrap_or_else(|_| json!(unquoted))
        }
        "real" | "double precision" | "numeric" => {
            unquoted.parse::<f64>()
                .map(|n| json!(n))
                .unwrap_or_else(|_| json!(unquoted))
        }
        "boolean" => {
            json!(unquoted == "t" || unquoted == "true")
        }
        "json" | "jsonb" => {
            serde_json::from_str(unquoted)
                .unwrap_or_else(|_| json!(unquoted))
        }
        _ => json!(unquoted)
    }
}