use crate::error::Result;
use crate::models::stream_event::Event;
use crate::models::{CdcEvent, EventType};
use chrono::Utc;

/// Parse test_decoding message into an Event
pub fn parse_test_decoding_message(lsn: String, message: &str) -> Result<Option<Event>> {
    // Parse the operation type and table using split_once to handle colons in data
    let (operation, rest) = match message.split_once(':') {
        Some((op, rest)) => (op.trim(), rest.trim()),
        None => return Ok(None),
    };

    // Extract table name
    let table_parts: Vec<&str> = rest.split_whitespace().collect();
    if table_parts.is_empty() {
        return Ok(None);
    }

    let table_full = table_parts[0];

    // Parse schema and table from format: schema.table
    let (schema, table) = if let Some(dot_pos) = table_full.rfind('.') {
        let schema_part = &table_full[..dot_pos];
        let table_part = &table_full[dot_pos + 1..];
        (schema_part, table_part)
    } else {
        ("public", table_full)
    };

    // Determine event type
    let event_type = match operation {
        "INSERT" => EventType::Create,
        "UPDATE" => EventType::Update,
        "DELETE" => EventType::Delete,
        _ => return Ok(None),
    };

    // The remaining parts after the table name contain the data
    let data_parts: Vec<&str> = rest.split_whitespace().skip(1).collect();

    // Parse the data based on operation type
    let (_primary_key, data) = match event_type {
        EventType::Create => parse_insert_data(&data_parts)?,
        EventType::Update => parse_update_data(&data_parts)?,
        EventType::Delete => parse_delete_data(&data_parts)?,
        _ => return Ok(None),
    };

    let cdc_event = CdcEvent {
        event_type,
        table: table.to_string(),
        schema: schema.to_string(),
        data: match data {
            serde_json::Value::Object(map) => map.into_iter().collect(),
            _ => std::collections::HashMap::new(),
        },
        timestamp: Utc::now(),
        position: Some(crate::models::Position::postgresql(lsn)),
    };

    Ok(Some(Event::Cdc(cdc_event)))
}

fn parse_insert_data(parts: &[&str]) -> Result<(Option<String>, serde_json::Value)> {
    // INSERT format: column1[type]:value1 column2[type]:value2 ...
    let mut fields = serde_json::Map::new();
    let mut primary_key = None;

    for part in parts {
        if let Some((col_info, value)) = part.split_once(':') {
            let column = extract_column_name(col_info);
            let parsed_value = parse_value(value);

            if column == "id" && primary_key.is_none() {
                primary_key = Some(value.to_string());
            }

            fields.insert(column, parsed_value);
        }
    }

    Ok((primary_key, serde_json::Value::Object(fields)))
}

fn parse_update_data(parts: &[&str]) -> Result<(Option<String>, serde_json::Value)> {
    // UPDATE format: old-key: column1[type]:old_value new-key: column1[type]:new_value
    let mut new_fields = serde_json::Map::new();
    let mut old_fields = serde_json::Map::new();
    let mut primary_key = None;
    let mut is_new_section = false;

    for part in parts {
        if *part == "new-key:" {
            is_new_section = true;
            continue;
        }
        if *part == "old-key:" {
            is_new_section = false;
            continue;
        }

        if let Some((col_info, value)) = part.split_once(':') {
            let column = extract_column_name(col_info);
            let parsed_value = parse_value(value);

            if is_new_section {
                if column == "id" && primary_key.is_none() {
                    primary_key = Some(value.to_string());
                }
                new_fields.insert(column.clone(), parsed_value.clone());
            } else {
                old_fields.insert(column.clone(), parsed_value.clone());
            }
        }
    }

    // If we have new fields, use them; otherwise use old fields
    let data = if !new_fields.is_empty() {
        serde_json::Value::Object(new_fields)
    } else {
        serde_json::Value::Object(old_fields)
    };

    Ok((primary_key, data))
}

fn parse_delete_data(parts: &[&str]) -> Result<(Option<String>, serde_json::Value)> {
    // DELETE format: column1[type]:value1 column2[type]:value2 ...
    let mut fields = serde_json::Map::new();
    let mut primary_key = None;

    for part in parts {
        if let Some((col_info, value)) = part.split_once(':') {
            let column = extract_column_name(col_info);

            if column == "id" && primary_key.is_none() {
                primary_key = Some(value.to_string());
            }

            fields.insert(column, parse_value(value));
        }
    }

    Ok((primary_key, serde_json::Value::Object(fields)))
}

fn extract_column_name(col_info: &str) -> String {
    // Extract column name from format: column[type]
    if let Some(bracket_pos) = col_info.find('[') {
        col_info[..bracket_pos].to_string()
    } else {
        col_info.to_string()
    }
}

fn parse_value(value: &str) -> serde_json::Value {
    // Try to parse as number
    if let Ok(num) = value.parse::<i64>() {
        return serde_json::Value::Number(num.into());
    }

    if let Ok(num) = value.parse::<f64>() {
        if let Some(n) = serde_json::Number::from_f64(num) {
            return serde_json::Value::Number(n);
        }
    }

    // Try to parse as boolean
    if value == "true" {
        return serde_json::Value::Bool(true);
    }
    if value == "false" {
        return serde_json::Value::Bool(false);
    }

    // Check for null
    if value == "null" {
        return serde_json::Value::Null;
    }

    // Otherwise, treat as string
    serde_json::Value::String(value.to_string())
}
