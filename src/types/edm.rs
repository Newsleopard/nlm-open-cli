//! EDM API request/response types.
//!
//! Covers all 20 EDM endpoints: contacts (6), campaign (5), A/B test (2),
//! report (4), template (2), automation (1), account/balance (1).
//!
//! Field names use `#[serde(rename = "camelCase")]` to match the EDM API's
//! JSON contract while keeping Rust-idiomatic snake_case in code.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── Account ────────────────────────────────────────────────────────────────

/// GET /v1/balance
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BalanceResponse {
    pub email: u64,
    pub sms: u64,
}

// ─── Contacts ───────────────────────────────────────────────────────────────

/// POST /v1/contacts/lists/insert — request body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGroupRequest {
    pub name: String,
}

/// POST /v1/contacts/lists/insert — response body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGroupResponse {
    pub sn: String,
}

/// A single contact group returned by list-groups.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContactGroup {
    pub sn: String,
    pub name: String,
    pub subscribed_cnt: u64,
    pub exclude_cnt: u64,
    pub opened_rate: f64,
    pub clicked_rate: f64,
    pub status: String,
    #[serde(rename = "type")]
    pub group_type: u8,
}

/// Pagination metadata included in list responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageInfo {
    pub total: u64,
    pub page: u32,
    pub size: u32,
}

/// GET /v1/contacts/lists — response body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListGroupsResponse {
    pub groups: Vec<ContactGroup>,
    #[serde(rename = "pageInfo")]
    pub page_info: PageInfo,
}

/// POST /v1/contacts/imports/{list_sn}/file — response body
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportFileResponse {
    pub import_sn: String,
}

/// POST /v1/contacts/imports/{list_sn}/text — request body
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportTextRequest {
    pub csv_text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook_url: Option<String>,
}

/// GET /v1/contacts/imports/result/{import_sn} — response body
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportStatusResponse {
    pub import_sn: String,
    pub status: String,
    pub file_cnt: u64,
    pub insert_cnt: u64,
    pub duplicate_cnt: u64,
    pub err_cnt: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_download_link: Option<String>,
}

/// DELETE /v1/contacts/{list_sn} — request body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveContactsRequest {
    pub field: String,
    pub operator: String,
    pub value: String,
}

// ─── Campaign ───────────────────────────────────────────────────────────────

/// POST /v1/campaign/normal/submit — top-level request body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignSubmitRequest {
    pub form: CampaignForm,
    pub content: CampaignContent,
    pub config: CampaignConfig,
}

/// Campaign form: name and list selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignForm {
    pub name: String,
    #[serde(rename = "selectedLists")]
    pub selected_lists: Vec<String>,
    #[serde(
        rename = "excludeLists",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub exclude_lists: Vec<String>,
}

/// Campaign content: subject, sender, HTML body, footer language, preheader.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignContent {
    /// Max 150 characters.
    pub subject: String,
    /// Max 50 characters.
    #[serde(rename = "fromName")]
    pub from_name: String,
    #[serde(rename = "fromAddress")]
    pub from_address: String,
    #[serde(rename = "htmlContent")]
    pub html_content: String,
    /// 0 = English, 1 = Chinese.
    #[serde(rename = "footerLang")]
    pub footer_lang: u8,
    /// Max 60 characters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preheader: Option<String>,
}

/// Campaign scheduling and tracking configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignConfig {
    pub schedule: ScheduleConfig,
    pub ga: GaConfig,
}

/// Schedule settings: immediate or scheduled send.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleConfig {
    /// 0 = immediate, 1 = scheduled.
    #[serde(rename = "type")]
    pub schedule_type: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<u8>,
    #[serde(rename = "scheduleDate", skip_serializing_if = "Option::is_none")]
    pub schedule_date: Option<String>,
}

/// Google Analytics tracking configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaConfig {
    pub enable: bool,
    #[serde(rename = "ecommerceEnable")]
    pub ecommerce_enable: bool,
    #[serde(rename = "utmCampaign", skip_serializing_if = "Option::is_none")]
    pub utm_campaign: Option<String>,
    #[serde(rename = "utmContent", skip_serializing_if = "Option::is_none")]
    pub utm_content: Option<String>,
}

/// POST /v1/campaign/normal/submit — response body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignSubmitResponse {
    pub sn: String,
}

/// POST /v1/campaign/normal/once — request body.
///
/// Similar to `CampaignSubmitRequest` but uses inline contacts
/// (`contacts_file_content`) instead of `selected_lists`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignOnceRequest {
    pub form: CampaignOnceForm,
    pub content: CampaignContent,
    pub config: CampaignConfig,
}

/// Campaign once form: name and inline CSV content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignOnceForm {
    pub name: String,
    #[serde(rename = "contactsFileContent")]
    pub contacts_file_content: String,
    #[serde(
        rename = "excludeLists",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub exclude_lists: Vec<String>,
}

/// GET /v1/campaign/normal/{campaign_sn} — response body
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignStatusResponse {
    pub sn: String,
    pub name: String,
    pub status: String,
    pub send_time_type: u8,
    #[serde(rename = "type")]
    pub campaign_type: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sent_begin_date: Option<String>,
}

/// DELETE /v1/campaign/normal — request body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignDeleteRequest {
    pub sns: Vec<String>,
}

/// DELETE /v1/campaign/normal — response body
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignDeleteResponse {
    pub success: Vec<String>,
    pub sending_campaign: Vec<String>,
    pub bad_campaigns: Vec<String>,
}

// ─── A/B Test ───────────────────────────────────────────────────────────────

/// POST /v1/campaign/testing/submit — top-level request body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbTestSubmitRequest {
    pub form: CampaignForm,
    pub content: AbTestContent,
    pub config: CampaignConfig,
}

/// POST /v1/campaign/testing/once — top-level request body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbTestOnceRequest {
    pub form: CampaignOnceForm,
    pub content: AbTestContent,
    pub config: CampaignConfig,
}

/// A/B test content with variant fields.
///
/// Which fields are required depends on `testing_on`:
///   1 = subject (subject_a/b required)
///   2 = sender (from_name_a/b, from_address_a/b required)
///   3 = content (html_content_a/b required)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbTestContent {
    pub testing_on: u8,
    pub testing: AbTestConfig,

    // Shared fields — used as the "winner" version or shared baseline.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preheader: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub footer_lang: Option<u8>,

    // Variant A fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject_a: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_name_a: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_address_a: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_content_a: Option<String>,

    // Variant B fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject_b: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_name_b: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_address_b: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_content_b: Option<String>,
}

/// A/B test configuration: proportion, duration, time unit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbTestConfig {
    /// Percentage of recipients to include in the test (0-100).
    pub proportion: u8,
    /// Duration of the testing period.
    pub time: u32,
    /// 0 = hours, 1 = days.
    pub unit: u8,
}

// ─── Report ─────────────────────────────────────────────────────────────────

/// GET /v1/report/campaigns — query parameters (not a request body).
///
/// Used to build query string parameters; not serialized as JSON body.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportListParams {
    pub start_date: String,
    pub end_date: String,
}

/// Campaign performance metrics.
///
/// Returned by POST /v1/report/campaigns/metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignMetrics {
    pub campaign_sn: String,
    pub name: String,
    pub channel: String,
    pub subject: String,
    pub recipient_cnt: u64,
    pub delivered: u64,
    pub bounced: u64,
    pub opened: u64,
    pub clicked: u64,
    pub distinct_click_cnt: u64,
    pub complained: u64,
    pub unsubscribed: u64,
}

/// POST /v1/report/{campaign_sn}/export — response body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportExportResponse {
    pub status: String,
}

/// GET /v1/report/{campaign_sn}/link — response body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportDownloadLinkResponse {
    pub link: Option<String>,
}

// ─── Template ───────────────────────────────────────────────────────────────

/// A single template in the list response (GET /v1/templates).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateListItem {
    pub id: String,
    pub name: String,
}

/// Full template content (GET /v1/templates/{id}).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateContent {
    pub id: String,
    pub name: String,
    pub html: String,
}

// ─── Automation ─────────────────────────────────────────────────────────────

/// POST /v1/automation/event — request body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationTriggerRequest {
    pub event: String,
    pub recipients: Vec<AutomationRecipient>,
}

/// A single recipient in an automation trigger.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationRecipient {
    pub name: String,
    pub address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<HashMap<String, String>>,
}

// ─── Error Parsing ──────────────────────────────────────────────────────────

/// Standard error body returned by the EDM API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiErrorResponse {
    pub error_code: Option<u32>,
    pub message: String,
}

impl<'a> From<&'a ApiErrorResponse> for (Option<i64>, String) {
    fn from(err: &'a ApiErrorResponse) -> Self {
        (err.error_code.map(|c| c as i64), err.message.clone())
    }
}

// ─── Report Metrics Request ─────────────────────────────────────────────────

/// POST /v1/report/campaigns/metrics — request body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetricsRequest {
    #[serde(rename = "campaignSns")]
    pub campaign_sns: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_balance_response_roundtrip() {
        let json = r#"{"email":10000,"sms":500}"#;
        let balance: BalanceResponse = serde_json::from_str(json).unwrap();
        assert_eq!(balance.email, 10000);
        assert_eq!(balance.sms, 500);
        assert_eq!(serde_json::to_string(&balance).unwrap(), json);
    }

    #[test]
    fn test_contact_group_camel_case() {
        let json = r#"{
            "sn": "G001",
            "name": "VIP",
            "subscribedCnt": 1500,
            "excludeCnt": 42,
            "openedRate": 0.35,
            "clickedRate": 0.12,
            "status": "GENERAL",
            "type": 0
        }"#;
        let group: ContactGroup = serde_json::from_str(json).unwrap();
        assert_eq!(group.sn, "G001");
        assert_eq!(group.subscribed_cnt, 1500);
        assert_eq!(group.group_type, 0);

        // Re-serialize and verify camelCase field names.
        let re = serde_json::to_value(&group).unwrap();
        assert!(re.get("subscribedCnt").is_some());
        assert!(re.get("type").is_some());
        // Rust field names must not leak into JSON.
        assert!(re.get("subscribed_cnt").is_none());
        assert!(re.get("group_type").is_none());
    }

    #[test]
    fn test_campaign_form_exclude_lists_skipped_when_empty() {
        let form = CampaignForm {
            name: "Test".into(),
            selected_lists: vec!["SN1".into()],
            exclude_lists: vec![],
        };
        let json = serde_json::to_value(&form).unwrap();
        assert!(json.get("excludeLists").is_none());
    }

    #[test]
    fn test_campaign_form_exclude_lists_present_when_set() {
        let form = CampaignForm {
            name: "Test".into(),
            selected_lists: vec!["SN1".into()],
            exclude_lists: vec!["EX1".into()],
        };
        let json = serde_json::to_value(&form).unwrap();
        assert!(json.get("excludeLists").is_some());
    }

    #[test]
    fn test_schedule_config_immediate() {
        let config = ScheduleConfig {
            schedule_type: 0,
            timezone: None,
            schedule_date: None,
        };
        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json.get("type").unwrap(), 0);
        assert!(json.get("scheduleDate").is_none());
    }

    #[test]
    fn test_schedule_config_scheduled() {
        let config = ScheduleConfig {
            schedule_type: 1,
            timezone: Some(8),
            schedule_date: Some("2025-01-15T10:00:00.000Z".into()),
        };
        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json.get("type").unwrap(), 1);
        assert_eq!(
            json.get("scheduleDate").unwrap(),
            "2025-01-15T10:00:00.000Z"
        );
    }

    #[test]
    fn test_ga_config_optional_fields() {
        let config = GaConfig {
            enable: true,
            ecommerce_enable: false,
            utm_campaign: Some("weekly".into()),
            utm_content: None,
        };
        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json.get("enable").unwrap(), true);
        assert_eq!(json.get("ecommerceEnable").unwrap(), false);
        assert_eq!(json.get("utmCampaign").unwrap(), "weekly");
        assert!(json.get("utmContent").is_none());
    }

    #[test]
    fn test_campaign_status_response_deserialization() {
        let json = r#"{
            "sn": "C001",
            "name": "Newsletter",
            "status": "SENT",
            "sendTimeType": 0,
            "type": 1,
            "sentBeginDate": "2025-01-15T10:00:00Z"
        }"#;
        let status: CampaignStatusResponse = serde_json::from_str(json).unwrap();
        assert_eq!(status.sn, "C001");
        assert_eq!(status.send_time_type, 0);
        assert_eq!(status.campaign_type, 1);
        assert_eq!(
            status.sent_begin_date.as_deref(),
            Some("2025-01-15T10:00:00Z")
        );
    }

    #[test]
    fn test_campaign_delete_response() {
        let json = r#"{
            "success": ["SN1", "SN2"],
            "sendingCampaign": ["SN3"],
            "badCampaigns": []
        }"#;
        let resp: CampaignDeleteResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.success.len(), 2);
        assert_eq!(resp.sending_campaign.len(), 1);
        assert!(resp.bad_campaigns.is_empty());
    }

    #[test]
    fn test_ab_test_content_subject_testing() {
        let content = AbTestContent {
            testing_on: 1,
            testing: AbTestConfig {
                proportion: 20,
                time: 4,
                unit: 0,
            },
            subject: None,
            from_name: Some("Brand".into()),
            from_address: Some("test@example.com".into()),
            html_content: Some("<p>Hi</p>".into()),
            preheader: None,
            footer_lang: Some(1),
            subject_a: Some("Subject A".into()),
            subject_b: Some("Subject B".into()),
            from_name_a: None,
            from_name_b: None,
            from_address_a: None,
            from_address_b: None,
            html_content_a: None,
            html_content_b: None,
        };
        let json = serde_json::to_value(&content).unwrap();
        assert_eq!(json.get("testingOn").unwrap(), 1);
        assert!(json.get("subjectA").is_some());
        assert!(json.get("subjectB").is_some());
        // Non-applicable variant fields should be absent.
        assert!(json.get("fromNameA").is_none());
    }

    #[test]
    fn test_import_text_request() {
        let req = ImportTextRequest {
            csv_text: "email\ntest@example.com".into(),
            webhook_url: None,
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json.get("csvText").unwrap(), "email\ntest@example.com");
        assert!(json.get("webhookUrl").is_none());
    }

    #[test]
    fn test_import_status_response() {
        let json = r#"{
            "importSn": "IMP001",
            "status": "COMPLETED",
            "fileCnt": 1000,
            "insertCnt": 950,
            "duplicateCnt": 40,
            "errCnt": 10,
            "errorDownloadLink": "https://example.com/errors.csv"
        }"#;
        let resp: ImportStatusResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.import_sn, "IMP001");
        assert_eq!(resp.insert_cnt, 950);
        assert!(resp.error_download_link.is_some());
    }

    #[test]
    fn test_campaign_metrics_deserialization() {
        let json = r#"{
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
        }"#;
        let metrics: CampaignMetrics = serde_json::from_str(json).unwrap();
        assert_eq!(metrics.campaign_sn, "C001");
        assert_eq!(metrics.recipient_cnt, 5000);
        assert_eq!(metrics.distinct_click_cnt, 320);
    }

    #[test]
    fn test_api_error_response() {
        let json = r#"{"errorCode": 40012, "message": "Insufficient balance"}"#;
        let err: ApiErrorResponse = serde_json::from_str(json).unwrap();
        assert_eq!(err.error_code, Some(40012));
        assert_eq!(err.message, "Insufficient balance");
    }

    #[test]
    fn test_automation_trigger_request() {
        let req = AutomationTriggerRequest {
            event: "order_complete".into(),
            recipients: vec![AutomationRecipient {
                name: "John".into(),
                address: "john@example.com".into(),
                variables: Some(HashMap::from([("order_id".into(), "12345".into())])),
            }],
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["event"], "order_complete");
        assert_eq!(json["recipients"][0]["variables"]["order_id"], "12345");
    }

    #[test]
    fn test_full_campaign_submit_request() {
        let req = CampaignSubmitRequest {
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
        let json = serde_json::to_string(&req).unwrap();
        // Verify JSON round-trips without losing data.
        let parsed: CampaignSubmitRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.form.name, "Weekly Newsletter");
        assert_eq!(parsed.content.footer_lang, 1);
    }
}
