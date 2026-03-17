//! Integration tests for the Surenotify API client using wiremock.
//!
//! Each test starts a local mock server, configures expected requests,
//! creates a `SurenotifyClient` pointing at the mock, and verifies both
//! the request shape and the parsed response.

use std::collections::HashMap;

use serde_json::json;
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use nl_cli::client::surenotify::SurenotifyClient;
use nl_cli::client::ApiClient;
use nl_cli::error::NlError;
use nl_cli::types::surenotify::*;

/// Shared test setup: start a mock server and create a non-dry-run ApiClient.
async fn setup() -> (MockServer, ApiClient) {
    let server = MockServer::start().await;
    let client = ApiClient::new(false, 0);
    (server, client)
}

const TEST_API_KEY: &str = "sn-test-key-abc123";

// ─── Email ───────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_send_email() {
    let (server, api_client) = setup().await;

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "MSG001",
            "success": [
                { "id": "S001", "address": "alice@example.com" }
            ],
            "failure": {}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let sn = SurenotifyClient::new_with_base_url(&api_client, TEST_API_KEY, &server.uri());
    let request = EmailSendRequest {
        subject: "Order Confirmation".into(),
        from_address: "noreply@example.com".into(),
        content: "<p>Hello {{name}}, your order is confirmed.</p>".into(),
        recipients: vec![EmailRecipient {
            name: "Alice".into(),
            address: "alice@example.com".into(),
            variables: Some(HashMap::from([("name".into(), "Alice".into())])),
        }],
        from_name: Some("MyShop".into()),
        unsubscribed_link: None,
    };
    let result = sn.send_email(&request).await;

    let value = result.expect("send_email should succeed");
    assert_eq!(value["id"], "MSG001");
    assert_eq!(value["success"][0]["id"], "S001");
    assert_eq!(value["success"][0]["address"], "alice@example.com");
}

#[tokio::test]
async fn test_send_email_with_failures() {
    let (server, api_client) = setup().await;

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "MSG002",
            "success": [
                { "id": "S001", "address": "alice@example.com" }
            ],
            "failure": {
                "bob@invalid": "Invalid email address"
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let sn = SurenotifyClient::new_with_base_url(&api_client, TEST_API_KEY, &server.uri());
    let request = EmailSendRequest {
        subject: "Test".into(),
        from_address: "test@example.com".into(),
        content: "<p>Hi</p>".into(),
        recipients: vec![
            EmailRecipient {
                name: "Alice".into(),
                address: "alice@example.com".into(),
                variables: None,
            },
            EmailRecipient {
                name: "Bob".into(),
                address: "bob@invalid".into(),
                variables: None,
            },
        ],
        from_name: None,
        unsubscribed_link: None,
    };
    let result = sn.send_email(&request).await;

    let value = result.expect("send_email should return partial success");
    assert_eq!(value["id"], "MSG002");
    assert_eq!(value["failure"]["bob@invalid"], "Invalid email address");
}

#[tokio::test]
async fn test_email_events() {
    let (server, api_client) = setup().await;

    Mock::given(method("GET"))
        .and(path("/v1/events"))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "events": [
                {
                    "id": "EVT001",
                    "recipient": "alice@example.com",
                    "status": "delivered",
                    "timestamp": "2025-01-15T10:00:00Z",
                    "subject": "Order Confirmation"
                }
            ],
            "total": 1
        })))
        .expect(1)
        .mount(&server)
        .await;

    let sn = SurenotifyClient::new_with_base_url(&api_client, TEST_API_KEY, &server.uri());
    let params = EmailEventsParams {
        id: Some("MSG001".into()),
        status: Some("delivered".into()),
        page: Some(1),
        size: Some(20),
        ..Default::default()
    };
    let result = sn.email_events(&params).await;

    let value = result.expect("email_events should succeed");
    assert_eq!(value["events"][0]["id"], "EVT001");
    assert_eq!(value["events"][0]["status"], "delivered");
    assert_eq!(value["total"], 1);
}

// ─── SMS ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_send_sms() {
    let (server, api_client) = setup().await;

    Mock::given(method("POST"))
        .and(path("/v1/sms/messages"))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "SMS001",
            "success": [
                { "id": "S001", "address": "912345678" }
            ],
            "failure": {}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let sn = SurenotifyClient::new_with_base_url(&api_client, TEST_API_KEY, &server.uri());
    let request = SmsSendRequest {
        content: "Your verification code is {{code}}".into(),
        recipients: vec![SmsRecipient {
            address: "912345678".into(),
            country_code: "886".into(),
            variables: Some(HashMap::from([("code".into(), "123456".into())])),
        }],
        from: None,
        alive_mins: Some(30),
    };
    let result = sn.send_sms(&request).await;

    let value = result.expect("send_sms should succeed");
    assert_eq!(value["id"], "SMS001");
    assert_eq!(value["success"][0]["address"], "912345678");
}

#[tokio::test]
async fn test_sms_events() {
    let (server, api_client) = setup().await;

    Mock::given(method("GET"))
        .and(path("/v1/sms/events"))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "events": [
                {
                    "id": "SMS001",
                    "recipient": "912345678",
                    "country_code": "886",
                    "status": "delivered",
                    "timestamp": "2025-01-15T10:05:00Z",
                    "content": "Your verification code is 123456"
                }
            ],
            "total": 1
        })))
        .expect(1)
        .mount(&server)
        .await;

    let sn = SurenotifyClient::new_with_base_url(&api_client, TEST_API_KEY, &server.uri());
    let params = SmsEventsParams {
        country_code: Some("886".into()),
        status: Some("delivered".into()),
        page: Some(1),
        ..Default::default()
    };
    let result = sn.sms_events(&params).await;

    let value = result.expect("sms_events should succeed");
    assert_eq!(value["events"][0]["id"], "SMS001");
    assert_eq!(value["events"][0]["status"], "delivered");
}

#[tokio::test]
async fn test_exclusive_number() {
    let (server, api_client) = setup().await;

    Mock::given(method("GET"))
        .and(path("/v1/sms/exclusive-number"))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "phoneNumbers": [
                {
                    "phoneNumber": "0912345678",
                    "createDate": "2025-01-01",
                    "updateDate": "2025-01-15"
                }
            ]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let sn = SurenotifyClient::new_with_base_url(&api_client, TEST_API_KEY, &server.uri());
    let result = sn.exclusive_number().await;

    let value = result.expect("exclusive_number should succeed");
    assert_eq!(value["phoneNumbers"][0]["phoneNumber"], "0912345678");
}

// ─── Email Webhook ───────────────────────────────────────────────────────────

#[tokio::test]
async fn test_create_webhook() {
    let (server, api_client) = setup().await;

    Mock::given(method("POST"))
        .and(path("/v1/webhooks"))
        .and(header("x-api-key", TEST_API_KEY))
        .and(body_json(json!({
            "type": 3,
            "url": "https://example.com/webhook/delivery"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "type": 3,
            "url": "https://example.com/webhook/delivery"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let sn = SurenotifyClient::new_with_base_url(&api_client, TEST_API_KEY, &server.uri());
    let request = WebhookRequest {
        event_type: 3,
        url: "https://example.com/webhook/delivery".into(),
    };
    let result = sn.create_webhook(&request).await;

    let value = result.expect("create_webhook should succeed");
    assert_eq!(value["type"], 3);
    assert_eq!(value["url"], "https://example.com/webhook/delivery");
}

#[tokio::test]
async fn test_list_webhooks() {
    let (server, api_client) = setup().await;

    Mock::given(method("GET"))
        .and(path("/v1/webhooks"))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "webhooks": [
                { "type": 3, "url": "https://example.com/delivery" },
                { "type": 4, "url": "https://example.com/open" },
                { "type": 6, "url": "https://example.com/bounce" }
            ]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let sn = SurenotifyClient::new_with_base_url(&api_client, TEST_API_KEY, &server.uri());
    let result = sn.list_webhooks().await;

    let value = result.expect("list_webhooks should succeed");
    let webhooks = value["webhooks"].as_array().unwrap();
    assert_eq!(webhooks.len(), 3);
    assert_eq!(webhooks[0]["type"], 3);
    assert_eq!(webhooks[2]["type"], 6);
}

#[tokio::test]
async fn test_delete_webhook() {
    let (server, api_client) = setup().await;

    Mock::given(method("DELETE"))
        .and(path("/v1/webhooks"))
        .and(header("x-api-key", TEST_API_KEY))
        .and(body_json(json!({ "type": 5 })))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let sn = SurenotifyClient::new_with_base_url(&api_client, TEST_API_KEY, &server.uri());
    let result = sn.delete_webhook(5).await;

    // 204 is mapped to NlError::NoContent (exit code 0, success-like).
    match result.unwrap_err() {
        NlError::NoContent => {} // expected
        other => panic!("Expected NoContent, got: {:?}", other),
    }
}

// ─── SMS Webhook ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_create_sms_webhook() {
    let (server, api_client) = setup().await;

    Mock::given(method("POST"))
        .and(path("/v1/sms/webhooks"))
        .and(header("x-api-key", TEST_API_KEY))
        .and(body_json(json!({
            "type": 6,
            "url": "https://example.com/sms/bounce"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "type": 6,
            "url": "https://example.com/sms/bounce"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let sn = SurenotifyClient::new_with_base_url(&api_client, TEST_API_KEY, &server.uri());
    let request = SmsWebhookRequest {
        event_type: 6,
        url: "https://example.com/sms/bounce".into(),
    };
    let result = sn.create_sms_webhook(&request).await;

    let value = result.expect("create_sms_webhook should succeed");
    assert_eq!(value["type"], 6);
}

#[tokio::test]
async fn test_list_sms_webhooks() {
    let (server, api_client) = setup().await;

    Mock::given(method("GET"))
        .and(path("/v1/sms/webhooks"))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "webhooks": [
                { "type": 3, "url": "https://example.com/sms/delivery" },
                { "type": 6, "url": "https://example.com/sms/bounce" }
            ]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let sn = SurenotifyClient::new_with_base_url(&api_client, TEST_API_KEY, &server.uri());
    let result = sn.list_sms_webhooks().await;

    let value = result.expect("list_sms_webhooks should succeed");
    let webhooks = value["webhooks"].as_array().unwrap();
    assert_eq!(webhooks.len(), 2);
    assert_eq!(webhooks[0]["type"], 3);
    assert_eq!(webhooks[1]["type"], 6);
}

#[tokio::test]
async fn test_delete_sms_webhook() {
    let (server, api_client) = setup().await;

    Mock::given(method("DELETE"))
        .and(path("/v1/sms/webhooks"))
        .and(header("x-api-key", TEST_API_KEY))
        .and(body_json(json!({ "type": 3 })))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let sn = SurenotifyClient::new_with_base_url(&api_client, TEST_API_KEY, &server.uri());
    let result = sn.delete_sms_webhook(3).await;

    match result.unwrap_err() {
        NlError::NoContent => {} // expected
        other => panic!("Expected NoContent, got: {:?}", other),
    }
}

// ─── Domain Verification ────────────────────────────────────────────────────

#[tokio::test]
async fn test_create_domain() {
    let (server, api_client) = setup().await;

    Mock::given(method("POST"))
        .and(path("/v1/domains/mail.example.com"))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "name": "_dmarc.mail.example.com",
                "value": "v=DMARC1; p=none",
                "record_type": 0,
                "valid": false
            },
            {
                "name": "em._domainkey.mail.example.com",
                "value": "CNAME_VALUE",
                "record_type": 1,
                "valid": false
            }
        ])))
        .expect(1)
        .mount(&server)
        .await;

    let sn = SurenotifyClient::new_with_base_url(&api_client, TEST_API_KEY, &server.uri());
    let result = sn.create_domain("mail.example.com").await;

    let value = result.expect("create_domain should succeed");
    let records = value.as_array().unwrap();
    assert_eq!(records.len(), 2);
    assert_eq!(records[0]["name"], "_dmarc.mail.example.com");
    assert_eq!(records[0]["record_type"], 0);
    assert_eq!(records[0]["valid"], false);
}

#[tokio::test]
async fn test_verify_domain() {
    let (server, api_client) = setup().await;

    Mock::given(method("PUT"))
        .and(path("/v1/domains/mail.example.com"))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "name": "_dmarc.mail.example.com",
                "value": "v=DMARC1; p=none",
                "record_type": 0,
                "valid": true
            },
            {
                "name": "em._domainkey.mail.example.com",
                "value": "CNAME_VALUE",
                "record_type": 1,
                "valid": true
            }
        ])))
        .expect(1)
        .mount(&server)
        .await;

    let sn = SurenotifyClient::new_with_base_url(&api_client, TEST_API_KEY, &server.uri());
    let result = sn.verify_domain("mail.example.com").await;

    let value = result.expect("verify_domain should succeed");
    let records = value.as_array().unwrap();
    assert_eq!(records.len(), 2);
    // After DNS is configured, both records should be valid.
    assert_eq!(records[0]["valid"], true);
    assert_eq!(records[1]["valid"], true);
}

#[tokio::test]
async fn test_remove_domain() {
    let (server, api_client) = setup().await;

    Mock::given(method("DELETE"))
        .and(path("/v1/domains/mail.example.com"))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let sn = SurenotifyClient::new_with_base_url(&api_client, TEST_API_KEY, &server.uri());
    let result = sn.remove_domain("mail.example.com").await;

    match result.unwrap_err() {
        NlError::NoContent => {} // expected
        other => panic!("Expected NoContent, got: {:?}", other),
    }
}

// ─── Error Handling ──────────────────────────────────────────────────────────

#[tokio::test]
async fn test_sn_error_handling_400() {
    let (server, api_client) = setup().await;

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "errorCode": 40001,
            "message": "Invalid email address format"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let sn = SurenotifyClient::new_with_base_url(&api_client, TEST_API_KEY, &server.uri());
    let request = EmailSendRequest {
        subject: "Test".into(),
        from_address: "bad-format".into(),
        content: "<p>Hi</p>".into(),
        recipients: vec![EmailRecipient {
            name: "Test".into(),
            address: "test@example.com".into(),
            variables: None,
        }],
        from_name: None,
        unsubscribed_link: None,
    };
    let result = sn.send_email(&request).await;

    match result.unwrap_err() {
        NlError::Api {
            status,
            code,
            message,
        } => {
            assert_eq!(status, 400);
            assert_eq!(code, Some(40001));
            assert_eq!(message, "Invalid email address format");
        }
        other => panic!("Expected Api error, got: {:?}", other),
    }
}

#[tokio::test]
#[ignore] // Takes ~100s due to retry backoff exhaustion. Run with: cargo test -- --ignored
async fn test_sn_server_error_500() {
    let (server, api_client) = setup().await;

    Mock::given(method("GET"))
        .and(path("/v1/sms/exclusive-number"))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .mount(&server)
        .await;

    let sn = SurenotifyClient::new_with_base_url(&api_client, TEST_API_KEY, &server.uri());
    let result = sn.exclusive_number().await;

    match result.unwrap_err() {
        NlError::Api {
            status,
            code,
            message,
        } => {
            assert_eq!(status, 500);
            assert_eq!(code, None);
            assert_eq!(message, "Internal Server Error");
        }
        other => panic!("Expected Api error, got: {:?}", other),
    }
}

// ─── Header Verification ────────────────────────────────────────────────────

#[tokio::test]
async fn test_sn_api_key_header_sent() {
    let (server, api_client) = setup().await;

    // The mock strictly requires the x-api-key header.
    Mock::given(method("GET"))
        .and(path("/v1/webhooks"))
        .and(header("x-api-key", "unique-sn-key-xyz"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "webhooks": []
        })))
        .expect(1)
        .mount(&server)
        .await;

    let sn = SurenotifyClient::new_with_base_url(&api_client, "unique-sn-key-xyz", &server.uri());
    let result = sn.list_webhooks().await;

    result.expect("Should succeed when correct API key is sent");
}

// ─── Email Events with Empty Params ─────────────────────────────────────────

#[tokio::test]
async fn test_email_events_no_filters() {
    let (server, api_client) = setup().await;

    Mock::given(method("GET"))
        .and(path("/v1/events"))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "events": [],
            "total": 0
        })))
        .expect(1)
        .mount(&server)
        .await;

    let sn = SurenotifyClient::new_with_base_url(&api_client, TEST_API_KEY, &server.uri());
    let params = EmailEventsParams::default();
    let result = sn.email_events(&params).await;

    let value = result.expect("email_events with no filters should succeed");
    assert_eq!(value["total"], 0);
    assert!(value["events"].as_array().unwrap().is_empty());
}

// ─── SMS Send with Exclusive Number ─────────────────────────────────────────

#[tokio::test]
async fn test_send_sms_with_from_number() {
    let (server, api_client) = setup().await;

    Mock::given(method("POST"))
        .and(path("/v1/sms/messages"))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "SMS002",
            "success": [
                { "id": "S002", "address": "987654321" }
            ],
            "failure": {}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let sn = SurenotifyClient::new_with_base_url(&api_client, TEST_API_KEY, &server.uri());
    let request = SmsSendRequest {
        content: "Hello from exclusive number".into(),
        recipients: vec![SmsRecipient {
            address: "987654321".into(),
            country_code: "886".into(),
            variables: None,
        }],
        from: Some("0912345678".into()),
        alive_mins: None,
    };
    let result = sn.send_sms(&request).await;

    let value = result.expect("send_sms with from number should succeed");
    assert_eq!(value["id"], "SMS002");
}
