use super::{flatten_json, value_to_string};
use crate::error::NlError;
use std::io::Cursor;

/// Format JSON data as CSV.
///
/// - Array of objects: each object becomes a row, keys become column headers.
/// - Single object: one header row plus one data row.
/// - Other values: single column "value" with the serialized data.
///
/// Nested objects/arrays are flattened with dot notation (same as table formatter).
pub fn format_csv(data: &serde_json::Value) -> Result<String, NlError> {
    match data {
        serde_json::Value::Array(arr) if arr.is_empty() => Ok(String::new()),
        serde_json::Value::Array(arr) => write_array_csv(arr, true),
        serde_json::Value::Object(obj) => write_object_csv(obj),
        other => {
            let mut wtr = csv::Writer::from_writer(Cursor::new(Vec::new()));
            wtr.write_record(["value"])
                .map_err(|e| NlError::Validation(e.to_string()))?;
            wtr.write_record([value_to_string(other)])
                .map_err(|e| NlError::Validation(e.to_string()))?;
            writer_to_string(wtr)
        }
    }
}

/// Format a CSV page — only include the header on the first page.
#[allow(dead_code)]
pub fn format_csv_page(data: &serde_json::Value, is_first_page: bool) -> Result<String, NlError> {
    match data {
        serde_json::Value::Array(arr) if arr.is_empty() => Ok(String::new()),
        serde_json::Value::Array(arr) => write_array_csv(arr, is_first_page),
        serde_json::Value::Object(obj) => {
            if is_first_page {
                write_object_csv(obj)
            } else {
                // For single objects on subsequent pages, write data only (no header).
                let flat = flatten_json(&serde_json::Value::Object(obj.clone()), "");
                let mut wtr = csv::Writer::from_writer(Cursor::new(Vec::new()));
                let values: Vec<String> = flat.into_iter().map(|(_, v)| v).collect();
                wtr.write_record(&values)
                    .map_err(|e| NlError::Validation(e.to_string()))?;
                writer_to_string(wtr)
            }
        }
        other => {
            let mut wtr = csv::Writer::from_writer(Cursor::new(Vec::new()));
            if is_first_page {
                wtr.write_record(["value"])
                    .map_err(|e| NlError::Validation(e.to_string()))?;
            }
            wtr.write_record([value_to_string(other)])
                .map_err(|e| NlError::Validation(e.to_string()))?;
            writer_to_string(wtr)
        }
    }
}

/// Write an array of objects as CSV rows.
fn write_array_csv(arr: &[serde_json::Value], include_header: bool) -> Result<String, NlError> {
    // Collect all unique keys across all objects, preserving insertion order.
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
        return Ok(String::new());
    }

    let mut wtr = csv::Writer::from_writer(Cursor::new(Vec::new()));

    if include_header {
        wtr.write_record(&columns)
            .map_err(|e| NlError::Validation(e.to_string()))?;
    }

    for item in arr {
        let flat: std::collections::HashMap<String, String> =
            flatten_json(item, "").into_iter().collect();
        let row: Vec<String> = columns
            .iter()
            .map(|col| flat.get(col).cloned().unwrap_or_default())
            .collect();
        wtr.write_record(&row)
            .map_err(|e| NlError::Validation(e.to_string()))?;
    }

    writer_to_string(wtr)
}

/// Write a single object as a CSV with keys as headers and values as one data row.
fn write_object_csv(obj: &serde_json::Map<String, serde_json::Value>) -> Result<String, NlError> {
    let flat = flatten_json(&serde_json::Value::Object(obj.clone()), "");

    if flat.is_empty() {
        return Ok(String::new());
    }

    let mut wtr = csv::Writer::from_writer(Cursor::new(Vec::new()));

    let headers: Vec<&str> = flat.iter().map(|(k, _)| k.as_str()).collect();
    wtr.write_record(&headers)
        .map_err(|e| NlError::Validation(e.to_string()))?;

    let values: Vec<&str> = flat.iter().map(|(_, v)| v.as_str()).collect();
    wtr.write_record(&values)
        .map_err(|e| NlError::Validation(e.to_string()))?;

    writer_to_string(wtr)
}

/// Flush a csv::Writer and convert its buffer to a trimmed String.
fn writer_to_string(wtr: csv::Writer<Cursor<Vec<u8>>>) -> Result<String, NlError> {
    let cursor = wtr
        .into_inner()
        .map_err(|e| NlError::Validation(e.to_string()))?;
    let bytes = cursor.into_inner();
    String::from_utf8(bytes)
        .map(|s| s.trim_end().to_string())
        .map_err(|e| NlError::Validation(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn csv_array_of_objects() {
        let data = json!([
            {"name": "Alice", "age": 30},
            {"name": "Bob", "age": 25}
        ]);
        let result = format_csv(&data).unwrap();
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 3); // header + 2 rows
        assert!(lines[0].contains("name"));
        assert!(lines[0].contains("age"));
        assert!(lines[1].contains("Alice"));
        assert!(lines[2].contains("Bob"));
    }

    #[test]
    fn csv_single_object() {
        let data = json!({"status": "ok", "count": 5});
        let result = format_csv(&data).unwrap();
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 2); // header + 1 row
        assert!(lines[0].contains("status"));
        assert!(lines[1].contains("ok"));
    }

    #[test]
    fn csv_empty_array() {
        let data = json!([]);
        let result = format_csv(&data).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn csv_nested_flattened() {
        let data = json!([{"config": {"schedule": {"type": 0}}}]);
        let result = format_csv(&data).unwrap();
        assert!(result.contains("config.schedule.type"));
        assert!(result.contains('0'));
    }

    #[test]
    fn csv_null_becomes_empty() {
        let data = json!([{"a": 1, "b": null}]);
        let result = format_csv(&data).unwrap();
        // "b" column value should be empty
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 2);
        // The null field should produce an empty CSV field
        assert!(lines[1].contains("1,"));
    }

    #[test]
    fn csv_heterogeneous_objects() {
        let data = json!([
            {"a": 1, "b": 2},
            {"a": 3, "c": 4}
        ]);
        let result = format_csv(&data).unwrap();
        let lines: Vec<&str> = result.lines().collect();
        // Header should have a, b, c
        assert!(lines[0].contains('a'));
        assert!(lines[0].contains('b'));
        assert!(lines[0].contains('c'));
        // Second row should have empty b
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn csv_page_first_has_header() {
        let data = json!([{"x": 10}]);
        let result = format_csv_page(&data, true).unwrap();
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "x");
    }

    #[test]
    fn csv_page_subsequent_no_header() {
        let data = json!([{"x": 10}]);
        let result = format_csv_page(&data, false).unwrap();
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "10");
    }

    #[test]
    fn csv_scalar_value() {
        let data = json!(42);
        let result = format_csv(&data).unwrap();
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "value");
        assert_eq!(lines[1], "42");
    }

    #[test]
    fn csv_string_with_comma_is_quoted() {
        let data = json!([{"text": "hello, world"}]);
        let result = format_csv(&data).unwrap();
        assert!(result.contains("\"hello, world\""));
    }

    #[test]
    fn csv_empty_object() {
        let data = json!({});
        let result = format_csv(&data).unwrap();
        assert!(result.is_empty());
    }
}
