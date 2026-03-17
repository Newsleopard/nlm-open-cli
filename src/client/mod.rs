//! HTTP client layer: shared infrastructure, EDM client, Surenotify client.
//!
//! `ApiClient` holds the reqwest HTTP client, rate limiters, and global flags
//! (dry-run, verbose). The protocol-specific clients (`EdmClient`, `SurenotifyClient`)
//! borrow the `ApiClient` and add endpoint methods.

pub mod edm;
pub mod mcp;
pub mod rate_limiter;
pub mod retry;
pub mod surenotify;

use reqwest::Client;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::time::Duration;

use crate::error::{DryRunInfo, NlError};
use rate_limiter::NlRateLimiter;

/// Shared HTTP client with rate limiters and global flags.
///
/// Created once per CLI invocation and borrowed by `EdmClient` / `SurenotifyClient`.
pub struct ApiClient {
    /// The underlying reqwest HTTP client (rustls, no OpenSSL).
    pub(crate) http: Client,
    /// Token-bucket limiter for general EDM API calls (2 req/s).
    pub(crate) edm_limiter: NlRateLimiter,
    /// Token-bucket limiter for report export calls (1 req/10s).
    pub(crate) report_limiter: NlRateLimiter,
    /// When true, return a `DryRun` error instead of sending HTTP requests.
    pub(crate) dry_run: bool,
    /// Verbosity level: 0 = quiet, 1 = request summary, 2+ = full body.
    pub(crate) verbose: u8,
}

impl ApiClient {
    /// Creates a new `ApiClient` with the given dry-run and verbose settings.
    pub fn new(dry_run: bool, verbose: u8) -> Self {
        let http = Client::builder()
            .user_agent(format!("nl-cli/{}", env!("CARGO_PKG_VERSION")))
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
        Self {
            http,
            edm_limiter: NlRateLimiter::edm_general(),
            report_limiter: NlRateLimiter::report_export(),
            dry_run,
            verbose,
        }
    }

    /// If dry-run mode is active, returns a `DryRun` error containing the
    /// request preview. Otherwise returns `None` and the caller should proceed
    /// with the actual HTTP call.
    pub(crate) fn check_dry_run(
        &self,
        method: &str,
        url: &str,
        api_key: &str,
        body: Option<&serde_json::Value>,
    ) -> Option<NlError> {
        if self.dry_run {
            let mut headers = HashMap::new();
            headers.insert("x-api-key".to_string(), mask_api_key(api_key));
            headers.insert("content-type".to_string(), "application/json".to_string());
            Some(NlError::DryRun(Box::new(DryRunInfo {
                method: method.to_string(),
                url: url.to_string(),
                headers,
                body: body.cloned(),
            })))
        } else {
            None
        }
    }

    /// Logs the outgoing request when verbose >= 1.
    pub(crate) fn log_request(&self, method: &str, url: &str) {
        if self.verbose >= 1 {
            tracing::info!("{} {}", method, url);
        }
    }

    /// Logs the response status and timing when verbose >= 1,
    /// and the body when verbose >= 2.
    pub(crate) fn log_response(&self, status: u16, elapsed_ms: u128, body: Option<&str>) {
        if self.verbose >= 1 {
            tracing::info!("[{} {}ms]", status, elapsed_ms);
        }
        if self.verbose >= 2 {
            if let Some(body) = body {
                tracing::debug!("Response body: {}", body);
            }
        }
    }
}

/// Masks an API key for safe display: shows only the last 3 characters.
pub fn mask_api_key(key: &str) -> String {
    if key.len() <= 3 {
        "****".to_string()
    } else {
        format!("****...{}", &key[key.len() - 3..])
    }
}

/// Generic response parser for API calls.
///
/// Handles 204 No Content, error responses (4xx/5xx) with optional structured error,
/// and successful JSON bodies. The error response type must implement `Deserialize`
/// and have `error_code: Option<u32>` and `message: String` fields.
pub(crate) fn parse_api_response<E>(
    status: u16,
    body_text: &str,
) -> Result<serde_json::Value, NlError>
where
    E: DeserializeOwned + std::fmt::Debug,
    for<'a> &'a E: Into<(Option<i64>, String)>,
{
    // 204 No Content
    if status == 204 {
        return Err(NlError::NoContent);
    }

    // 4xx / 5xx errors
    if status >= 400 {
        // Try to extract structured error from the response body.
        if let Ok(err_resp) = serde_json::from_str::<E>(body_text) {
            let (code, message) = (&err_resp).into();
            return Err(NlError::Api {
                status,
                code,
                message,
            });
        }

        // Fallback: use the raw body as the error message.
        return Err(NlError::Api {
            status,
            code: None,
            message: if body_text.is_empty() {
                format!("HTTP {}", status)
            } else {
                body_text.to_string()
            },
        });
    }

    // Successful response — parse as JSON.
    if body_text.is_empty() {
        Ok(serde_json::Value::Null)
    } else {
        serde_json::from_str(body_text).map_err(|e| NlError::Api {
            status,
            code: None,
            message: format!("Failed to parse response JSON: {}", e),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_api_key_long() {
        assert_eq!(mask_api_key("abcdefghij"), "****...hij");
    }

    #[test]
    fn test_mask_api_key_short() {
        assert_eq!(mask_api_key("ab"), "****");
        assert_eq!(mask_api_key("abc"), "****");
    }

    #[test]
    fn test_mask_api_key_empty() {
        assert_eq!(mask_api_key(""), "****");
    }

    #[test]
    fn test_mask_api_key_four_chars() {
        assert_eq!(mask_api_key("abcd"), "****...bcd");
    }

    #[test]
    fn test_check_dry_run_active() {
        let client = ApiClient::new(true, 0);
        let result = client.check_dry_run(
            "POST",
            "https://api.newsleopard.com/v1/campaign/normal/submit",
            "test-api-key-123",
            Some(&serde_json::json!({"name": "test"})),
        );
        assert!(result.is_some());
        match result.unwrap() {
            NlError::DryRun(info) => {
                assert_eq!(info.method, "POST");
                assert!(info.url.contains("/v1/campaign/normal/submit"));
                assert_eq!(info.headers["x-api-key"], "****...123");
                assert!(info.body.is_some());
            }
            _ => panic!("Expected DryRun error"),
        }
    }

    #[test]
    fn test_check_dry_run_inactive() {
        let client = ApiClient::new(false, 0);
        let result =
            client.check_dry_run("GET", "https://api.newsleopard.com/v1/balance", "key", None);
        assert!(result.is_none());
    }
}
