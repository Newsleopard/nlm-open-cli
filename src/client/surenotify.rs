//! Surenotify API client — 14 endpoint methods covering email, SMS,
//! email webhook, SMS webhook, and domain verification.
//!
//! All methods return `Result<serde_json::Value, NlError>` so the executor can
//! pass the result directly to the formatter layer.
//!
//! Surenotify has **no rate limiter** per the spec (unlike the EDM API).

use std::time::Instant;

use serde_json::Value;

use crate::client::{parse_api_response, retry::with_retry, ApiClient};
use crate::error::NlError;
use crate::types::surenotify::*;

/// Client for the Surenotify API (`mail.surenotifyapi.com`).
///
/// Borrows the shared `ApiClient` for HTTP transport and dry-run / verbose
/// behaviour. Surenotify does not use a rate limiter.
pub struct SurenotifyClient<'a> {
    client: &'a ApiClient,
    api_key: String,
    base_url: String,
}

impl<'a> SurenotifyClient<'a> {
    /// Creates a new `SurenotifyClient` pointing at the production Surenotify API.
    pub fn new(client: &'a ApiClient, api_key: &str) -> Self {
        Self {
            client,
            api_key: api_key.to_string(),
            base_url: "https://mail.surenotifyapi.com".to_string(),
        }
    }

    /// Creates a new `SurenotifyClient` with a custom base URL (for wiremock tests).
    pub fn new_with_base_url(client: &'a ApiClient, api_key: &str, base_url: &str) -> Self {
        Self {
            client,
            api_key: api_key.to_string(),
            base_url: base_url.to_string(),
        }
    }

    // ── Email (2) ────────────────────────────────────────────────────────────

    /// POST /v1/messages — Send a transactional email.
    pub async fn send_email(&self, request: &EmailSendRequest) -> Result<Value, NlError> {
        let url = format!("{}/v1/messages", self.base_url);
        let body = serde_json::to_value(request)?;

        if let Some(err) = self
            .client
            .check_dry_run("POST", &url, &self.api_key, Some(&body))
        {
            return Err(err);
        }

        self.post_json(&url, &body).await
    }

    /// GET /v1/events — Query email delivery events.
    pub async fn email_events(&self, params: &EmailEventsParams) -> Result<Value, NlError> {
        let url = format!("{}/v1/events", self.base_url);

        if let Some(err) = self.client.check_dry_run("GET", &url, &self.api_key, None) {
            return Err(err);
        }

        self.get_with_email_params("/v1/events", params).await
    }

    // ── SMS (3) ──────────────────────────────────────────────────────────────

    /// POST /v1/sms/messages — Send an SMS message.
    pub async fn send_sms(&self, request: &SmsSendRequest) -> Result<Value, NlError> {
        let url = format!("{}/v1/sms/messages", self.base_url);
        let body = serde_json::to_value(request)?;

        if let Some(err) = self
            .client
            .check_dry_run("POST", &url, &self.api_key, Some(&body))
        {
            return Err(err);
        }

        self.post_json(&url, &body).await
    }

    /// GET /v1/sms/events — Query SMS delivery events.
    pub async fn sms_events(&self, params: &SmsEventsParams) -> Result<Value, NlError> {
        let url = format!("{}/v1/sms/events", self.base_url);

        if let Some(err) = self.client.check_dry_run("GET", &url, &self.api_key, None) {
            return Err(err);
        }

        self.get_with_sms_params("/v1/sms/events", params).await
    }

    /// GET /v1/sms/exclusive-number — List exclusive SMS numbers.
    pub async fn exclusive_number(&self) -> Result<Value, NlError> {
        let url = format!("{}/v1/sms/exclusive-number", self.base_url);

        if let Some(err) = self.client.check_dry_run("GET", &url, &self.api_key, None) {
            return Err(err);
        }

        self.get(&url).await
    }

    // ── Email Webhook (3) ────────────────────────────────────────────────────

    /// POST /v1/webhooks — Create or update an email webhook.
    pub async fn create_webhook(&self, request: &WebhookRequest) -> Result<Value, NlError> {
        let url = format!("{}/v1/webhooks", self.base_url);
        let body = serde_json::to_value(request)?;

        if let Some(err) = self
            .client
            .check_dry_run("POST", &url, &self.api_key, Some(&body))
        {
            return Err(err);
        }

        self.post_json(&url, &body).await
    }

    /// GET /v1/webhooks — List all email webhooks.
    pub async fn list_webhooks(&self) -> Result<Value, NlError> {
        let url = format!("{}/v1/webhooks", self.base_url);

        if let Some(err) = self.client.check_dry_run("GET", &url, &self.api_key, None) {
            return Err(err);
        }

        self.get(&url).await
    }

    /// DELETE /v1/webhooks — Delete an email webhook by event type.
    pub async fn delete_webhook(&self, event_type: u8) -> Result<Value, NlError> {
        let url = format!("{}/v1/webhooks", self.base_url);
        let body = serde_json::json!({ "type": event_type });

        if let Some(err) = self
            .client
            .check_dry_run("DELETE", &url, &self.api_key, Some(&body))
        {
            return Err(err);
        }

        self.delete_json(&url, &body).await
    }

    // ── SMS Webhook (3) ──────────────────────────────────────────────────────

    /// POST /v1/sms/webhooks — Create or update an SMS webhook.
    pub async fn create_sms_webhook(&self, request: &SmsWebhookRequest) -> Result<Value, NlError> {
        let url = format!("{}/v1/sms/webhooks", self.base_url);
        let body = serde_json::to_value(request)?;

        if let Some(err) = self
            .client
            .check_dry_run("POST", &url, &self.api_key, Some(&body))
        {
            return Err(err);
        }

        self.post_json(&url, &body).await
    }

    /// GET /v1/sms/webhooks — List all SMS webhooks.
    pub async fn list_sms_webhooks(&self) -> Result<Value, NlError> {
        let url = format!("{}/v1/sms/webhooks", self.base_url);

        if let Some(err) = self.client.check_dry_run("GET", &url, &self.api_key, None) {
            return Err(err);
        }

        self.get(&url).await
    }

    /// DELETE /v1/sms/webhooks — Delete an SMS webhook by event type.
    pub async fn delete_sms_webhook(&self, event_type: u8) -> Result<Value, NlError> {
        let url = format!("{}/v1/sms/webhooks", self.base_url);
        let body = serde_json::json!({ "type": event_type });

        if let Some(err) = self
            .client
            .check_dry_run("DELETE", &url, &self.api_key, Some(&body))
        {
            return Err(err);
        }

        self.delete_json(&url, &body).await
    }

    // ── Domain (3) ───────────────────────────────────────────────────────────

    /// POST /v1/domains/{domain} — Create domain verification records.
    pub async fn create_domain(&self, domain: &str) -> Result<Value, NlError> {
        let url = format!("{}/v1/domains/{}", self.base_url, domain);

        if let Some(err) = self.client.check_dry_run("POST", &url, &self.api_key, None) {
            return Err(err);
        }

        self.post_empty(&url).await
    }

    /// PUT /v1/domains/{domain} — Verify domain DNS records.
    pub async fn verify_domain(&self, domain: &str) -> Result<Value, NlError> {
        let url = format!("{}/v1/domains/{}", self.base_url, domain);

        if let Some(err) = self.client.check_dry_run("PUT", &url, &self.api_key, None) {
            return Err(err);
        }

        self.put_empty(&url).await
    }

    /// DELETE /v1/domains/{domain} — Remove domain verification.
    pub async fn remove_domain(&self, domain: &str) -> Result<Value, NlError> {
        let url = format!("{}/v1/domains/{}", self.base_url, domain);

        if let Some(err) = self
            .client
            .check_dry_run("DELETE", &url, &self.api_key, None)
        {
            return Err(err);
        }

        self.delete_empty(&url).await
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

                parse_api_response::<SnApiErrorResponse>(status, &body_text)
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

                parse_api_response::<SnApiErrorResponse>(status, &body_text)
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

                parse_api_response::<SnApiErrorResponse>(status, &body_text)
            }
        })
        .await
    }

    /// Send a PUT request with no body and automatic retry.
    async fn put_empty(&self, url: &str) -> Result<Value, NlError> {
        let url = url.to_string();
        let api_key = self.api_key.clone();
        let client = self.client;

        with_retry(|| {
            let url = url.clone();
            let api_key = api_key.clone();
            async move {
                client.log_request("PUT", &url);
                let start = Instant::now();

                let response = client
                    .http
                    .put(&url)
                    .header("x-api-key", &api_key)
                    .send()
                    .await?;

                let status = response.status().as_u16();
                let body_text = response.text().await?;
                let elapsed = start.elapsed().as_millis();

                client.log_response(status, elapsed, Some(&body_text));

                parse_api_response::<SnApiErrorResponse>(status, &body_text)
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

                parse_api_response::<SnApiErrorResponse>(status, &body_text)
            }
        })
        .await
    }

    /// Send a DELETE request with no body and automatic retry.
    async fn delete_empty(&self, url: &str) -> Result<Value, NlError> {
        let url = url.to_string();
        let api_key = self.api_key.clone();
        let client = self.client;

        with_retry(|| {
            let url = url.clone();
            let api_key = api_key.clone();
            async move {
                client.log_request("DELETE", &url);
                let start = Instant::now();

                let response = client
                    .http
                    .delete(&url)
                    .header("x-api-key", &api_key)
                    .send()
                    .await?;

                let status = response.status().as_u16();
                let body_text = response.text().await?;
                let elapsed = start.elapsed().as_millis();

                client.log_response(status, elapsed, Some(&body_text));

                parse_api_response::<SnApiErrorResponse>(status, &body_text)
            }
        })
        .await
    }

    /// Send a GET request with email event query parameters and automatic retry.
    async fn get_with_email_params(
        &self,
        path: &str,
        params: &EmailEventsParams,
    ) -> Result<Value, NlError> {
        let base_url = format!("{}{}", self.base_url, path);
        let api_key = self.api_key.clone();
        let params = params.clone();
        let client = self.client;

        with_retry(|| {
            let base_url = base_url.clone();
            let api_key = api_key.clone();
            let params = params.clone();
            async move {
                client.log_request("GET", &base_url);
                let start = Instant::now();

                let mut req = client.http.get(&base_url).header("x-api-key", &api_key);

                if let Some(ref v) = params.id {
                    req = req.query(&[("id", v.clone())]);
                }
                if let Some(ref v) = params.recipient {
                    req = req.query(&[("recipient", v.clone())]);
                }
                if let Some(ref v) = params.from {
                    req = req.query(&[("from", v.clone())]);
                }
                if let Some(ref v) = params.to {
                    req = req.query(&[("to", v.clone())]);
                }
                if let Some(ref v) = params.status {
                    req = req.query(&[("status", v.clone())]);
                }
                if let Some(v) = params.page {
                    req = req.query(&[("page", v.to_string())]);
                }
                if let Some(v) = params.size {
                    req = req.query(&[("size", v.to_string())]);
                }

                let response = req.send().await?;

                let status = response.status().as_u16();
                let body_text = response.text().await?;
                let elapsed = start.elapsed().as_millis();

                client.log_response(status, elapsed, Some(&body_text));

                parse_api_response::<SnApiErrorResponse>(status, &body_text)
            }
        })
        .await
    }

    /// Send a GET request with SMS event query parameters and automatic retry.
    async fn get_with_sms_params(
        &self,
        path: &str,
        params: &SmsEventsParams,
    ) -> Result<Value, NlError> {
        let base_url = format!("{}{}", self.base_url, path);
        let api_key = self.api_key.clone();
        let params = params.clone();
        let client = self.client;

        with_retry(|| {
            let base_url = base_url.clone();
            let api_key = api_key.clone();
            let params = params.clone();
            async move {
                client.log_request("GET", &base_url);
                let start = Instant::now();

                let mut req = client.http.get(&base_url).header("x-api-key", &api_key);

                if let Some(ref v) = params.id {
                    req = req.query(&[("id", v.clone())]);
                }
                if let Some(ref v) = params.recipient {
                    req = req.query(&[("recipient", v.clone())]);
                }
                if let Some(ref v) = params.country_code {
                    req = req.query(&[("country_code", v.clone())]);
                }
                if let Some(ref v) = params.from {
                    req = req.query(&[("from", v.clone())]);
                }
                if let Some(ref v) = params.to {
                    req = req.query(&[("to", v.clone())]);
                }
                if let Some(ref v) = params.status {
                    req = req.query(&[("status", v.clone())]);
                }
                if let Some(v) = params.page {
                    req = req.query(&[("page", v.to_string())]);
                }
                if let Some(v) = params.size {
                    req = req.query(&[("size", v.to_string())]);
                }

                let response = req.send().await?;

                let status = response.status().as_u16();
                let body_text = response.text().await?;
                let elapsed = start.elapsed().as_millis();

                client.log_response(status, elapsed, Some(&body_text));

                parse_api_response::<SnApiErrorResponse>(status, &body_text)
            }
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // ── Dry-run tests ────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_dry_run_send_email() {
        let api_client = ApiClient::new(true, 0);
        let sn = SurenotifyClient::new(&api_client, "sn-test-key-123");
        let request = EmailSendRequest {
            subject: "Test".into(),
            from_address: "test@example.com".into(),
            content: "<p>Hello {{name}}</p>".into(),
            recipients: vec![EmailRecipient {
                name: "Alice".into(),
                address: "alice@example.com".into(),
                variables: None,
            }],
            from_name: None,
            unsubscribed_link: None,
        };
        let result = sn.send_email(&request).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            NlError::DryRun(info) => {
                assert_eq!(info.method, "POST");
                assert!(info.url.ends_with("/v1/messages"));
                assert_eq!(info.headers["x-api-key"], "****...123");
                let body = info.body.unwrap();
                assert_eq!(body["subject"], "Test");
            }
            _ => panic!("Expected DryRun error"),
        }
    }

    #[tokio::test]
    async fn test_dry_run_send_sms() {
        let api_client = ApiClient::new(true, 0);
        let sn = SurenotifyClient::new(&api_client, "sn-key");
        let request = SmsSendRequest {
            content: "Code: {{code}}".into(),
            recipients: vec![SmsRecipient {
                address: "912345678".into(),
                country_code: "886".into(),
                variables: Some(HashMap::from([("code".into(), "123456".into())])),
            }],
            from: None,
            alive_mins: Some(30),
        };
        let result = sn.send_sms(&request).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            NlError::DryRun(info) => {
                assert_eq!(info.method, "POST");
                assert!(info.url.ends_with("/v1/sms/messages"));
                let body = info.body.unwrap();
                assert_eq!(body["recipients"][0]["country_code"], "886");
            }
            _ => panic!("Expected DryRun error"),
        }
    }

    #[tokio::test]
    async fn test_dry_run_email_events() {
        let api_client = ApiClient::new(true, 0);
        let sn = SurenotifyClient::new(&api_client, "sn-key");
        let params = EmailEventsParams {
            id: Some("MSG001".into()),
            ..Default::default()
        };
        let result = sn.email_events(&params).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            NlError::DryRun(info) => {
                assert_eq!(info.method, "GET");
                assert!(info.url.contains("/v1/events"));
                // Note: Query parameters are added by get_with_email_params,
                // which is not called when dry-run check returns early.
            }
            _ => panic!("Expected DryRun error"),
        }
    }

    #[tokio::test]
    async fn test_dry_run_exclusive_number() {
        let api_client = ApiClient::new(true, 0);
        let sn = SurenotifyClient::new(&api_client, "sn-key");
        let result = sn.exclusive_number().await;
        assert!(result.is_err());
        match result.unwrap_err() {
            NlError::DryRun(info) => {
                assert_eq!(info.method, "GET");
                assert!(info.url.ends_with("/v1/sms/exclusive-number"));
            }
            _ => panic!("Expected DryRun error"),
        }
    }

    #[tokio::test]
    async fn test_dry_run_create_webhook() {
        let api_client = ApiClient::new(true, 0);
        let sn = SurenotifyClient::new(&api_client, "sn-key");
        let request = WebhookRequest {
            event_type: 3,
            url: "https://example.com/webhook".into(),
        };
        let result = sn.create_webhook(&request).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            NlError::DryRun(info) => {
                assert_eq!(info.method, "POST");
                assert!(info.url.ends_with("/v1/webhooks"));
                let body = info.body.unwrap();
                assert_eq!(body["type"], 3);
            }
            _ => panic!("Expected DryRun error"),
        }
    }

    #[tokio::test]
    async fn test_dry_run_delete_webhook() {
        let api_client = ApiClient::new(true, 0);
        let sn = SurenotifyClient::new(&api_client, "sn-key");
        let result = sn.delete_webhook(5).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            NlError::DryRun(info) => {
                assert_eq!(info.method, "DELETE");
                let body = info.body.unwrap();
                assert_eq!(body["type"], 5);
            }
            _ => panic!("Expected DryRun error"),
        }
    }

    #[tokio::test]
    async fn test_dry_run_list_webhooks() {
        let api_client = ApiClient::new(true, 0);
        let sn = SurenotifyClient::new(&api_client, "sn-key");
        let result = sn.list_webhooks().await;
        assert!(result.is_err());
        match result.unwrap_err() {
            NlError::DryRun(info) => {
                assert_eq!(info.method, "GET");
                assert!(info.url.ends_with("/v1/webhooks"));
            }
            _ => panic!("Expected DryRun error"),
        }
    }

    #[tokio::test]
    async fn test_dry_run_create_sms_webhook() {
        let api_client = ApiClient::new(true, 0);
        let sn = SurenotifyClient::new(&api_client, "sn-key");
        let request = SmsWebhookRequest {
            event_type: 6,
            url: "https://example.com/sms/bounce".into(),
        };
        let result = sn.create_sms_webhook(&request).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            NlError::DryRun(info) => {
                assert_eq!(info.method, "POST");
                assert!(info.url.ends_with("/v1/sms/webhooks"));
                let body = info.body.unwrap();
                assert_eq!(body["type"], 6);
            }
            _ => panic!("Expected DryRun error"),
        }
    }

    #[tokio::test]
    async fn test_dry_run_create_domain() {
        let api_client = ApiClient::new(true, 0);
        let sn = SurenotifyClient::new(&api_client, "sn-key");
        let result = sn.create_domain("mail.example.com").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            NlError::DryRun(info) => {
                assert_eq!(info.method, "POST");
                assert!(info.url.ends_with("/v1/domains/mail.example.com"));
            }
            _ => panic!("Expected DryRun error"),
        }
    }

    #[tokio::test]
    async fn test_dry_run_verify_domain() {
        let api_client = ApiClient::new(true, 0);
        let sn = SurenotifyClient::new(&api_client, "sn-key");
        let result = sn.verify_domain("mail.example.com").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            NlError::DryRun(info) => {
                assert_eq!(info.method, "PUT");
                assert!(info.url.ends_with("/v1/domains/mail.example.com"));
            }
            _ => panic!("Expected DryRun error"),
        }
    }

    #[tokio::test]
    async fn test_dry_run_remove_domain() {
        let api_client = ApiClient::new(true, 0);
        let sn = SurenotifyClient::new(&api_client, "sn-key");
        let result = sn.remove_domain("mail.example.com").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            NlError::DryRun(info) => {
                assert_eq!(info.method, "DELETE");
                assert!(info.url.ends_with("/v1/domains/mail.example.com"));
            }
            _ => panic!("Expected DryRun error"),
        }
    }

    #[tokio::test]
    async fn test_dry_run_sms_events() {
        let api_client = ApiClient::new(true, 0);
        let sn = SurenotifyClient::new(&api_client, "sn-key");
        let params = SmsEventsParams {
            country_code: Some("886".into()),
            page: Some(1),
            ..Default::default()
        };
        let result = sn.sms_events(&params).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            NlError::DryRun(info) => {
                assert!(info.url.contains("/v1/sms/events"));
                // Note: Query parameters are added by get_with_sms_params,
                // which is not called when dry-run check returns early.
            }
            _ => panic!("Expected DryRun error"),
        }
    }

    #[tokio::test]
    async fn test_dry_run_list_sms_webhooks() {
        let api_client = ApiClient::new(true, 0);
        let sn = SurenotifyClient::new(&api_client, "sn-key");
        let result = sn.list_sms_webhooks().await;
        assert!(result.is_err());
        match result.unwrap_err() {
            NlError::DryRun(info) => {
                assert_eq!(info.method, "GET");
                assert!(info.url.ends_with("/v1/sms/webhooks"));
            }
            _ => panic!("Expected DryRun error"),
        }
    }

    #[tokio::test]
    async fn test_dry_run_delete_sms_webhook() {
        let api_client = ApiClient::new(true, 0);
        let sn = SurenotifyClient::new(&api_client, "sn-key");
        let result = sn.delete_sms_webhook(3).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            NlError::DryRun(info) => {
                assert_eq!(info.method, "DELETE");
                let body = info.body.unwrap();
                assert_eq!(body["type"], 3);
            }
            _ => panic!("Expected DryRun error"),
        }
    }
}
