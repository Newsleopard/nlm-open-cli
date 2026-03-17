use crate::error::NlError;

/// Pretty JSON for terminal, compact JSON for piped output.
pub fn format_json(data: &serde_json::Value, is_piped: bool) -> Result<String, NlError> {
    if is_piped {
        serde_json::to_string(data).map_err(|e| NlError::Validation(e.to_string()))
    } else {
        serde_json::to_string_pretty(data).map_err(|e| NlError::Validation(e.to_string()))
    }
}

/// NDJSON for `--page-all` streaming — one JSON object per line.
///
/// If the input is an array, each element is serialized as a separate line.
/// If the input is a single value, it is serialized as one line.
#[allow(dead_code)]
pub fn format_ndjson(data: &serde_json::Value) -> Result<String, NlError> {
    match data {
        serde_json::Value::Array(arr) => {
            let lines: Result<Vec<String>, _> = arr.iter().map(serde_json::to_string).collect();
            lines
                .map(|l| l.join("\n"))
                .map_err(|e| NlError::Validation(e.to_string()))
        }
        _ => serde_json::to_string(data).map_err(|e| NlError::Validation(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn pretty_json_has_newlines() {
        let data = json!({"name": "test", "value": 42});
        let result = format_json(&data, false).unwrap();
        assert!(result.contains('\n'));
        assert!(result.contains("  ")); // indentation
    }

    #[test]
    fn compact_json_single_line() {
        let data = json!({"name": "test", "value": 42});
        let result = format_json(&data, true).unwrap();
        assert!(!result.contains('\n'));
    }

    #[test]
    fn ndjson_array_one_line_per_item() {
        let data = json!([
            {"id": 1, "name": "a"},
            {"id": 2, "name": "b"},
            {"id": 3, "name": "c"}
        ]);
        let result = format_ndjson(&data).unwrap();
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 3);
        // Each line should be valid JSON
        for line in &lines {
            let _: serde_json::Value = serde_json::from_str(line).unwrap();
        }
    }

    #[test]
    fn ndjson_single_value() {
        let data = json!({"id": 1});
        let result = format_ndjson(&data).unwrap();
        assert!(!result.contains('\n'));
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["id"], 1);
    }

    #[test]
    fn ndjson_empty_array() {
        let data = json!([]);
        let result = format_ndjson(&data).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn pretty_json_null() {
        let data = json!(null);
        let result = format_json(&data, false).unwrap();
        assert_eq!(result, "null");
    }

    #[test]
    fn compact_json_nested() {
        let data = json!({"outer": {"inner": [1, 2, 3]}});
        let result = format_json(&data, true).unwrap();
        assert!(!result.contains('\n'));
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["outer"]["inner"][0], 1);
    }
}
