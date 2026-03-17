//! MCP (Model Context Protocol) client for composite/smart tool invocations.
//!
//! Communicates with the Newsleopard MCP server over JSON-RPC 2.0 / HTTPS.
//! Used by both `nl mcp call` (generic passthrough) and named subcommand aliases
//! (e.g. `nl edm campaign analyze`).

use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::retry::with_mcp_retry;
use super::ApiClient;
use crate::error::NlError;

/// Default MCP server URL.
pub const DEFAULT_MCP_URL: &str = "https://mcp.newsleopard.com";

/// MCP JSON-RPC request envelope.
#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    jsonrpc: &'static str,
    id: u64,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<Value>,
}

/// MCP JSON-RPC response envelope.
#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    #[allow(dead_code)]
    id: Option<u64>,
    result: Option<Value>,
    error: Option<JsonRpcError>,
}

/// JSON-RPC error object.
#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i64,
    message: String,
}

/// Describes a single MCP tool (from `tools/list`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "inputSchema", skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<Value>,
}

/// MCP client that wraps the shared `ApiClient` and adds MCP protocol handling.
pub struct McpClient<'a> {
    client: &'a ApiClient,
    api_key: String,
    base_url: String,
    session_id: Option<String>,
    next_id: AtomicU64,
}

impl<'a> McpClient<'a> {
    /// Creates a new `McpClient`.
    pub fn new(client: &'a ApiClient, api_key: &str, base_url: &str) -> Self {
        // Strip trailing slash for consistent URL building.
        let base_url = base_url.trim_end_matches('/').to_string();
        Self {
            client,
            api_key: api_key.to_string(),
            base_url,
            session_id: None,
            next_id: AtomicU64::new(1),
        }
    }

    /// Returns the next JSON-RPC request ID.
    fn next_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    /// MCP endpoint URL.
    fn mcp_url(&self) -> String {
        format!("{}/mcp", self.base_url)
    }

    /// Performs the MCP initialize handshake to negotiate protocol version
    /// and obtain a session ID.
    ///
    /// Called lazily on the first tool invocation. The `notifications/initialized`
    /// notification is sent as fire-and-forget in `ensure_initialized` so it
    /// can overlap with the subsequent RPC call.
    pub async fn initialize(&mut self) -> Result<(), NlError> {
        if self.session_id.is_some() {
            return Ok(());
        }

        let url = self.mcp_url();
        self.client.log_request("POST", &url);

        // Check dry-run.
        if let Some(err) = self.client.check_dry_run(
            "POST",
            &url,
            &self.api_key,
            Some(&serde_json::json!({
                "jsonrpc": "2.0",
                "method": "initialize",
                "params": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {},
                    "clientInfo": {
                        "name": "nl-cli",
                        "version": env!("CARGO_PKG_VERSION")
                    }
                }
            })),
        ) {
            return Err(err);
        }

        let request = JsonRpcRequest {
            jsonrpc: "2.0",
            id: self.next_id(),
            method: "initialize".to_string(),
            params: Some(serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "nl-cli",
                    "version": env!("CARGO_PKG_VERSION")
                }
            })),
        };

        let resp = self.send_raw(&request).await?;

        // Extract session ID from response if the server provides one.
        if let Some(result) = &resp.result {
            if let Some(sid) = result.get("sessionId").and_then(|v| v.as_str()) {
                self.session_id = Some(sid.to_string());
            }
        }

        // Fire the initialized notification in the background so it
        // overlaps with the next RPC call instead of blocking.
        let notif_url = self.mcp_url();
        let notif_api_key = self.api_key.clone();
        let notif_session_id = self.session_id.clone();
        let notif_http = self.client.http.clone();

        // NOTE: builds the request manually (duplicates header logic from
        // send_raw) because this is fire-and-forget in a detached task.
        tokio::spawn(async move {
            // JSON-RPC notifications MUST NOT include an "id" field.
            let body = serde_json::json!({
                "jsonrpc": "2.0",
                "method": "notifications/initialized"
            });
            let mut req = notif_http
                .post(&notif_url)
                .header("x-api-key", &notif_api_key)
                .header("content-type", "application/json");
            if let Some(ref sid) = notif_session_id {
                req = req.header("mcp-session-id", sid);
            }
            // Best-effort: ignore errors on the notification.
            let _ = req.json(&body).send().await;
        });

        Ok(())
    }

    /// Lists all available MCP tools.
    pub async fn list_tools(&mut self) -> Result<Vec<ToolInfo>, NlError> {
        self.ensure_initialized().await?;

        let result = self.rpc_call("tools/list", None).await?;

        let tools = result
            .get("tools")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let tool_infos: Vec<ToolInfo> = tools
            .into_iter()
            .map(serde_json::from_value)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| NlError::Api {
                status: 0,
                code: None,
                message: format!("Failed to parse MCP tool list: {e}"),
            })?;

        Ok(tool_infos)
    }

    /// Calls an MCP tool by name with the given arguments.
    ///
    /// This is the core method used by both `nl mcp call` and named subcommand aliases.
    pub async fn call_tool(&mut self, name: &str, args: Value) -> Result<Value, NlError> {
        self.ensure_initialized().await?;

        let params = serde_json::json!({
            "name": name,
            "arguments": args,
        });

        let result = self.rpc_call("tools/call", Some(params)).await?;

        // Check for MCP-level tool error (isError: true).
        if result
            .get("isError")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            let error_text = extract_content_text(&result)
                .unwrap_or_else(|| "Unknown MCP tool error".to_string());
            return Err(NlError::Api {
                status: 0,
                code: None,
                message: error_text,
            });
        }

        // Extract content[0].text and parse as JSON, or return the raw result.
        match extract_content_text(&result) {
            Some(text) => {
                // Try to parse as JSON; if it fails, return as a string value.
                match serde_json::from_str::<Value>(&text) {
                    Ok(parsed) => Ok(parsed),
                    Err(_) => Ok(Value::String(text)),
                }
            }
            None => Ok(result),
        }
    }

    /// Ensures the MCP handshake has been performed.
    async fn ensure_initialized(&mut self) -> Result<(), NlError> {
        if self.session_id.is_none() {
            self.initialize().await?;
        }
        Ok(())
    }

    /// Sends a JSON-RPC call with retry logic for transient errors.
    async fn rpc_call(&self, method: &str, params: Option<Value>) -> Result<Value, NlError> {
        let url = self.mcp_url();
        self.client.log_request("POST", &url);

        let request_body = JsonRpcRequest {
            jsonrpc: "2.0",
            id: self.next_id(),
            method: method.to_string(),
            params,
        };
        let body_value = serde_json::to_value(&request_body)?;

        // Check dry-run.
        if let Some(err) = self
            .client
            .check_dry_run("POST", &url, &self.api_key, Some(&body_value))
        {
            return Err(err);
        }

        let resp = self.send_raw(&request_body).await?;

        if let Some(err) = resp.error {
            // Preserve negative JSON-RPC error codes (e.g., -32601) safely as Option<i64>.
            // Clamp to None only if the code is negative (not representable as u32).
            let code = if err.code < 0 { None } else { Some(err.code) };
            return Err(NlError::Api {
                status: 0,
                code,
                message: err.message,
            });
        }

        resp.result.ok_or_else(|| NlError::Api {
            status: 0,
            code: None,
            message: "MCP response contained neither result nor error".to_string(),
        })
    }

    /// Low-level: sends a JSON-RPC request with retry.
    async fn send_raw(&self, request: &JsonRpcRequest) -> Result<JsonRpcResponse, NlError> {
        let url = self.mcp_url();
        let api_key = self.api_key.clone();
        let session_id = self.session_id.clone();
        let body = serde_json::to_string(request)?;
        let http = self.client.http.clone();

        with_mcp_retry(|| {
            let url = url.clone();
            let api_key = api_key.clone();
            let session_id = session_id.clone();
            let body = body.clone();
            let http = http.clone();
            async move {
                let mut req = http
                    .post(&url)
                    .header("x-api-key", &api_key)
                    .header("content-type", "application/json");

                if let Some(ref sid) = session_id {
                    req = req.header("mcp-session-id", sid);
                }

                let start = std::time::Instant::now();
                let response = req
                    .body(body)
                    .send()
                    .await
                    .map_err(|e| NlError::Network(e.to_string()))?;
                let status = response.status().as_u16();
                let elapsed = start.elapsed().as_millis();

                match status {
                    401 | 403 => {
                        let text = response.text().await.unwrap_or_default();
                        Err(NlError::Auth(format!(
                            "MCP authentication failed (HTTP {status}): {text}"
                        )))
                    }
                    429 => Err(NlError::RateLimit("MCP rate limit exceeded".to_string())),
                    s if s >= 500 => {
                        let text = response.text().await.unwrap_or_default();
                        Err(NlError::Network(format!(
                            "MCP server error (HTTP {s}): {text}"
                        )))
                    }
                    _ => {
                        let text = response
                            .text()
                            .await
                            .map_err(|e| NlError::Network(e.to_string()))?;
                        // Log after we have the body text.
                        if elapsed > 0 {
                            // Verbose logging is handled at the ApiClient level
                            tracing::debug!(
                                "[MCP {} {}ms] {}",
                                status,
                                elapsed,
                                &text[..text.len().min(200)]
                            );
                        }
                        let parsed: JsonRpcResponse =
                            serde_json::from_str(&text).map_err(|e| NlError::Api {
                                status,
                                code: None,
                                message: format!("Failed to parse MCP JSON-RPC response: {e}"),
                            })?;
                        Ok(parsed)
                    }
                }
            }
        })
        .await
    }
}

/// Extracts the text from the first content block: `result.content[0].text`.
fn extract_content_text(result: &Value) -> Option<String> {
    result
        .get("content")?
        .as_array()?
        .first()?
        .get("text")?
        .as_str()
        .map(|s| s.to_string())
}

/// Parses `--arg key=value` pairs into a JSON object.
pub fn parse_kv_args(args: &[String]) -> Result<Value, NlError> {
    let mut map = serde_json::Map::new();
    for arg in args {
        let (key, value) = arg.split_once('=').ok_or_else(|| {
            NlError::Validation(format!("Invalid --arg format '{arg}'. Expected key=value"))
        })?;
        // Try to parse value as JSON (number, bool, object, array).
        // Fall back to string if it doesn't parse.
        let json_value =
            serde_json::from_str::<Value>(value).unwrap_or(Value::String(value.to_string()));
        map.insert(key.to_string(), json_value);
    }
    Ok(Value::Object(map))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_content_text() {
        let result = serde_json::json!({
            "content": [
                {"type": "text", "text": "{\"status\":\"ok\"}"}
            ]
        });
        assert_eq!(
            extract_content_text(&result),
            Some("{\"status\":\"ok\"}".to_string())
        );
    }

    #[test]
    fn test_extract_content_text_empty_content() {
        let result = serde_json::json!({"content": []});
        assert_eq!(extract_content_text(&result), None);
    }

    #[test]
    fn test_extract_content_text_no_content() {
        let result = serde_json::json!({"data": "something"});
        assert_eq!(extract_content_text(&result), None);
    }

    #[test]
    fn test_parse_kv_args_basic() {
        let args = vec!["sn=12345".to_string(), "days=30".to_string()];
        let result = parse_kv_args(&args).unwrap();
        // Numeric strings parse as numbers; use quotes for string values.
        assert_eq!(result["sn"], 12345);
        assert_eq!(result["days"], 30);
    }

    #[test]
    fn test_parse_kv_args_string_value() {
        let args = vec!["name=hello world".to_string()];
        let result = parse_kv_args(&args).unwrap();
        assert_eq!(result["name"], "hello world");
    }

    #[test]
    fn test_parse_kv_args_json_value() {
        let args = vec!["sns=[\"A\",\"B\"]".to_string()];
        let result = parse_kv_args(&args).unwrap();
        assert!(result["sns"].is_array());
    }

    #[test]
    fn test_parse_kv_args_bool() {
        let args = vec!["verbose=true".to_string()];
        let result = parse_kv_args(&args).unwrap();
        assert_eq!(result["verbose"], true);
    }

    #[test]
    fn test_parse_kv_args_invalid() {
        let args = vec!["no-equals-sign".to_string()];
        let result = parse_kv_args(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_kv_args_empty() {
        let args: Vec<String> = vec![];
        let result = parse_kv_args(&args).unwrap();
        assert!(result.as_object().unwrap().is_empty());
    }

    #[test]
    fn test_default_mcp_url() {
        assert_eq!(DEFAULT_MCP_URL, "https://mcp.newsleopard.com");
    }

    #[test]
    fn test_mcp_client_url_trailing_slash() {
        let client = ApiClient::new(false, 0);
        let mcp = McpClient::new(&client, "key", "https://mcp.example.com/");
        assert_eq!(mcp.mcp_url(), "https://mcp.example.com/mcp");
    }

    #[test]
    fn test_mcp_client_url_no_trailing_slash() {
        let client = ApiClient::new(false, 0);
        let mcp = McpClient::new(&client, "key", "https://mcp.example.com");
        assert_eq!(mcp.mcp_url(), "https://mcp.example.com/mcp");
    }

    #[test]
    fn test_tool_info_deserialization() {
        let json = serde_json::json!({
            "name": "analyze_campaign",
            "description": "Analyze campaign performance",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "sn": {"type": "string"}
                }
            }
        });
        let info: ToolInfo = serde_json::from_value(json).unwrap();
        assert_eq!(info.name, "analyze_campaign");
        assert_eq!(info.description.unwrap(), "Analyze campaign performance");
        assert!(info.input_schema.is_some());
    }

    #[test]
    fn test_tool_info_minimal() {
        let json = serde_json::json!({"name": "test_tool"});
        let info: ToolInfo = serde_json::from_value(json).unwrap();
        assert_eq!(info.name, "test_tool");
        assert!(info.description.is_none());
        assert!(info.input_schema.is_none());
    }

    #[test]
    fn test_dry_run_check() {
        let client = ApiClient::new(true, 0);
        let mcp = McpClient::new(&client, "test-key", "https://mcp.example.com");
        let dry_run = client.check_dry_run(
            "POST",
            &mcp.mcp_url(),
            "test-key",
            Some(&serde_json::json!({"method": "tools/call"})),
        );
        assert!(dry_run.is_some());
        match dry_run.unwrap() {
            NlError::DryRun(info) => {
                assert_eq!(info.method, "POST");
                assert!(info.url.contains("/mcp"));
            }
            _ => panic!("Expected DryRun"),
        }
    }
}
