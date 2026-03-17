//! SureNotify API request/response types.
//!
//! Covers all 14 SureNotify endpoints: email (2), SMS (3), email webhook (3),
//! SMS webhook (3), domain verification (3).
//!
//! SureNotify uses `{{variable_name}}` template syntax (distinct from EDM's `${FIELD_NAME}`).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── Email ──────────────────────────────────────────────────────────────────

/// POST /v1/messages — request body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailSendRequest {
    pub subject: String,
    #[serde(rename = "fromAddress")]
    pub from_address: String,
    /// HTML body content.
    pub content: String,
    pub recipients: Vec<EmailRecipient>,
    #[serde(rename = "fromName", skip_serializing_if = "Option::is_none")]
    pub from_name: Option<String>,
    #[serde(rename = "unsubscribedLink", skip_serializing_if = "Option::is_none")]
    pub unsubscribed_link: Option<String>,
}

/// A single email recipient with optional personalization variables.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailRecipient {
    pub name: String,
    pub address: String,
    /// Personalization variables — must be nested under `variables`, never top-level.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<HashMap<String, String>>,
}

/// POST /v1/messages — response body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailSendResponse {
    pub id: String,
    pub success: Vec<EmailSuccess>,
    #[serde(default)]
    pub failure: HashMap<String, String>,
}

/// A successfully queued email recipient.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailSuccess {
    pub id: String,
    pub address: String,
}

/// GET /v1/events — query parameters (built as query string, not JSON body).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EmailEventsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipient: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u32>,
}

/// A single email event returned by GET /v1/events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailEvent {
    pub id: String,
    pub recipient: String,
    pub status: String,
    pub timestamp: String,
    pub subject: String,
}

// ─── SMS ────────────────────────────────────────────────────────────────────

/// POST /v1/sms/messages — request body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsSendRequest {
    pub content: String,
    pub recipients: Vec<SmsRecipient>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    /// Message validity period in minutes (5-480).
    #[serde(rename = "alive_mins", skip_serializing_if = "Option::is_none")]
    pub alive_mins: Option<u16>,
}

/// A single SMS recipient.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsRecipient {
    /// Phone number in digits only (no + or -).
    pub address: String,
    /// Country code, e.g. "886".
    pub country_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<HashMap<String, String>>,
}

/// POST /v1/sms/messages — response body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsSendResponse {
    pub id: String,
    pub success: Vec<SmsSuccess>,
    #[serde(default)]
    pub failure: HashMap<String, String>,
}

/// A successfully queued SMS.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsSuccess {
    pub id: String,
    pub address: String,
}

/// GET /v1/sms/events — query parameters.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SmsEventsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipient: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u32>,
}

/// A single SMS event returned by GET /v1/sms/events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsEvent {
    pub id: String,
    pub recipient: String,
    pub country_code: String,
    pub status: String,
    pub timestamp: String,
    pub content: String,
}

/// GET /v1/sms/exclusive-number — response body
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExclusiveNumberResponse {
    pub phone_numbers: Vec<PhoneNumber>,
}

/// A single exclusive phone number.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhoneNumber {
    pub phone_number: String,
    pub create_date: String,
    pub update_date: String,
}

// ─── Email Webhook ──────────────────────────────────────────────────────────

/// POST /v1/webhooks — request body
///
/// Event types: 3=delivery, 4=open, 5=click, 6=bounce, 7=complaint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookRequest {
    /// Event type: 3=delivery, 4=open, 5=click, 6=bounce, 7=complaint.
    #[serde(rename = "type")]
    pub event_type: u8,
    pub url: String,
}

/// GET /v1/webhooks — response body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookResponse {
    pub webhooks: Vec<WebhookItem>,
}

/// A single webhook registration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookItem {
    #[serde(rename = "type")]
    pub event_type: u8,
    pub url: String,
}

/// DELETE /v1/webhooks — request body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookDeleteRequest {
    #[serde(rename = "type")]
    pub event_type: u8,
}

// ─── SMS Webhook ────────────────────────────────────────────────────────────

/// POST /v1/sms/webhooks — request body
///
/// SMS event types: 3=delivery, 6=bounce
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsWebhookRequest {
    /// 3=delivery, 6=bounce.
    #[serde(rename = "type")]
    pub event_type: u8,
    pub url: String,
}

/// GET /v1/sms/webhooks — response body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsWebhookResponse {
    pub webhooks: Vec<WebhookItem>,
}

/// DELETE /v1/sms/webhooks — request body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsWebhookDeleteRequest {
    #[serde(rename = "type")]
    pub event_type: u8,
}

// ─── Domain Verification ────────────────────────────────────────────────────

/// POST /v1/domains/{domain} — response body.
///
/// Returns a list of DNS records that must be configured.
pub type DomainCreateResponse = Vec<DnsRecord>;

/// A single DNS record for domain verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRecord {
    pub name: String,
    pub value: String,
    /// 0 = TXT, 1 = CNAME.
    pub record_type: u8,
    pub valid: bool,
}

/// PUT /v1/domains/{domain} — response body.
///
/// Returns the same DNS records with updated `valid` status.
pub type DomainVerifyResponse = Vec<DnsRecord>;

// ─── Error Parsing ──────────────────────────────────────────────────────────

/// Standard error body returned by the SureNotify API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnApiErrorResponse {
    #[serde(rename = "errorCode", default)]
    pub error_code: Option<u32>,
    #[serde(default)]
    pub message: String,
}

impl<'a> From<&'a SnApiErrorResponse> for (Option<i64>, String) {
    fn from(err: &'a SnApiErrorResponse) -> Self {
        (err.error_code.map(|c| c as i64), err.message.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_send_request_serialization() {
        let req = EmailSendRequest {
            subject: "Order Confirmation".into(),
            from_address: "noreply@example.com".into(),
            content: "<p>Hello {{name}}</p>".into(),
            recipients: vec![EmailRecipient {
                name: "Alice".into(),
                address: "alice@example.com".into(),
                variables: Some(HashMap::from([("name".into(), "Alice".into())])),
            }],
            from_name: Some("MyShop".into()),
            unsubscribed_link: None,
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["fromAddress"], "noreply@example.com");
        assert_eq!(json["fromName"], "MyShop");
        assert!(json.get("unsubscribedLink").is_none());
        assert_eq!(json["recipients"][0]["variables"]["name"], "Alice");
    }

    #[test]
    fn test_email_send_response_deserialization() {
        let json = r#"{
            "id": "MSG001",
            "success": [
                {"id": "S001", "address": "alice@example.com"}
            ],
            "failure": {
                "bob@invalid": "Invalid email address"
            }
        }"#;
        let resp: EmailSendResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.id, "MSG001");
        assert_eq!(resp.success.len(), 1);
        assert_eq!(resp.success[0].id, "S001");
        assert_eq!(resp.success[0].address, "alice@example.com");
        assert_eq!(resp.failure.len(), 1);
        assert!(resp.failure.contains_key("bob@invalid"));
    }

    #[test]
    fn test_email_send_response_empty_failure() {
        let json = r#"{
            "id": "MSG002",
            "success": [{"id": "S001", "address": "test@example.com"}]
        }"#;
        let resp: EmailSendResponse = serde_json::from_str(json).unwrap();
        assert!(resp.failure.is_empty());
    }

    #[test]
    fn test_email_event_deserialization() {
        let json = r#"{
            "id": "EVT001",
            "recipient": "alice@example.com",
            "status": "delivered",
            "timestamp": "2025-01-15T10:00:00Z",
            "subject": "Hello World"
        }"#;
        let event: EmailEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.id, "EVT001");
        assert_eq!(event.recipient, "alice@example.com");
        assert_eq!(event.status, "delivered");
    }

    #[test]
    fn test_sms_send_request_serialization() {
        let req = SmsSendRequest {
            content: "Your code is {{code}}".into(),
            recipients: vec![SmsRecipient {
                address: "912345678".into(),
                country_code: "886".into(),
                variables: Some(HashMap::from([("code".into(), "123456".into())])),
            }],
            from: None,
            alive_mins: Some(30),
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["recipients"][0]["country_code"], "886");
        assert_eq!(json["alive_mins"], 30);
        assert!(json.get("from").is_none());
    }

    #[test]
    fn test_sms_send_response_deserialization() {
        let json = r#"{
            "id": "SMS001",
            "success": [{"id": "S001", "address": "912345678"}],
            "failure": {}
        }"#;
        let resp: SmsSendResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.id, "SMS001");
        assert_eq!(resp.success[0].address, "912345678");
        assert!(resp.failure.is_empty());
    }

    #[test]
    fn test_sms_event_deserialization() {
        let json = r#"{
            "id": "SMS001",
            "recipient": "912345678",
            "country_code": "886",
            "status": "delivered",
            "timestamp": "2025-01-15T10:00:00Z",
            "content": "Your code is 123456"
        }"#;
        let event: SmsEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.id, "SMS001");
        assert_eq!(event.country_code, "886");
        assert_eq!(event.content, "Your code is 123456");
    }

    #[test]
    fn test_exclusive_number_response() {
        let json = r#"{
            "phoneNumbers": [
                {
                    "phoneNumber": "0912345678",
                    "createDate": "2025-01-01",
                    "updateDate": "2025-01-15"
                }
            ]
        }"#;
        let resp: ExclusiveNumberResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.phone_numbers.len(), 1);
        assert_eq!(resp.phone_numbers[0].phone_number, "0912345678");
        assert_eq!(resp.phone_numbers[0].create_date, "2025-01-01");
    }

    #[test]
    fn test_webhook_request_serialization() {
        let req = WebhookRequest {
            event_type: 3,
            url: "https://example.com/webhook".into(),
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["type"], 3);
        assert_eq!(json["url"], "https://example.com/webhook");
        // Rust field name must not appear in JSON.
        assert!(json.get("event_type").is_none());
    }

    #[test]
    fn test_webhook_response_deserialization() {
        let json = r#"{
            "webhooks": [
                {"type": 3, "url": "https://example.com/delivery"},
                {"type": 4, "url": "https://example.com/open"},
                {"type": 6, "url": "https://example.com/bounce"}
            ]
        }"#;
        let resp: WebhookResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.webhooks.len(), 3);
        assert_eq!(resp.webhooks[0].event_type, 3);
        assert_eq!(resp.webhooks[2].event_type, 6);
    }

    #[test]
    fn test_sms_webhook_request() {
        let req = SmsWebhookRequest {
            event_type: 6,
            url: "https://example.com/sms/bounce".into(),
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["type"], 6);
    }

    #[test]
    fn test_sms_webhook_response_deserialization() {
        let json = r#"{
            "webhooks": [
                {"type": 3, "url": "https://example.com/sms/delivery"},
                {"type": 6, "url": "https://example.com/sms/bounce"}
            ]
        }"#;
        let resp: SmsWebhookResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.webhooks.len(), 2);
        assert_eq!(resp.webhooks[0].event_type, 3);
        assert_eq!(resp.webhooks[1].event_type, 6);
    }

    #[test]
    fn test_dns_record_deserialization() {
        let json = r#"{
            "name": "_dmarc.mail.example.com",
            "value": "v=DMARC1; p=none",
            "record_type": 0,
            "valid": false
        }"#;
        let record: DnsRecord = serde_json::from_str(json).unwrap();
        assert_eq!(record.name, "_dmarc.mail.example.com");
        assert_eq!(record.record_type, 0);
        assert!(!record.valid);
    }

    #[test]
    fn test_domain_create_response() {
        let json = r#"[
            {"name": "_dmarc.example.com", "value": "v=DMARC1; p=none", "record_type": 0, "valid": false},
            {"name": "em._domainkey.example.com", "value": "CNAME...", "record_type": 1, "valid": true}
        ]"#;
        let records: DomainCreateResponse = serde_json::from_str(json).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].record_type, 0);
        assert!(!records[0].valid);
        assert!(records[1].valid);
    }

    #[test]
    fn test_webhook_delete_request() {
        let req = WebhookDeleteRequest { event_type: 5 };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["type"], 5);
        assert!(json.get("event_type").is_none());
    }

    #[test]
    fn test_sms_webhook_delete_request() {
        let req = SmsWebhookDeleteRequest { event_type: 3 };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["type"], 3);
    }

    #[test]
    fn test_sn_api_error_response() {
        let json = r#"{"errorCode": 40001, "message": "Invalid parameters"}"#;
        let err: SnApiErrorResponse = serde_json::from_str(json).unwrap();
        assert_eq!(err.error_code, Some(40001));
        assert_eq!(err.message, "Invalid parameters");
    }

    #[test]
    fn test_sn_api_error_response_missing_fields() {
        let json = r#"{"message": "Bad request"}"#;
        let err: SnApiErrorResponse = serde_json::from_str(json).unwrap();
        assert_eq!(err.error_code, None);
        assert_eq!(err.message, "Bad request");
    }

    #[test]
    fn test_email_events_params_default() {
        let params = EmailEventsParams::default();
        let json = serde_json::to_value(&params).unwrap();
        // All optional fields should be absent.
        assert!(json.as_object().unwrap().is_empty());
    }

    #[test]
    fn test_email_events_params_with_filters() {
        let params = EmailEventsParams {
            id: Some("MSG001".into()),
            status: Some("delivered".into()),
            page: Some(1),
            size: Some(20),
            ..Default::default()
        };
        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["id"], "MSG001");
        assert_eq!(json["status"], "delivered");
        assert_eq!(json["page"], 1);
        assert!(json.get("recipient").is_none());
    }

    #[test]
    fn test_sms_events_params_with_country_code() {
        let params = SmsEventsParams {
            country_code: Some("886".into()),
            status: Some("delivered".into()),
            ..Default::default()
        };
        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["country_code"], "886");
        assert_eq!(json["status"], "delivered");
    }
}
