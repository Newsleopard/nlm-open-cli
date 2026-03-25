use crate::error::NlError;

/// Format data as YAML.
pub fn format_yaml(data: &serde_json::Value) -> Result<String, NlError> {
    serde_norway::to_string(data).map_err(|e| NlError::Validation(e.to_string()))
}

/// Format a YAML page with `---` separator between pages.
///
/// The first page uses the standard YAML document start (`---` emitted by serde_norway).
/// Subsequent pages prepend an explicit `---` separator to delimit documents.
#[allow(dead_code)]
pub fn format_yaml_page(data: &serde_json::Value, is_first_page: bool) -> Result<String, NlError> {
    let yaml = serde_norway::to_string(data).map_err(|e| NlError::Validation(e.to_string()))?;
    if is_first_page {
        Ok(yaml)
    } else {
        Ok(format!("---\n{}", yaml))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn basic_yaml() {
        let data = json!({"name": "Alice", "age": 30});
        let result = format_yaml(&data).unwrap();
        assert!(result.contains("name"));
        assert!(result.contains("Alice"));
        assert!(result.contains("age"));
        assert!(result.contains("30"));
    }

    #[test]
    fn yaml_array() {
        let data = json!([{"id": 1}, {"id": 2}]);
        let result = format_yaml(&data).unwrap();
        assert!(result.contains("id"));
    }

    #[test]
    fn yaml_null() {
        let data = json!(null);
        let result = format_yaml(&data).unwrap();
        assert!(result.contains("null"));
    }

    #[test]
    fn yaml_nested() {
        let data = json!({"config": {"schedule": {"type": 0}}});
        let result = format_yaml(&data).unwrap();
        assert!(result.contains("config"));
        assert!(result.contains("schedule"));
        assert!(result.contains("type"));
    }

    #[test]
    fn yaml_page_first() {
        let data = json!({"page": 1});
        let result = format_yaml_page(&data, true).unwrap();
        // First page should not have double ---
        let count = result.matches("---").count();
        assert!(
            count <= 1,
            "First page should have at most one --- (from serde_yaml)"
        );
    }

    #[test]
    fn yaml_page_subsequent() {
        let data = json!({"page": 2});
        let result = format_yaml_page(&data, false).unwrap();
        assert!(
            result.starts_with("---\n"),
            "Subsequent pages must start with ---"
        );
    }

    #[test]
    fn yaml_empty_object() {
        let data = json!({});
        let result = format_yaml(&data).unwrap();
        assert!(result.contains("{}"));
    }
}
