pub mod csv_fmt;
pub mod json;
pub mod table;
pub mod yaml;

use crate::error::NlError;
use serde_json::Value;

/// Flatten a JSON value into a list of (dotted-key, string-value) pairs.
///
/// Nested objects produce keys like `"parent.child.grandchild"`.
/// Arrays of objects are flattened with indexed keys like `"items[0].name"`.
/// Arrays of primitives are serialized as compact JSON strings.
/// Null values become empty strings.
pub(crate) fn flatten_json(value: &Value, prefix: &str) -> Vec<(String, String)> {
    let mut result = Vec::new();

    match value {
        Value::Object(obj) => {
            for (key, val) in obj {
                let full_key = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", prefix, key)
                };
                match val {
                    Value::Object(_) => {
                        result.extend(flatten_json(val, &full_key));
                    }
                    Value::Array(arr) => {
                        if arr.iter().all(|v| v.is_object()) && !arr.is_empty() {
                            for (i, item) in arr.iter().enumerate() {
                                let indexed_key = format!("{}[{}]", full_key, i);
                                result.extend(flatten_json(item, &indexed_key));
                            }
                        } else {
                            result.push((full_key, serde_json::to_string(val).unwrap_or_default()));
                        }
                    }
                    _ => {
                        result.push((full_key, value_to_string(val)));
                    }
                }
            }
        }
        _ => {
            let key = if prefix.is_empty() {
                "value".to_string()
            } else {
                prefix.to_string()
            };
            result.push((key, value_to_string(value)));
        }
    }

    result
}

/// Convert a JSON value to a display string.
/// Null → empty string, strings → unquoted, others → JSON representation.
pub(crate) fn value_to_string(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::Array(_) | Value::Object(_) => serde_json::to_string(value).unwrap_or_default(),
    }
}

/// Output format enum (matches cli::OutputFormat but kept separate for modularity)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Json,
    Table,
    Yaml,
    Csv,
}

/// Format output data based on the selected format.
///
/// `is_piped` controls whether JSON output is pretty-printed (terminal) or compact (piped).
pub fn format_output(
    data: &serde_json::Value,
    format: Format,
    is_piped: bool,
) -> Result<String, NlError> {
    match format {
        Format::Json => json::format_json(data, is_piped),
        Format::Table => table::format_table(data),
        Format::Yaml => yaml::format_yaml(data),
        Format::Csv => csv_fmt::format_csv(data),
    }
}

/// Format a page of data (for `--page-all` streaming).
///
/// JSON uses NDJSON (one object per line). CSV omits the header after the first page.
/// YAML separates pages with `---`.
#[allow(dead_code)]
pub fn format_page(
    data: &serde_json::Value,
    format: Format,
    is_first_page: bool,
) -> Result<String, NlError> {
    match format {
        Format::Json => json::format_ndjson(data),
        Format::Table => table::format_table(data),
        Format::Yaml => yaml::format_yaml_page(data, is_first_page),
        Format::Csv => csv_fmt::format_csv_page(data, is_first_page),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn format_output_delegates_to_json() {
        let data = json!({"key": "value"});
        let result = format_output(&data, Format::Json, false).unwrap();
        assert!(result.contains('\n'), "Pretty JSON should contain newlines");
        assert!(result.contains("\"key\""));

        let compact = format_output(&data, Format::Json, true).unwrap();
        assert!(
            !compact.contains('\n'),
            "Compact JSON should not contain newlines"
        );
    }

    #[test]
    fn format_output_delegates_to_table() {
        let data = json!({"name": "Alice", "age": 30});
        let result = format_output(&data, Format::Table, false).unwrap();
        assert!(result.contains("name"));
        assert!(result.contains("Alice"));
    }

    #[test]
    fn format_output_delegates_to_yaml() {
        let data = json!({"greeting": "hello"});
        let result = format_output(&data, Format::Yaml, false).unwrap();
        assert!(result.contains("greeting"));
        assert!(result.contains("hello"));
    }

    #[test]
    fn format_output_delegates_to_csv() {
        let data = json!([{"a": 1}, {"a": 2}]);
        let result = format_output(&data, Format::Csv, false).unwrap();
        assert!(result.contains("a"));
        assert!(result.contains('1'));
        assert!(result.contains('2'));
    }

    #[test]
    fn format_page_json_uses_ndjson() {
        let data = json!([{"id": 1}, {"id": 2}]);
        let result = format_page(&data, Format::Json, true).unwrap();
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn format_page_csv_omits_header_on_subsequent_pages() {
        let data = json!([{"x": 10}]);
        let first = format_page(&data, Format::Csv, true).unwrap();
        let second = format_page(&data, Format::Csv, false).unwrap();
        // First page has header + data
        assert!(first.lines().count() >= 2);
        // Second page has data only (no header)
        assert_eq!(second.lines().count(), 1);
    }

    #[test]
    fn format_page_yaml_separator() {
        let data = json!({"page": 2});
        let first = format_page(&data, Format::Yaml, true).unwrap();
        assert!(
            !first.starts_with("---\n---"),
            "First page should not have extra separator"
        );

        let second = format_page(&data, Format::Yaml, false).unwrap();
        assert!(
            second.starts_with("---\n"),
            "Subsequent pages should start with ---"
        );
    }
}
