//! EDM API client — 20 endpoint methods covering contacts, campaign, A/B test,
//! report, template, automation, and account.
//!
//! All methods return `Result<serde_json::Value, NlError>` so the executor can
//! pass the result directly to the formatter layer without caring about concrete
//! response types.

use std::path::Path;
use std::time::Instant;

use reqwest::multipart;
use serde_json::Value;

use crate::client::retry::with_retry;
use crate::client::{parse_api_response, ApiClient};
use crate::error::NlError;
use crate::types::edm::*;

/// Client for the NewsLeopard EDM API (`api.newsleopard.com`).
///
/// Borrows the shared `ApiClient` for HTTP transport, rate limiting, and
/// dry-run / verbose behaviour.
pub struct EdmClient<'a> {
    client: &'a ApiClient,
    api_key: String,
    base_url: String,
}

impl<'a> EdmClient<'a> {
    /// Creates a new `EdmClient` pointing at the production EDM API.
    pub fn new(client: &'a ApiClient, api_key: &str) -> Self {
        Self {
            client,
            api_key: api_key.to_string(),
            base_url: "https://api.newsleopard.com".to_string(),
        }
    }

    /// Creates a new `EdmClient` with a custom base URL (for wiremock tests).
    pub fn new_with_base_url(client: &'a ApiClient, api_key: &str, base_url: &str) -> Self {
        Self {
            client,
            api_key: api_key.to_string(),
            base_url: base_url.to_string(),
        }
    }

    // ── Account ──────────────────────────────────────────────────────────────

    /// GET /v1/balance — Retrieve account balance (email + SMS credits).
    pub async fn get_balance(&self) -> Result<Value, NlError> {
        let url = format!("{}/v1/balance", self.base_url);

        if let Some(err) = self.client.check_dry_run("GET", &url, &self.api_key, None) {
            return Err(err);
        }

        self.client.edm_limiter.until_ready().await;

        self.get(&url).await
    }

    // ── Contacts (6) ─────────────────────────────────────────────────────────

    /// POST /v1/contacts/lists/insert — Create a new contact group.
    pub async fn create_group(&self, name: &str) -> Result<Value, NlError> {
        let url = format!("{}/v1/contacts/lists/insert", self.base_url);
        let body = serde_json::json!({ "name": name });

        if let Some(err) = self
            .client
            .check_dry_run("POST", &url, &self.api_key, Some(&body))
        {
            return Err(err);
        }

        self.client.edm_limiter.until_ready().await;

        self.post_json(&url, &body).await
    }

    /// GET /v1/contacts/lists — List contact groups with optional pagination.
    pub async fn list_groups(
        &self,
        page: Option<u32>,
        size: Option<u32>,
    ) -> Result<Value, NlError> {
        let base_url = format!("{}/v1/contacts/lists", self.base_url);

        // Build URL with query params for dry-run check
        let mut url = base_url.clone();
        let mut param_strs = Vec::new();
        if let Some(p) = page {
            param_strs.push(format!("page={}", p));
        }
        if let Some(s) = size {
            param_strs.push(format!("size={}", s));
        }
        if !param_strs.is_empty() {
            url = format!("{}?{}", url, param_strs.join("&"));
        }

        if let Some(err) = self.client.check_dry_run("GET", &url, &self.api_key, None) {
            return Err(err);
        }

        self.client.edm_limiter.until_ready().await;

        self.get_with_query(&base_url, page, size).await
    }

    /// POST /v1/contacts/imports/{list_sn}/file — Import contacts from a file (multipart upload).
    ///
    /// This method does NOT use `with_retry` because multipart uploads are not
    /// idempotent and file bytes should not be re-read on transient failures.
    pub async fn import_file(
        &self,
        list_sn: &str,
        file_path: &Path,
        webhook_url: Option<&str>,
    ) -> Result<Value, NlError> {
        let url = format!("{}/v1/contacts/imports/{}/file", self.base_url, list_sn);

        if let Some(err) = self.client.check_dry_run(
            "POST",
            &url,
            &self.api_key,
            Some(&serde_json::json!({
                "file": file_path.display().to_string(),
                "webhookUrl": webhook_url,
            })),
        ) {
            return Err(err);
        }

        self.client.edm_limiter.until_ready().await;

        // Read file bytes for multipart upload.
        let file_name = file_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "upload".to_string());
        let file_bytes = tokio::fs::read(file_path).await.map_err(NlError::Io)?;

        let file_part = multipart::Part::bytes(file_bytes)
            .file_name(file_name)
            .mime_str("application/octet-stream")
            .map_err(|e| NlError::Validation(format!("Invalid MIME type: {}", e)))?;

        let mut form = multipart::Form::new().part("file", file_part);

        if let Some(wh) = webhook_url {
            form = form.text("webhookUrl", wh.to_string());
        }

        self.client.log_request("POST", &url);

        let start = Instant::now();
        let response = self
            .client
            .http
            .post(&url)
            .header("x-api-key", &self.api_key)
            .multipart(form)
            .send()
            .await?;

        let status = response.status().as_u16();
        let body_text = response.text().await?;
        let elapsed = start.elapsed().as_millis();

        self.client.log_response(status, elapsed, Some(&body_text));

        parse_api_response::<ApiErrorResponse>(status, &body_text)
    }

    /// POST /v1/contacts/imports/{list_sn}/text — Import contacts from CSV text.
    pub async fn import_text(
        &self,
        list_sn: &str,
        request: &ImportTextRequest,
    ) -> Result<Value, NlError> {
        let url = format!("{}/v1/contacts/imports/{}/text", self.base_url, list_sn);
        let body = serde_json::to_value(request)?;

        if let Some(err) = self
            .client
            .check_dry_run("POST", &url, &self.api_key, Some(&body))
        {
            return Err(err);
        }

        self.client.edm_limiter.until_ready().await;

        self.post_json(&url, &body).await
    }

    /// GET /v1/contacts/imports/result/{import_sn} — Check import status.
    pub async fn import_status(&self, import_sn: &str) -> Result<Value, NlError> {
        let url = format!("{}/v1/contacts/imports/result/{}", self.base_url, import_sn);

        if let Some(err) = self.client.check_dry_run("GET", &url, &self.api_key, None) {
            return Err(err);
        }

        self.client.edm_limiter.until_ready().await;

        self.get(&url).await
    }

    /// DELETE /v1/contacts/{list_sn} — Remove contacts from a group.
    pub async fn remove_contacts(
        &self,
        list_sn: &str,
        request: &RemoveContactsRequest,
    ) -> Result<Value, NlError> {
        let url = format!("{}/v1/contacts/{}", self.base_url, list_sn);
        let body = serde_json::to_value(request)?;

        if let Some(err) = self
            .client
            .check_dry_run("DELETE", &url, &self.api_key, Some(&body))
        {
            return Err(err);
        }

        self.client.edm_limiter.until_ready().await;

        self.delete_json(&url, &body).await
    }

    // ── Campaign (5) ─────────────────────────────────────────────────────────

    /// POST /v1/campaign/normal/submit — Submit a campaign for sending.
    pub async fn submit_campaign(&self, request: &CampaignSubmitRequest) -> Result<Value, NlError> {
        let url = format!("{}/v1/campaign/normal/submit", self.base_url);
        let body = serde_json::to_value(request)?;

        if let Some(err) = self
            .client
            .check_dry_run("POST", &url, &self.api_key, Some(&body))
        {
            return Err(err);
        }

        self.client.edm_limiter.until_ready().await;

        self.post_json(&url, &body).await
    }

    /// POST /v1/campaign/normal/once — Submit a single-shot campaign with inline contacts.
    pub async fn submit_campaign_once(
        &self,
        request: &CampaignOnceRequest,
    ) -> Result<Value, NlError> {
        let url = format!("{}/v1/campaign/normal/once", self.base_url);
        let body = serde_json::to_value(request)?;

        if let Some(err) = self
            .client
            .check_dry_run("POST", &url, &self.api_key, Some(&body))
        {
            return Err(err);
        }

        self.client.edm_limiter.until_ready().await;

        self.post_json(&url, &body).await
    }

    /// DELETE /v1/campaign/normal — Delete one or more campaigns.
    pub async fn delete_campaigns(
        &self,
        request: &CampaignDeleteRequest,
    ) -> Result<Value, NlError> {
        let url = format!("{}/v1/campaign/normal", self.base_url);
        let body = serde_json::to_value(request)?;

        if let Some(err) = self
            .client
            .check_dry_run("DELETE", &url, &self.api_key, Some(&body))
        {
            return Err(err);
        }

        self.client.edm_limiter.until_ready().await;

        self.delete_json(&url, &body).await
    }

    /// PATCH /v1/campaign/normal/{campaign_sn} — Pause a running campaign.
    pub async fn pause_campaign(&self, campaign_sn: &str) -> Result<Value, NlError> {
        let url = format!("{}/v1/campaign/normal/{}", self.base_url, campaign_sn);

        if let Some(err) = self
            .client
            .check_dry_run("PATCH", &url, &self.api_key, None)
        {
            return Err(err);
        }

        self.client.edm_limiter.until_ready().await;

        self.patch_empty(&url).await
    }

    /// GET /v1/campaign/normal/{campaign_sn} — Get campaign status.
    pub async fn campaign_status(&self, campaign_sn: &str) -> Result<Value, NlError> {
        let url = format!("{}/v1/campaign/normal/{}", self.base_url, campaign_sn);

        if let Some(err) = self.client.check_dry_run("GET", &url, &self.api_key, None) {
            return Err(err);
        }

        self.client.edm_limiter.until_ready().await;

        self.get(&url).await
    }

    // ── A/B Test (2) ─────────────────────────────────────────────────────────

    /// POST /v1/campaign/testing/submit — Submit an A/B test campaign.
    pub async fn submit_ab_test(&self, request: &AbTestSubmitRequest) -> Result<Value, NlError> {
        let url = format!("{}/v1/campaign/testing/submit", self.base_url);
        let body = serde_json::to_value(request)?;

        if let Some(err) = self
            .client
            .check_dry_run("POST", &url, &self.api_key, Some(&body))
        {
            return Err(err);
        }

        self.client.edm_limiter.until_ready().await;

        self.post_json(&url, &body).await
    }

    /// POST /v1/campaign/testing/once — Submit a single-shot A/B test.
    pub async fn submit_ab_test_once(&self, request: &AbTestOnceRequest) -> Result<Value, NlError> {
        let url = format!("{}/v1/campaign/testing/once", self.base_url);
        let body = serde_json::to_value(request)?;

        if let Some(err) = self
            .client
            .check_dry_run("POST", &url, &self.api_key, Some(&body))
        {
            return Err(err);
        }

        self.client.edm_limiter.until_ready().await;

        self.post_json(&url, &body).await
    }

    // ── Report (4) — report_export uses the stricter report_limiter ──────────

    /// GET /v1/report/campaigns?startDate=&endDate= — List campaigns by date range.
    pub async fn report_list(&self, start_date: &str, end_date: &str) -> Result<Value, NlError> {
        let base_url = format!("{}/v1/report/campaigns", self.base_url);

        // Build URL with query params for dry-run check
        let url = format!("{}?startDate={}&endDate={}", base_url, start_date, end_date);

        if let Some(err) = self.client.check_dry_run("GET", &url, &self.api_key, None) {
            return Err(err);
        }

        self.client.edm_limiter.until_ready().await;

        self.get_with_report_params(&base_url, start_date, end_date)
            .await
    }

    /// POST /v1/report/campaigns/metrics — Fetch performance metrics for campaigns.
    pub async fn report_metrics(&self, sns: &[String]) -> Result<Value, NlError> {
        let url = format!("{}/v1/report/campaigns/metrics", self.base_url);
        let body = serde_json::to_value(ReportMetricsRequest {
            campaign_sns: sns.to_vec(),
        })?;

        if let Some(err) = self
            .client
            .check_dry_run("POST", &url, &self.api_key, Some(&body))
        {
            return Err(err);
        }

        self.client.edm_limiter.until_ready().await;

        self.post_json(&url, &body).await
    }

    /// POST /v1/report/{campaign_sn}/export — Trigger report export.
    ///
    /// Uses the **report_limiter** (1 req/10s) instead of the general EDM limiter.
    pub async fn report_export(&self, campaign_sn: &str) -> Result<Value, NlError> {
        let url = format!("{}/v1/report/{}/export", self.base_url, campaign_sn);

        if let Some(err) = self.client.check_dry_run("POST", &url, &self.api_key, None) {
            return Err(err);
        }

        // Report export uses the stricter rate limiter.
        self.client.report_limiter.until_ready().await;

        self.post_empty(&url).await
    }

    /// GET /v1/report/{campaign_sn}/link — Get the download link for an exported report.
    pub async fn report_download_link(&self, campaign_sn: &str) -> Result<Value, NlError> {
        let url = format!("{}/v1/report/{}/link", self.base_url, campaign_sn);

        if let Some(err) = self.client.check_dry_run("GET", &url, &self.api_key, None) {
            return Err(err);
        }

        self.client.edm_limiter.until_ready().await;

        self.get(&url).await
    }

    // ── Template (2) ─────────────────────────────────────────────────────────

    /// GET /v1/templates — List all templates.
    pub async fn list_templates(&self) -> Result<Value, NlError> {
        let url = format!("{}/v1/templates", self.base_url);

        if let Some(err) = self.client.check_dry_run("GET", &url, &self.api_key, None) {
            return Err(err);
        }

        self.client.edm_limiter.until_ready().await;

        self.get(&url).await
    }

    /// GET /v1/templates/{id} — Get a template by ID.
    pub async fn get_template(&self, id: &str) -> Result<Value, NlError> {
        let url = format!("{}/v1/templates/{}", self.base_url, id);

        if let Some(err) = self.client.check_dry_run("GET", &url, &self.api_key, None) {
            return Err(err);
        }

        self.client.edm_limiter.until_ready().await;

        self.get(&url).await
    }

    // ── Automation (1) ───────────────────────────────────────────────────────

    /// POST /v1/automation/event — Trigger an automation event.
    pub async fn trigger_automation(
        &self,
        request: &AutomationTriggerRequest,
    ) -> Result<Value, NlError> {
        let url = format!("{}/v1/automation/event", self.base_url);
        let body = serde_json::to_value(request)?;

        if let Some(err) = self
            .client
            .check_dry_run("POST", &url, &self.api_key, Some(&body))
        {
            return Err(err);
        }

        self.client.edm_limiter.until_ready().await;

        self.post_json(&url, &body).await
    }

    // ── Internal HTTP helpers ────────────────────────────────────────────────

    /// Send a GET request with automatic retry.
    async fn get(&self, url: &str) -> Result<Value, NlError> {
        let url = url.to_string();
        let api_key = self.api_key.clone();
        let client = self.client;

        with_retry(|| {
            let url = url.clone();
            let api_key = api_key.clone();
            async move {
                client.log_request("GET", &url);
                let start = Instant::now();

                let response = client
                    .http
                    .get(&url)
                    .header("x-api-key", &api_key)
                    .send()
                    .await?;

                let status = response.status().as_u16();
                let body_text = response.text().await?;
                let elapsed = start.elapsed().as_millis();

                client.log_response(status, elapsed, Some(&body_text));

                parse_api_response::<ApiErrorResponse>(status, &body_text)
            }
        })
        .await
    }

    /// Send a GET request with query parameters using proper URL encoding.
    async fn get_with_query(
        &self,
        base_url: &str,
        page: Option<u32>,
        size: Option<u32>,
    ) -> Result<Value, NlError> {
        let api_key = self.api_key.clone();
        let client = self.client;
        let base_url = base_url.to_string();

        with_retry(|| {
            let base_url = base_url.clone();
            let api_key = api_key.clone();
            async move {
                let mut req = client.http.get(&base_url).header("x-api-key", &api_key);
                if let Some(p) = page {
                    req = req.query(&[("page", p.to_string())]);
                }
                if let Some(s) = size {
                    req = req.query(&[("size", s.to_string())]);
                }

                let url_str = format!("{}?page={:?}&size={:?}", base_url, page, size);
                client.log_request("GET", &url_str);
                let start = Instant::now();

                let response = req.send().await?;

                let status = response.status().as_u16();
                let body_text = response.text().await?;
                let elapsed = start.elapsed().as_millis();

                client.log_response(status, elapsed, Some(&body_text));

                parse_api_response::<ApiErrorResponse>(status, &body_text)
            }
        })
        .await
    }

    /// Send a GET request with date range query parameters.
    async fn get_with_report_params(
        &self,
        base_url: &str,
        start_date: &str,
        end_date: &str,
    ) -> Result<Value, NlError> {
        let api_key = self.api_key.clone();
        let client = self.client;
        let base_url = base_url.to_string();
        let start_date = start_date.to_string();
        let end_date = end_date.to_string();

        with_retry(|| {
            let base_url = base_url.clone();
            let api_key = api_key.clone();
            let start_date = start_date.clone();
            let end_date = end_date.clone();
            async move {
                let req = client
                    .http
                    .get(&base_url)
                    .header("x-api-key", &api_key)
                    .query(&[("startDate", &start_date), ("endDate", &end_date)]);

                let url_str = format!("{}?startDate={}&endDate={}", base_url, start_date, end_date);
                client.log_request("GET", &url_str);
                let start = Instant::now();

                let response = req.send().await?;

                let status = response.status().as_u16();
                let body_text = response.text().await?;
                let elapsed = start.elapsed().as_millis();

                client.log_response(status, elapsed, Some(&body_text));

                parse_api_response::<ApiErrorResponse>(status, &body_text)
            }
        })
        .await
    }

    /// Send a POST request with a JSON body and automatic retry.
    async fn post_json(&self, url: &str, body: &Value) -> Result<Value, NlError> {
        let url = url.to_string();
        let api_key = self.api_key.clone();
        let body = body.clone();
        let client = self.client;

        with_retry(|| {
            let url = url.clone();
            let api_key = api_key.clone();
            let body = body.clone();
            async move {
                client.log_request("POST", &url);
                let start = Instant::now();

                let response = client
                    .http
                    .post(&url)
                    .header("x-api-key", &api_key)
                    .json(&body)
                    .send()
                    .await?;

                let status = response.status().as_u16();
                let body_text = response.text().await?;
                let elapsed = start.elapsed().as_millis();

                client.log_response(status, elapsed, Some(&body_text));

                parse_api_response::<ApiErrorResponse>(status, &body_text)
            }
        })
        .await
    }

    /// Send a POST request with no body and automatic retry.
    async fn post_empty(&self, url: &str) -> Result<Value, NlError> {
        let url = url.to_string();
        let api_key = self.api_key.clone();
        let client = self.client;

        with_retry(|| {
            let url = url.clone();
            let api_key = api_key.clone();
            async move {
                client.log_request("POST", &url);
                let start = Instant::now();

                let response = client
                    .http
                    .post(&url)
                    .header("x-api-key", &api_key)
                    .send()
                    .await?;

                let status = response.status().as_u16();
                let body_text = response.text().await?;
                let elapsed = start.elapsed().as_millis();

                client.log_response(status, elapsed, Some(&body_text));

                parse_api_response::<ApiErrorResponse>(status, &body_text)
            }
        })
        .await
    }

    /// Send a PATCH request with no body and automatic retry.
    async fn patch_empty(&self, url: &str) -> Result<Value, NlError> {
        let url = url.to_string();
        let api_key = self.api_key.clone();
        let client = self.client;

        with_retry(|| {
            let url = url.clone();
            let api_key = api_key.clone();
            async move {
                client.log_request("PATCH", &url);
                let start = Instant::now();

                let response = client
                    .http
                    .patch(&url)
                    .header("x-api-key", &api_key)
                    .send()
                    .await?;

                let status = response.status().as_u16();
                let body_text = response.text().await?;
                let elapsed = start.elapsed().as_millis();

                client.log_response(status, elapsed, Some(&body_text));

                parse_api_response::<ApiErrorResponse>(status, &body_text)
            }
        })
        .await
    }

    /// Send a DELETE request with a JSON body and automatic retry.
    async fn delete_json(&self, url: &str, body: &Value) -> Result<Value, NlError> {
        let url = url.to_string();
        let api_key = self.api_key.clone();
        let body = body.clone();
        let client = self.client;

        with_retry(|| {
            let url = url.clone();
            let api_key = api_key.clone();
            let body = body.clone();
            async move {
                client.log_request("DELETE", &url);
                let start = Instant::now();

                let response = client
                    .http
                    .delete(&url)
                    .header("x-api-key", &api_key)
                    .json(&body)
                    .send()
                    .await?;

                let status = response.status().as_u16();
                let body_text = response.text().await?;
                let elapsed = start.elapsed().as_millis();

                client.log_response(status, elapsed, Some(&body_text));

                parse_api_response::<ApiErrorResponse>(status, &body_text)
            }
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dry_run_get_balance() {
        let api_client = ApiClient::new(true, 0);
        let edm = EdmClient::new(&api_client, "test-key-abc");
        let result = edm.get_balance().await;
        assert!(result.is_err());
        match result.unwrap_err() {
            NlError::DryRun(info) => {
                assert_eq!(info.method, "GET");
                assert!(info.url.ends_with("/v1/balance"));
            }
            _ => panic!("Expected DryRun error"),
        }
    }

    #[tokio::test]
    async fn test_dry_run_create_group() {
        let api_client = ApiClient::new(true, 0);
        let edm = EdmClient::new(&api_client, "test-key");
        let result = edm.create_group("VIP Customers").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            NlError::DryRun(info) => {
                assert_eq!(info.method, "POST");
                assert!(info.url.ends_with("/v1/contacts/lists/insert"));
                let body = info.body.unwrap();
                assert_eq!(body["name"], "VIP Customers");
            }
            _ => panic!("Expected DryRun error"),
        }
    }

    #[tokio::test]
    async fn test_dry_run_list_groups_with_pagination() {
        let api_client = ApiClient::new(true, 0);
        let edm = EdmClient::new(&api_client, "test-key");
        let result = edm.list_groups(Some(2), Some(10)).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            NlError::DryRun(info) => {
                assert!(info.url.contains("page=2"));
                assert!(info.url.contains("size=10"));
            }
            _ => panic!("Expected DryRun error"),
        }
    }

    #[tokio::test]
    async fn test_dry_run_submit_campaign() {
        let api_client = ApiClient::new(true, 0);
        let edm = EdmClient::new(&api_client, "test-key-xyz");
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
        assert!(result.is_err());
        match result.unwrap_err() {
            NlError::DryRun(info) => {
                assert_eq!(info.method, "POST");
                assert!(info.url.ends_with("/v1/campaign/normal/submit"));
                assert!(info.body.is_some());
            }
            _ => panic!("Expected DryRun error"),
        }
    }

    #[tokio::test]
    async fn test_dry_run_delete_campaigns() {
        let api_client = ApiClient::new(true, 0);
        let edm = EdmClient::new(&api_client, "test-key");
        let request = CampaignDeleteRequest {
            sns: vec!["SN1".into()],
        };
        let result = edm.delete_campaigns(&request).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            NlError::DryRun(info) => {
                assert_eq!(info.method, "DELETE");
                assert!(info.url.ends_with("/v1/campaign/normal"));
                assert!(info.body.is_some());
            }
            _ => panic!("Expected DryRun error"),
        }
    }

    #[tokio::test]
    async fn test_dry_run_pause_campaign() {
        let api_client = ApiClient::new(true, 0);
        let edm = EdmClient::new(&api_client, "test-key");
        let result = edm.pause_campaign("C001").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            NlError::DryRun(info) => {
                assert_eq!(info.method, "PATCH");
                assert!(info.url.ends_with("/v1/campaign/normal/C001"));
            }
            _ => panic!("Expected DryRun error"),
        }
    }

    #[tokio::test]
    async fn test_dry_run_report_export() {
        let api_client = ApiClient::new(true, 0);
        let edm = EdmClient::new(&api_client, "test-key");
        let result = edm.report_export("C001").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            NlError::DryRun(info) => {
                assert_eq!(info.method, "POST");
                assert!(info.url.contains("/v1/report/C001/export"));
            }
            _ => panic!("Expected DryRun error"),
        }
    }

    #[tokio::test]
    async fn test_dry_run_report_metrics() {
        let api_client = ApiClient::new(true, 0);
        let edm = EdmClient::new(&api_client, "test-key");
        let result = edm.report_metrics(&["C001".into(), "C002".into()]).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            NlError::DryRun(info) => {
                let body = info.body.unwrap();
                assert_eq!(body["campaignSns"][0], "C001");
                assert_eq!(body["campaignSns"][1], "C002");
            }
            _ => panic!("Expected DryRun error"),
        }
    }

    #[tokio::test]
    async fn test_dry_run_trigger_automation() {
        let api_client = ApiClient::new(true, 0);
        let edm = EdmClient::new(&api_client, "test-key");
        let request = AutomationTriggerRequest {
            event: "order_complete".into(),
            recipients: vec![],
        };
        let result = edm.trigger_automation(&request).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            NlError::DryRun(info) => {
                assert_eq!(info.method, "POST");
                assert!(info.url.ends_with("/v1/automation/event"));
                let body = info.body.unwrap();
                assert_eq!(body["event"], "order_complete");
            }
            _ => panic!("Expected DryRun error"),
        }
    }
}
