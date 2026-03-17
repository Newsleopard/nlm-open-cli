//! Integration tests for the EDM API client using wiremock.
//!
//! Each test starts a local mock server, configures expected requests,
//! creates an `EdmClient` pointing at the mock, and verifies both the
//! request shape and the parsed response.

use serde_json::json;
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use nlm_cli::client::edm::EdmClient;
use nlm_cli::client::ApiClient;
use nlm_cli::error::NlError;
use nlm_cli::types::edm::*;

/// Shared test setup: start a mock server and create a non-dry-run ApiClient.
async fn setup() -> (MockServer, ApiClient) {
    let server = MockServer::start().await;
    let client = ApiClient::new(false, 0);
    (server, client)
}

const TEST_API_KEY: &str = "test-api-key-abc123";

// ─── Account ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_get_balance_success() {
    let (server, api_client) = setup().await;

    Mock::given(method("GET"))
        .and(path("/v1/balance"))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "email": 10000,
            "sms": 500
        })))
        .expect(1)
        .mount(&server)
        .await;

    let edm = EdmClient::new_with_base_url(&api_client, TEST_API_KEY, &server.uri());
    let result = edm.get_balance().await;

    let value = result.expect("get_balance should succeed");
    assert_eq!(value["email"], 10000);
    assert_eq!(value["sms"], 500);
}

#[tokio::test]
async fn test_get_balance_auth_error() {
    let (server, api_client) = setup().await;

    Mock::given(method("GET"))
        .and(path("/v1/balance"))
        .respond_with(ResponseTemplate::new(403).set_body_json(json!({
            "errorCode": 40300,
            "message": "Invalid API key"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let edm = EdmClient::new_with_base_url(&api_client, "bad-key", &server.uri());
    let result = edm.get_balance().await;

    match result.unwrap_err() {
        NlError::Api {
            status,
            code,
            message,
        } => {
            assert_eq!(status, 403);
            assert_eq!(code, Some(40300));
            assert_eq!(message, "Invalid API key");
        }
        other => panic!("Expected Api error, got: {:?}", other),
    }
}

// ─── Contacts ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_create_group() {
    let (server, api_client) = setup().await;

    Mock::given(method("POST"))
        .and(path("/v1/contacts/lists/insert"))
        .and(header("x-api-key", TEST_API_KEY))
        .and(body_json(json!({ "name": "VIP Customers" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "sn": "G001"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let edm = EdmClient::new_with_base_url(&api_client, TEST_API_KEY, &server.uri());
    let result = edm.create_group("VIP Customers").await;

    let value = result.expect("create_group should succeed");
    assert_eq!(value["sn"], "G001");
}

#[tokio::test]
async fn test_list_groups() {
    let (server, api_client) = setup().await;

    // list_groups builds query params into the URL directly, so we match the full path+query.
    Mock::given(method("GET"))
        .and(path("/v1/contacts/lists"))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "groups": [
                {
                    "sn": "G001",
                    "name": "VIP",
                    "subscribedCnt": 1500,
                    "excludeCnt": 42,
                    "openedRate": 0.35,
                    "clickedRate": 0.12,
                    "status": "GENERAL",
                    "type": 0
                }
            ],
            "pageInfo": {
                "total": 1,
                "page": 1,
                "size": 10
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let edm = EdmClient::new_with_base_url(&api_client, TEST_API_KEY, &server.uri());
    let result = edm.list_groups(Some(1), Some(10)).await;

    let value = result.expect("list_groups should succeed");
    assert_eq!(value["groups"][0]["sn"], "G001");
    assert_eq!(value["groups"][0]["subscribedCnt"], 1500);
    assert_eq!(value["pageInfo"]["total"], 1);
}

#[tokio::test]
async fn test_import_text() {
    let (server, api_client) = setup().await;

    Mock::given(method("POST"))
        .and(path("/v1/contacts/imports/LIST001/text"))
        .and(header("x-api-key", TEST_API_KEY))
        .and(body_json(json!({
            "csvText": "email\nalice@example.com\nbob@example.com"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "importSn": "IMP001"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let edm = EdmClient::new_with_base_url(&api_client, TEST_API_KEY, &server.uri());
    let request = ImportTextRequest {
        csv_text: "email\nalice@example.com\nbob@example.com".into(),
        webhook_url: None,
    };
    let result = edm.import_text("LIST001", &request).await;

    let value = result.expect("import_text should succeed");
    assert_eq!(value["importSn"], "IMP001");
}

#[tokio::test]
async fn test_import_status_complete() {
    let (server, api_client) = setup().await;

    Mock::given(method("GET"))
        .and(path("/v1/contacts/imports/result/IMP001"))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "importSn": "IMP001",
            "status": "COMPLETED",
            "fileCnt": 1000,
            "insertCnt": 950,
            "duplicateCnt": 40,
            "errCnt": 10,
            "errorDownloadLink": null
        })))
        .expect(1)
        .mount(&server)
        .await;

    let edm = EdmClient::new_with_base_url(&api_client, TEST_API_KEY, &server.uri());
    let result = edm.import_status("IMP001").await;

    let value = result.expect("import_status should succeed");
    assert_eq!(value["status"], "COMPLETED");
    assert_eq!(value["insertCnt"], 950);
    assert_eq!(value["errCnt"], 10);
}

#[tokio::test]
async fn test_remove_contacts() {
    let (server, api_client) = setup().await;

    Mock::given(method("DELETE"))
        .and(path("/v1/contacts/LIST001"))
        .and(header("x-api-key", TEST_API_KEY))
        .and(body_json(json!({
            "field": "email",
            "operator": "eq",
            "value": "alice@example.com"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "removed": 1
        })))
        .expect(1)
        .mount(&server)
        .await;

    let edm = EdmClient::new_with_base_url(&api_client, TEST_API_KEY, &server.uri());
    let request = RemoveContactsRequest {
        field: "email".into(),
        operator: "eq".into(),
        value: "alice@example.com".into(),
    };
    let result = edm.remove_contacts("LIST001", &request).await;

    let value = result.expect("remove_contacts should succeed");
    assert_eq!(value["removed"], 1);
}

// ─── Campaign ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_submit_campaign() {
    let (server, api_client) = setup().await;

    Mock::given(method("POST"))
        .and(path("/v1/campaign/normal/submit"))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "sn": "CAMP001"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let edm = EdmClient::new_with_base_url(&api_client, TEST_API_KEY, &server.uri());
    let request = CampaignSubmitRequest {
        form: CampaignForm {
            name: "Weekly Newsletter".into(),
            selected_lists: vec!["SN1".into(), "SN2".into()],
            exclude_lists: vec![],
        },
        content: CampaignContent {
            subject: "This Week's Highlights".into(),
            from_name: "Brand Newsletter".into(),
            from_address: "newsletter@example.com".into(),
            html_content: "<html><body><p>Hello ${NAME}!</p></body></html>".into(),
            footer_lang: 1,
            preheader: Some("Don't miss out!".into()),
        },
        config: CampaignConfig {
            schedule: ScheduleConfig {
                schedule_type: 0,
                timezone: None,
                schedule_date: None,
            },
            ga: GaConfig {
                enable: true,
                ecommerce_enable: false,
                utm_campaign: Some("weekly-20250115".into()),
                utm_content: None,
            },
        },
    };
    let result = edm.submit_campaign(&request).await;

    let value = result.expect("submit_campaign should succeed");
    assert_eq!(value["sn"], "CAMP001");
}

#[tokio::test]
async fn test_campaign_status() {
    let (server, api_client) = setup().await;

    Mock::given(method("GET"))
        .and(path("/v1/campaign/normal/CAMP001"))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "sn": "CAMP001",
            "name": "Newsletter",
            "status": "SENT",
            "sendTimeType": 0,
            "type": 1,
            "sentBeginDate": "2025-01-15T10:00:00Z"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let edm = EdmClient::new_with_base_url(&api_client, TEST_API_KEY, &server.uri());
    let result = edm.campaign_status("CAMP001").await;

    let value = result.expect("campaign_status should succeed");
    assert_eq!(value["sn"], "CAMP001");
    assert_eq!(value["status"], "SENT");
    assert_eq!(value["sendTimeType"], 0);
}

#[tokio::test]
async fn test_pause_campaign_204() {
    let (server, api_client) = setup().await;

    Mock::given(method("PATCH"))
        .and(path("/v1/campaign/normal/CAMP001"))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let edm = EdmClient::new_with_base_url(&api_client, TEST_API_KEY, &server.uri());
    let result = edm.pause_campaign("CAMP001").await;

    // 204 is treated as NlError::NoContent (exit code 0).
    match result.unwrap_err() {
        NlError::NoContent => {} // expected
        other => panic!("Expected NoContent, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_delete_campaigns() {
    let (server, api_client) = setup().await;

    Mock::given(method("DELETE"))
        .and(path("/v1/campaign/normal"))
        .and(header("x-api-key", TEST_API_KEY))
        .and(body_json(json!({
            "sns": ["SN1", "SN2"]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": ["SN1", "SN2"],
            "sendingCampaign": [],
            "badCampaigns": []
        })))
        .expect(1)
        .mount(&server)
        .await;

    let edm = EdmClient::new_with_base_url(&api_client, TEST_API_KEY, &server.uri());
    let request = CampaignDeleteRequest {
        sns: vec!["SN1".into(), "SN2".into()],
    };
    let result = edm.delete_campaigns(&request).await;

    let value = result.expect("delete_campaigns should succeed");
    assert_eq!(value["success"][0], "SN1");
    assert_eq!(value["success"][1], "SN2");
    assert!(value["badCampaigns"].as_array().unwrap().is_empty());
}

// ─── Report ──────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_report_list() {
    let (server, api_client) = setup().await;

    Mock::given(method("GET"))
        .and(path("/v1/report/campaigns"))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "campaigns": [
                {
                    "sn": "C001",
                    "name": "January Newsletter",
                    "status": "SENT",
                    "sentDate": "2025-01-15"
                }
            ]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let edm = EdmClient::new_with_base_url(&api_client, TEST_API_KEY, &server.uri());
    let result = edm.report_list("2025-01-01", "2025-01-31").await;

    let value = result.expect("report_list should succeed");
    assert_eq!(value["campaigns"][0]["sn"], "C001");
    assert_eq!(value["campaigns"][0]["name"], "January Newsletter");
}

#[tokio::test]
async fn test_report_metrics() {
    let (server, api_client) = setup().await;

    Mock::given(method("POST"))
        .and(path("/v1/report/campaigns/metrics"))
        .and(header("x-api-key", TEST_API_KEY))
        .and(body_json(json!({
            "campaignSns": ["C001", "C002"]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "campaignSn": "C001",
                "name": "Newsletter",
                "channel": "email",
                "subject": "Hello",
                "recipientCnt": 5000,
                "delivered": 4800,
                "bounced": 200,
                "opened": 2400,
                "clicked": 480,
                "distinctClickCnt": 320,
                "complained": 5,
                "unsubscribed": 15
            }
        ])))
        .expect(1)
        .mount(&server)
        .await;

    let edm = EdmClient::new_with_base_url(&api_client, TEST_API_KEY, &server.uri());
    let result = edm.report_metrics(&["C001".into(), "C002".into()]).await;

    let value = result.expect("report_metrics should succeed");
    assert_eq!(value[0]["campaignSn"], "C001");
    assert_eq!(value[0]["recipientCnt"], 5000);
    assert_eq!(value[0]["delivered"], 4800);
}

#[tokio::test]
async fn test_report_export() {
    let (server, api_client) = setup().await;

    Mock::given(method("POST"))
        .and(path("/v1/report/C001/export"))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "PROCESSING"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let edm = EdmClient::new_with_base_url(&api_client, TEST_API_KEY, &server.uri());
    let result = edm.report_export("C001").await;

    let value = result.expect("report_export should succeed");
    assert_eq!(value["status"], "PROCESSING");
}

#[tokio::test]
async fn test_report_download_link() {
    let (server, api_client) = setup().await;

    Mock::given(method("GET"))
        .and(path("/v1/report/C001/link"))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "link": "https://cdn.example.com/reports/C001.csv"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let edm = EdmClient::new_with_base_url(&api_client, TEST_API_KEY, &server.uri());
    let result = edm.report_download_link("C001").await;

    let value = result.expect("report_download_link should succeed");
    assert_eq!(value["link"], "https://cdn.example.com/reports/C001.csv");
}

// ─── Template ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_list_templates() {
    let (server, api_client) = setup().await;

    Mock::given(method("GET"))
        .and(path("/v1/templates"))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            { "id": "T001", "name": "Welcome Email" },
            { "id": "T002", "name": "Monthly Report" }
        ])))
        .expect(1)
        .mount(&server)
        .await;

    let edm = EdmClient::new_with_base_url(&api_client, TEST_API_KEY, &server.uri());
    let result = edm.list_templates().await;

    let value = result.expect("list_templates should succeed");
    assert_eq!(value[0]["id"], "T001");
    assert_eq!(value[1]["name"], "Monthly Report");
}

#[tokio::test]
async fn test_get_template() {
    let (server, api_client) = setup().await;

    Mock::given(method("GET"))
        .and(path("/v1/templates/T001"))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "T001",
            "name": "Welcome Email",
            "html": "<html><body><h1>Welcome!</h1></body></html>"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let edm = EdmClient::new_with_base_url(&api_client, TEST_API_KEY, &server.uri());
    let result = edm.get_template("T001").await;

    let value = result.expect("get_template should succeed");
    assert_eq!(value["id"], "T001");
    assert_eq!(value["name"], "Welcome Email");
    assert!(value["html"]
        .as_str()
        .unwrap()
        .contains("<h1>Welcome!</h1>"));
}

// ─── Automation ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_trigger_automation() {
    let (server, api_client) = setup().await;

    Mock::given(method("POST"))
        .and(path("/v1/automation/event"))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "triggered"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let edm = EdmClient::new_with_base_url(&api_client, TEST_API_KEY, &server.uri());
    let request = AutomationTriggerRequest {
        event: "order_complete".into(),
        recipients: vec![AutomationRecipient {
            name: "Alice".into(),
            address: "alice@example.com".into(),
            variables: Some(std::collections::HashMap::from([(
                "order_id".into(),
                "12345".into(),
            )])),
        }],
    };
    let result = edm.trigger_automation(&request).await;

    let value = result.expect("trigger_automation should succeed");
    assert_eq!(value["status"], "triggered");
}

// ─── Error Handling ──────────────────────────────────────────────────────────

#[tokio::test]
async fn test_api_error_40012_insufficient_balance() {
    let (server, api_client) = setup().await;

    Mock::given(method("POST"))
        .and(path("/v1/campaign/normal/submit"))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "errorCode": 40012,
            "message": "Insufficient balance"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let edm = EdmClient::new_with_base_url(&api_client, TEST_API_KEY, &server.uri());
    let request = CampaignSubmitRequest {
        form: CampaignForm {
            name: "Test".into(),
            selected_lists: vec!["SN1".into()],
            exclude_lists: vec![],
        },
        content: CampaignContent {
            subject: "Hello".into(),
            from_name: "Sender".into(),
            from_address: "test@example.com".into(),
            html_content: "<p>Hi</p>".into(),
            footer_lang: 1,
            preheader: None,
        },
        config: CampaignConfig {
            schedule: ScheduleConfig {
                schedule_type: 0,
                timezone: None,
                schedule_date: None,
            },
            ga: GaConfig {
                enable: false,
                ecommerce_enable: false,
                utm_campaign: None,
                utm_content: None,
            },
        },
    };
    let result = edm.submit_campaign(&request).await;

    match result.unwrap_err() {
        NlError::Api {
            status,
            code,
            message,
        } => {
            assert_eq!(status, 400);
            assert_eq!(code, Some(40012));
            assert_eq!(message, "Insufficient balance");
        }
        other => panic!("Expected Api error, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_api_error_40011_unverified_sender() {
    let (server, api_client) = setup().await;

    Mock::given(method("POST"))
        .and(path("/v1/campaign/normal/submit"))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "errorCode": 40011,
            "message": "Unverified sender address"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let edm = EdmClient::new_with_base_url(&api_client, TEST_API_KEY, &server.uri());
    let request = CampaignSubmitRequest {
        form: CampaignForm {
            name: "Test".into(),
            selected_lists: vec!["SN1".into()],
            exclude_lists: vec![],
        },
        content: CampaignContent {
            subject: "Hello".into(),
            from_name: "Sender".into(),
            from_address: "unverified@example.com".into(),
            html_content: "<p>Hi</p>".into(),
            footer_lang: 1,
            preheader: None,
        },
        config: CampaignConfig {
            schedule: ScheduleConfig {
                schedule_type: 0,
                timezone: None,
                schedule_date: None,
            },
            ga: GaConfig {
                enable: false,
                ecommerce_enable: false,
                utm_campaign: None,
                utm_content: None,
            },
        },
    };
    let result = edm.submit_campaign(&request).await;

    match result.unwrap_err() {
        NlError::Api {
            status,
            code,
            message,
        } => {
            assert_eq!(status, 400);
            assert_eq!(code, Some(40011));
            assert_eq!(message, "Unverified sender address");
        }
        other => panic!("Expected Api error, got: {:?}", other),
    }
}

#[tokio::test]
#[ignore] // Takes ~100s due to retry backoff exhaustion. Run with: cargo test -- --ignored
async fn test_server_error_500() {
    let (server, api_client) = setup().await;

    // 5xx errors are transient and will be retried by the backoff logic.
    // We mount the mock to respond 500 every time; the retry will eventually
    // exhaust the timeout. We configure a short-lived mock to keep the test fast
    // by returning a non-structured body.
    Mock::given(method("GET"))
        .and(path("/v1/balance"))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .mount(&server)
        .await;

    let edm = EdmClient::new_with_base_url(&api_client, TEST_API_KEY, &server.uri());
    let result = edm.get_balance().await;

    // After retries are exhausted, we get the last error back.
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
async fn test_api_key_header_sent_on_every_request() {
    let (server, api_client) = setup().await;

    // This mock requires the x-api-key header; if missing, wiremock will return 404.
    Mock::given(method("GET"))
        .and(path("/v1/templates"))
        .and(header("x-api-key", "custom-key-xyz"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .expect(1)
        .mount(&server)
        .await;

    let edm = EdmClient::new_with_base_url(&api_client, "custom-key-xyz", &server.uri());
    let result = edm.list_templates().await;

    result.expect("Should succeed when correct API key is sent");
}

// ─── List Groups with No Pagination ──────────────────────────────────────────

#[tokio::test]
async fn test_list_groups_no_pagination() {
    let (server, api_client) = setup().await;

    Mock::given(method("GET"))
        .and(path("/v1/contacts/lists"))
        .and(header("x-api-key", TEST_API_KEY))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "groups": [],
            "pageInfo": {
                "total": 0,
                "page": 1,
                "size": 10
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let edm = EdmClient::new_with_base_url(&api_client, TEST_API_KEY, &server.uri());
    let result = edm.list_groups(None, None).await;

    let value = result.expect("list_groups with no pagination should succeed");
    assert!(value["groups"].as_array().unwrap().is_empty());
}

// ─── Import Text with Webhook URL ───────────────────────────────────────────

#[tokio::test]
async fn test_import_text_with_webhook() {
    let (server, api_client) = setup().await;

    Mock::given(method("POST"))
        .and(path("/v1/contacts/imports/LIST002/text"))
        .and(header("x-api-key", TEST_API_KEY))
        .and(body_json(json!({
            "csvText": "email\ntest@example.com",
            "webhookUrl": "https://hooks.example.com/import-done"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "importSn": "IMP002"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let edm = EdmClient::new_with_base_url(&api_client, TEST_API_KEY, &server.uri());
    let request = ImportTextRequest {
        csv_text: "email\ntest@example.com".into(),
        webhook_url: Some("https://hooks.example.com/import-done".into()),
    };
    let result = edm.import_text("LIST002", &request).await;

    let value = result.expect("import_text with webhook should succeed");
    assert_eq!(value["importSn"], "IMP002");
}
