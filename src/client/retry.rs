//! Exponential backoff retry for transient HTTP errors.
//!
//! Retries on:
//!   - `NlError::Network` (connection failures, timeouts)
//!   - `NlError::RateLimit` (local rate-limit exhaustion)
//!   - `NlError::Api` with status 429 or >= 500
//!
//! All other errors are treated as permanent and returned immediately.
//!
//! Configuration:
//!   - Initial interval: 500 ms
//!   - Max interval: 30 s
//!   - Max elapsed time: 120 s (total retry budget)

use std::future::Future;
use std::time::Duration;

use backoff::ExponentialBackoffBuilder;

use crate::error::NlError;

/// Wraps an async closure with exponential backoff retry.
///
/// The closure `f` is called repeatedly until it succeeds or a permanent error
/// is returned. Transient errors (network, rate-limit, server 5xx, 429) trigger
/// a backoff sleep before the next attempt.
///
/// # Example
///
/// ```ignore
/// let result = with_retry(|| async {
///     client.get_balance().await
/// }).await?;
/// ```
pub async fn with_retry<F, Fut, T>(f: F) -> Result<T, NlError>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, NlError>>,
{
    let backoff = ExponentialBackoffBuilder::default()
        .with_initial_interval(Duration::from_millis(500))
        .with_max_interval(Duration::from_secs(30))
        .with_max_elapsed_time(Some(Duration::from_secs(120)))
        .build();

    backoff::future::retry(backoff, || async {
        match f().await {
            Ok(v) => Ok(v),
            Err(e) => {
                if is_transient(&e) {
                    Err(backoff::Error::transient(e))
                } else {
                    Err(backoff::Error::permanent(e))
                }
            }
        }
    })
    .await
}

/// Determines whether an `NlError` is transient and should be retried.
fn is_transient(e: &NlError) -> bool {
    match e {
        NlError::Network(_) => true,
        NlError::RateLimit(_) => true,
        NlError::Api { status, .. } if *status >= 500 => true,
        NlError::Api { status, .. } if *status == 429 => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[test]
    fn test_is_transient_network() {
        assert!(is_transient(&NlError::Network("timeout".into())));
    }

    #[test]
    fn test_is_transient_rate_limit() {
        assert!(is_transient(&NlError::RateLimit("throttled".into())));
    }

    #[test]
    fn test_is_transient_server_error() {
        assert!(is_transient(&NlError::Api {
            status: 500,
            code: None,
            message: "internal".into(),
        }));
        assert!(is_transient(&NlError::Api {
            status: 502,
            code: None,
            message: "bad gateway".into(),
        }));
    }

    #[test]
    fn test_is_transient_429() {
        assert!(is_transient(&NlError::Api {
            status: 429,
            code: None,
            message: "too many requests".into(),
        }));
    }

    #[test]
    fn test_not_transient_client_error() {
        assert!(!is_transient(&NlError::Api {
            status: 400,
            code: Some(40001),
            message: "bad request".into(),
        }));
        assert!(!is_transient(&NlError::Validation("bad".into())));
        assert!(!is_transient(&NlError::Auth("no key".into())));
        assert!(!is_transient(&NlError::Config("missing".into())));
    }

    #[tokio::test]
    async fn test_with_retry_succeeds_immediately() {
        let result = with_retry(|| async { Ok::<_, NlError>(42) }).await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_with_retry_permanent_error() {
        let result =
            with_retry(|| async { Err::<i32, _>(NlError::Validation("permanent".into())) }).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().exit_code(), 2);
    }

    #[tokio::test]
    async fn test_with_retry_recovers_after_transient() {
        let attempts = AtomicU32::new(0);
        let result = with_retry(|| async {
            let n = attempts.fetch_add(1, Ordering::SeqCst);
            if n < 2 {
                Err(NlError::Network("transient failure".into()))
            } else {
                Ok(99)
            }
        })
        .await;
        assert_eq!(result.unwrap(), 99);
        assert!(attempts.load(Ordering::SeqCst) >= 3);
    }
}
