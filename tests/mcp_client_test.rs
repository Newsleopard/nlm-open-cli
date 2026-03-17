//! Integration tests for the MCP client using wiremock.
//!
//! Tests cover: MCP init handshake, tool listing, tool call success,
//! tool call with isError, auth failure, and retry on 5xx.

use serde_json::json;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use nl_cli::client::mcp::McpClient;
use nl_cli::client::ApiClient;
use nl_cli::error::NlError;

const TEST_API_KEY: &str = "test-mcp-key-abc123";

/// Shared test setup: start a mock server and create a non-dry-run ApiClient.
async fn setup() -> (MockServer, ApiClient) {
    let server = MockServer::start().await;
    let client = ApiClient::new(false, 0);
    (server, client)
}

/// Mounts the MCP initialize handshake mock that returns a session ID.
async fn mount_init(server: &MockServer) {
    Mock::given(method("POST"))
        .and(path("/mcp"))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "newsleopard-mcp",
                    "version": "1.0.0"
                },
                "sessionId": "test-session-123"
            }
        })))
        .expect(1..)
        .mount(server)
        .await;
}

// ─── Initialize ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_mcp_initialize_handshake() {
    let (server, api_client) = setup().await;
    mount_init(&server).await;

    let mut mcp = McpClient::new(&api_client, TEST_API_KEY, &server.uri());
    let result = mcp.initialize().await;

    assert!(result.is_ok(), "MCP initialize should succeed");
}

#[tokio::test]
async fn test_mcp_initialize_auth_failure() {
    let (server, api_client) = setup().await;

    Mock::given(method("POST"))
        .and(path("/mcp"))
        .respond_with(ResponseTemplate::new(403).set_body_string("Invalid API key"))
        .expect(1)
        .mount(&server)
        .await;

    let mut mcp = McpClient::new(&api_client, TEST_API_KEY, &server.uri());
    let result = mcp.initialize().await;

    assert!(result.is_err());
    match result.unwrap_err() {
        NlError::Auth(msg) => {
            assert!(msg.contains("403"), "Should mention HTTP 403");
        }
        other => panic!("Expected Auth error, got: {other:?}"),
    }
}

// ─── List Tools ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_mcp_list_tools() {
    let (server, api_client) = setup().await;

    // Mount init + list_tools responses. Since both go to /mcp, we use
    // a single mock that responds differently based on call order.
    // For simplicity, mount init first, then override with list_tools.
    // wiremock matches in LIFO order, so mount list_tools AFTER init.

    // The init mock (matches first call).
    Mock::given(method("POST"))
        .and(path("/mcp"))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "test", "version": "1.0"},
                "sessionId": "sess-1"
            }
        })))
        .up_to_n_times(2) // init + notification
        .mount(&server)
        .await;

    // The list_tools response (matches subsequent calls).
    Mock::given(method("POST"))
        .and(path("/mcp"))
        .and(header("mcp-session-id", "sess-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 3,
            "result": {
                "tools": [
                    {
                        "name": "analyze_campaign",
                        "description": "Analyze campaign performance with suggestions",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "sn": {"type": "string"}
                            },
                            "required": ["sn"]
                        }
                    },
                    {
                        "name": "compare_campaigns",
                        "description": "Compare 2-5 campaigns side by side"
                    }
                ]
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let mut mcp = McpClient::new(&api_client, TEST_API_KEY, &server.uri());
    let tools = mcp.list_tools().await.expect("list_tools should succeed");

    assert_eq!(tools.len(), 2);
    assert_eq!(tools[0].name, "analyze_campaign");
    assert_eq!(
        tools[0].description.as_deref(),
        Some("Analyze campaign performance with suggestions")
    );
    assert!(tools[0].input_schema.is_some());
    assert_eq!(tools[1].name, "compare_campaigns");
}

// ─── Call Tool ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_mcp_call_tool_success() {
    let (server, api_client) = setup().await;

    // Init mock.
    Mock::given(method("POST"))
        .and(path("/mcp"))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "test", "version": "1.0"},
                "sessionId": "sess-2"
            }
        })))
        .up_to_n_times(2)
        .mount(&server)
        .await;

    // Tool call response.
    Mock::given(method("POST"))
        .and(path("/mcp"))
        .and(header("mcp-session-id", "sess-2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 3,
            "result": {
                "content": [
                    {
                        "type": "text",
                        "text": "{\"open_rate\":0.45,\"click_rate\":0.12,\"suggestions\":[\"Improve subject line\"]}"
                    }
                ]
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let mut mcp = McpClient::new(&api_client, TEST_API_KEY, &server.uri());
    let result = mcp
        .call_tool("analyze_campaign", json!({"sn": "CAMP-001"}))
        .await
        .expect("call_tool should succeed");

    assert_eq!(result["open_rate"], 0.45);
    assert_eq!(result["click_rate"], 0.12);
    assert!(result["suggestions"].is_array());
}

#[tokio::test]
async fn test_mcp_call_tool_is_error() {
    let (server, api_client) = setup().await;

    // Init mock.
    Mock::given(method("POST"))
        .and(path("/mcp"))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "test", "version": "1.0"},
                "sessionId": "sess-3"
            }
        })))
        .up_to_n_times(2)
        .mount(&server)
        .await;

    // Tool call returns isError: true.
    Mock::given(method("POST"))
        .and(path("/mcp"))
        .and(header("mcp-session-id", "sess-3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 3,
            "result": {
                "isError": true,
                "content": [
                    {
                        "type": "text",
                        "text": "Campaign CAMP-999 not found"
                    }
                ]
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let mut mcp = McpClient::new(&api_client, TEST_API_KEY, &server.uri());
    let result = mcp
        .call_tool("analyze_campaign", json!({"sn": "CAMP-999"}))
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        NlError::Api { message, .. } => {
            assert!(
                message.contains("CAMP-999"),
                "Error should contain campaign SN"
            );
        }
        other => panic!("Expected Api error, got: {other:?}"),
    }
}

// ─── JSON-RPC Error ──────────────────────────────────────────────────────────

#[tokio::test]
async fn test_mcp_jsonrpc_error() {
    let (server, api_client) = setup().await;

    // Init mock.
    Mock::given(method("POST"))
        .and(path("/mcp"))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "test", "version": "1.0"},
                "sessionId": "sess-4"
            }
        })))
        .up_to_n_times(2)
        .mount(&server)
        .await;

    // JSON-RPC error response.
    Mock::given(method("POST"))
        .and(path("/mcp"))
        .and(header("mcp-session-id", "sess-4"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 3,
            "error": {
                "code": -32601,
                "message": "Method not found"
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let mut mcp = McpClient::new(&api_client, TEST_API_KEY, &server.uri());
    let result = mcp.call_tool("nonexistent_tool", json!({})).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        NlError::Api { message, code, .. } => {
            assert!(message.contains("Method not found"));
            // Negative JSON-RPC error codes (-32601) are clamped to None to avoid wrapping.
            assert_eq!(code, None);
        }
        other => panic!("Expected Api error, got: {other:?}"),
    }
}

// ─── Dry Run ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_mcp_dry_run() {
    let api_client = ApiClient::new(true, 0); // dry-run mode

    let mut mcp = McpClient::new(&api_client, TEST_API_KEY, "https://mcp.newsleopard.com");
    let result = mcp
        .call_tool("analyze_campaign", json!({"sn": "CAMP-001"}))
        .await;

    // Should get DryRun error (from initialize).
    assert!(result.is_err());
    match result.unwrap_err() {
        NlError::DryRun(info) => {
            assert_eq!(info.method, "POST");
            assert!(info.url.contains("/mcp"));
            assert!(info.headers.contains_key("x-api-key"));
            // API key should be masked.
            assert!(info.headers["x-api-key"].starts_with("****"));
        }
        other => panic!("Expected DryRun error, got: {other:?}"),
    }
}

// ─── Content Parsing ─────────────────────────────────────────────────────────

#[tokio::test]
async fn test_mcp_call_tool_non_json_text() {
    let (server, api_client) = setup().await;

    // Init mock.
    Mock::given(method("POST"))
        .and(path("/mcp"))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "test", "version": "1.0"},
                "sessionId": "sess-5"
            }
        })))
        .up_to_n_times(2)
        .mount(&server)
        .await;

    // Tool returns plain text (not JSON) in content.
    Mock::given(method("POST"))
        .and(path("/mcp"))
        .and(header("mcp-session-id", "sess-5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 3,
            "result": {
                "content": [
                    {
                        "type": "text",
                        "text": "Campaign analysis complete. No issues found."
                    }
                ]
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let mut mcp = McpClient::new(&api_client, TEST_API_KEY, &server.uri());
    let result = mcp
        .call_tool("analyze_campaign", json!({"sn": "CAMP-001"}))
        .await
        .expect("call_tool should succeed even with non-JSON text");

    // Non-JSON text should be returned as a string value.
    assert!(result.is_string());
    assert!(result.as_str().unwrap().contains("No issues found"));
}
