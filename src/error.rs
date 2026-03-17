use serde::Serialize;
use std::collections::HashMap;
use thiserror::Error;

/// Data for a dry-run preview, boxed inside `NlError::DryRun` to keep the
/// overall enum size small.
#[derive(Debug)]
pub struct DryRunInfo {
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<serde_json::Value>,
}

/// All error types for the nl CLI, each mapping to a specific exit code.
///
/// Exit codes:
///   0 - DryRun, NoContent (success-like)
///   1 - Api (HTTP 4xx/5xx from the API)
///   2 - Validation (parameter validation failures)
///   3 - Auth, Config (authentication or configuration errors)
///   4 - Network, RateLimit (connectivity or throttling issues)
///   5 - Io (file read/write failures)
#[derive(Error, Debug)]
pub enum NlError {
    /// API error (exit code 1)
    #[error("API error {status}: [{code}] {message}", code = code.map(|c| c.to_string()).unwrap_or_else(|| "-".to_string()))]
    Api {
        status: u16,
        code: Option<i64>,
        message: String,
    },

    /// Validation error (exit code 2)
    #[error("Validation error: {0}")]
    Validation(String),

    /// Auth error (exit code 3)
    #[error("Authentication error: {0}")]
    Auth(String),

    /// Config error (exit code 3)
    #[error("Config error: {0}")]
    Config(String),

    /// Network error (exit code 4)
    #[error("Network error: {0}")]
    Network(String),

    /// Rate limit exceeded (exit code 4)
    #[allow(dead_code)]
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    /// IO error (exit code 5)
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Dry-run preview (exit code 0)
    #[error("Dry run")]
    DryRun(Box<DryRunInfo>),

    /// 204 No Content (exit code 0)
    #[error("No content")]
    NoContent,
}

/// Serializable error envelope for JSON stderr output.
#[derive(Debug, Serialize)]
struct ErrorEnvelope {
    error: ErrorDetail,
}

#[derive(Debug, Serialize)]
struct ErrorDetail {
    #[serde(rename = "type")]
    error_type: String,
    message: String,
    exit_code: i32,
}

/// Serializable dry-run envelope for JSON stderr output.
#[derive(Debug, Serialize)]
struct DryRunEnvelope {
    dry_run: DryRunDetail,
}

#[derive(Debug, Serialize)]
struct DryRunDetail {
    method: String,
    url: String,
    headers: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    body: Option<serde_json::Value>,
}

impl NlError {
    /// Returns the process exit code for this error variant.
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::DryRun(_) | Self::NoContent => 0,
            Self::Api { .. } => 1,
            Self::Validation(_) => 2,
            Self::Auth(_) | Self::Config(_) => 3,
            Self::Network(_) | Self::RateLimit(_) => 4,
            Self::Io(_) => 5,
        }
    }

    /// Returns a machine-readable error type string.
    pub fn error_type(&self) -> &str {
        match self {
            Self::Api { .. } => "api",
            Self::Validation(_) => "validation",
            Self::Auth(_) => "auth",
            Self::Config(_) => "config",
            Self::Network(_) => "network",
            Self::RateLimit(_) => "rate_limit",
            Self::Io(_) => "io",
            Self::DryRun(_) => "dry_run",
            Self::NoContent => "no_content",
        }
    }

    /// Outputs structured JSON to stderr.
    ///
    /// For DryRun: outputs the request preview.
    /// For all other variants: outputs an error envelope with type, message, and exit code.
    pub fn to_json_stderr(&self) {
        match self {
            Self::DryRun(info) => {
                let envelope = DryRunEnvelope {
                    dry_run: DryRunDetail {
                        method: info.method.clone(),
                        url: info.url.clone(),
                        headers: info.headers.clone(),
                        body: info.body.clone(),
                    },
                };
                // Unwrap is safe: DryRunEnvelope is always serializable.
                eprintln!("{}", serde_json::to_string_pretty(&envelope).unwrap());
            }
            _ => {
                let envelope = ErrorEnvelope {
                    error: ErrorDetail {
                        error_type: self.error_type().to_string(),
                        message: self.to_string(),
                        exit_code: self.exit_code(),
                    },
                };
                eprintln!("{}", serde_json::to_string_pretty(&envelope).unwrap());
            }
        }
    }
}

impl From<reqwest::Error> for NlError {
    fn from(err: reqwest::Error) -> Self {
        NlError::Network(err.to_string())
    }
}

impl From<toml::de::Error> for NlError {
    fn from(err: toml::de::Error) -> Self {
        NlError::Config(err.to_string())
    }
}

impl From<serde_json::Error> for NlError {
    fn from(err: serde_json::Error) -> Self {
        NlError::Validation(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exit_codes() {
        assert_eq!(
            NlError::DryRun(Box::new(DryRunInfo {
                method: "GET".into(),
                url: "http://example.com".into(),
                headers: HashMap::new(),
                body: None,
            }))
            .exit_code(),
            0
        );
        assert_eq!(NlError::NoContent.exit_code(), 0);
        assert_eq!(
            NlError::Api {
                status: 400,
                code: Some(40001),
                message: "bad request".into(),
            }
            .exit_code(),
            1
        );
        assert_eq!(NlError::Validation("bad input".into()).exit_code(), 2);
        assert_eq!(NlError::Auth("no key".into()).exit_code(), 3);
        assert_eq!(NlError::Config("missing file".into()).exit_code(), 3);
        assert_eq!(NlError::Network("timeout".into()).exit_code(), 4);
        assert_eq!(NlError::RateLimit("too fast".into()).exit_code(), 4);

        let io_err = NlError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found",
        ));
        assert_eq!(io_err.exit_code(), 5);
    }

    #[test]
    fn test_error_types() {
        assert_eq!(
            NlError::Api {
                status: 500,
                code: None,
                message: "internal".into()
            }
            .error_type(),
            "api"
        );
        assert_eq!(NlError::Validation("x".into()).error_type(), "validation");
        assert_eq!(NlError::Auth("x".into()).error_type(), "auth");
        assert_eq!(NlError::Config("x".into()).error_type(), "config");
        assert_eq!(NlError::Network("x".into()).error_type(), "network");
        assert_eq!(NlError::RateLimit("x".into()).error_type(), "rate_limit");
        assert_eq!(
            NlError::Io(std::io::Error::new(std::io::ErrorKind::Other, "x")).error_type(),
            "io"
        );
        assert_eq!(
            NlError::DryRun(Box::new(DryRunInfo {
                method: "GET".into(),
                url: "http://x".into(),
                headers: HashMap::new(),
                body: None,
            }))
            .error_type(),
            "dry_run"
        );
        assert_eq!(NlError::NoContent.error_type(), "no_content");
    }

    #[test]
    fn test_api_error_display() {
        let err = NlError::Api {
            status: 400,
            code: Some(40001),
            message: "field validation failed".into(),
        };
        assert_eq!(
            err.to_string(),
            "API error 400: [40001] field validation failed"
        );

        let err_no_code = NlError::Api {
            status: 500,
            code: None,
            message: "internal server error".into(),
        };
        assert_eq!(
            err_no_code.to_string(),
            "API error 500: [-] internal server error"
        );
    }

    #[test]
    fn test_from_serde_json_error() {
        let json_err: Result<serde_json::Value, _> = serde_json::from_str("not json");
        let nl_err: NlError = json_err.unwrap_err().into();
        assert_eq!(nl_err.exit_code(), 2);
        assert_eq!(nl_err.error_type(), "validation");
    }

    #[test]
    fn test_from_toml_error() {
        let toml_err: Result<toml::Value, _> = toml::from_str("[invalid");
        let nl_err: NlError = toml_err.unwrap_err().into();
        assert_eq!(nl_err.exit_code(), 3);
        assert_eq!(nl_err.error_type(), "config");
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let nl_err: NlError = io_err.into();
        assert_eq!(nl_err.exit_code(), 5);
        assert_eq!(nl_err.error_type(), "io");
        assert!(nl_err.to_string().contains("access denied"));
    }

    #[test]
    fn test_dry_run_display() {
        let err = NlError::DryRun(Box::new(DryRunInfo {
            method: "POST".into(),
            url: "https://api.newsleopard.com/v1/campaign/normal/submit".into(),
            headers: HashMap::from([("x-api-key".into(), "****".into())]),
            body: Some(serde_json::json!({"name": "test"})),
        }));
        assert_eq!(err.to_string(), "Dry run");
        assert_eq!(err.exit_code(), 0);
    }
}
