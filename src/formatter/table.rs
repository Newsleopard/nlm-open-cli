use super::{flatten_json, value_to_string};
use crate::error::NlError;
use tabled::{builder::Builder, settings::Style};

/// Format JSON data as a table.
///
/// - Array of objects: each object becomes a row with column headers.
/// - Single object: displayed as a two-column key-value table.
/// - Other values (strings, numbers, booleans, null): displayed as a single-cell table.
/// - Nested objects/arrays are flattened with dot notation.
pub fn format_table(data: &serde_json::Value) -> Result<String, NlError> {
    match data {
        serde_json::Value::Array(arr) if arr.is_empty() => Ok("(empty)".to_string()),
        serde_json::Value::Array(arr) => build_array_table(arr),
        serde_json::Value::Object(obj) if obj.is_empty() => Ok("(empty)".to_string()),
        serde_json::Value::Object(obj) => build_object_table(obj),
        other => {
            let mut builder = Builder::default();
            builder.push_record(["Value"]);
            builder.push_record([value_to_string(other)]);
            let table = builder.build().with(Style::modern()).to_string();
            Ok(table)
        }
    }
}

/// Build a table from an array of objects.
///
/// Collects all unique keys from every object in the array (preserving insertion order)
/// to handle heterogeneous objects, then creates rows with flattened values.
fn build_array_table(arr: &[serde_json::Value]) -> Result<String, NlError> {
    // Collect all unique keys across all objects, preserving order of first appearance.
    let mut columns: Vec<String> = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for item in arr {
        let flat = flatten_json(item, "");
        for (key, _) in &flat {
            if seen.insert(key.clone()) {
                columns.push(key.clone());
            }
        }
    }

    if columns.is_empty() {
        return Ok("(empty)".to_string());
    }

    let mut builder = Builder::default();
    builder.push_record(&columns);

    for item in arr {
        let flat: std::collections::HashMap<String, String> =
            flatten_json(item, "").into_iter().collect();
        let row: Vec<String> = columns
            .iter()
            .map(|col| flat.get(col).cloned().unwrap_or_default())
            .collect();
        builder.push_record(row);
    }

    let table = builder.build().with(Style::modern()).to_string();
    Ok(table)
}

/// Build a two-column key-value table from a single object.
fn build_object_table(obj: &serde_json::Map<String, serde_json::Value>) -> Result<String, NlError> {
    let flat = flatten_json(&serde_json::Value::Object(obj.clone()), "");

    let mut builder = Builder::default();
    builder.push_record(["Key", "Value"]);

    for (key, value) in &flat {
        builder.push_record([key.as_str(), value.as_str()]);
    }

    let table = builder.build().with(Style::modern()).to_string();
    Ok(table)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn array_of_objects() {
        let data = json!([
            {"name": "Alice", "age": 30},
            {"name": "Bob", "age": 25}
        ]);
        let result = format_table(&data).unwrap();
        assert!(result.contains("name"));
        assert!(result.contains("age"));
        assert!(result.contains("Alice"));
        assert!(result.contains("Bob"));
        assert!(result.contains("30"));
        assert!(result.contains("25"));
    }

    #[test]
    fn single_object_key_value() {
        let data = json!({"status": "ok", "count": 42});
        let result = format_table(&data).unwrap();
        assert!(result.contains("Key"));
        assert!(result.contains("Value"));
        assert!(result.contains("status"));
        assert!(result.contains("ok"));
        assert!(result.contains("count"));
        assert!(result.contains("42"));
    }

    #[test]
    fn nested_object_flattened() {
        let data = json!({"config": {"schedule": {"type": 0}}});
        let result = format_table(&data).unwrap();
        assert!(result.contains("config.schedule.type"));
        assert!(result.contains('0'));
    }

    #[test]
    fn empty_array() {
        let data = json!([]);
        let result = format_table(&data).unwrap();
        assert_eq!(result, "(empty)");
    }

    #[test]
    fn empty_object() {
        let data = json!({});
        let result = format_table(&data).unwrap();
        assert_eq!(result, "(empty)");
    }

    #[test]
    fn heterogeneous_objects() {
        let data = json!([
            {"a": 1, "b": 2},
            {"a": 3, "c": 4}
        ]);
        let result = format_table(&data).unwrap();
        // All three columns should be present
        assert!(result.contains('a'));
        assert!(result.contains('b'));
        assert!(result.contains('c'));
    }

    #[test]
    fn null_values_display_as_empty() {
        let data = json!({"key": null});
        let result = format_table(&data).unwrap();
        assert!(result.contains("key"));
        // Null should be an empty cell, not "null"
    }

    #[test]
    fn scalar_value() {
        let data = json!(42);
        let result = format_table(&data).unwrap();
        assert!(result.contains("42"));
    }

    #[test]
    fn flatten_json_basic() {
        let data = json!({"a": {"b": 1, "c": "hello"}});
        let flat: std::collections::HashMap<String, String> =
            flatten_json(&data, "").into_iter().collect();
        assert_eq!(flat.get("a.b").unwrap(), "1");
        assert_eq!(flat.get("a.c").unwrap(), "hello");
    }

    #[test]
    fn flatten_json_deep_nesting() {
        let data = json!({"l1": {"l2": {"l3": true}}});
        let flat: std::collections::HashMap<String, String> =
            flatten_json(&data, "").into_iter().collect();
        assert_eq!(flat.get("l1.l2.l3").unwrap(), "true");
    }

    #[test]
    fn flatten_json_array_of_primitives() {
        let data = json!({"tags": [1, 2, 3]});
        let flat: std::collections::HashMap<String, String> =
            flatten_json(&data, "").into_iter().collect();
        assert_eq!(flat.get("tags").unwrap(), "[1,2,3]");
    }
}
