//! Token-bucket rate limiter built on the `governor` crate.
//!
//! Two pre-configured buckets:
//!   - EDM general: 2 requests per second
//!   - Report export: 1 request per 10 seconds

use governor::{
    clock::DefaultClock,
    middleware::NoOpMiddleware,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter as GovernorLimiter,
};
use std::num::NonZeroU32;

/// Type alias for the concrete governor limiter used throughout the project.
pub type GovLimiter = GovernorLimiter<NotKeyed, InMemoryState, DefaultClock, NoOpMiddleware>;

/// A thin wrapper around `governor::RateLimiter` with factory methods for
/// the two rate-limit tiers used by the NewsLeopard EDM API.
pub struct NlRateLimiter {
    limiter: GovLimiter,
}

impl NlRateLimiter {
    /// EDM general rate limit: 2 requests per second.
    pub fn edm_general() -> Self {
        let quota = Quota::per_second(NonZeroU32::new(2).unwrap());
        Self {
            limiter: GovernorLimiter::direct(quota),
        }
    }

    /// Report export rate limit: 1 request per 10 seconds.
    pub fn report_export() -> Self {
        let quota = Quota::with_period(std::time::Duration::from_secs(10))
            .unwrap()
            .allow_burst(NonZeroU32::new(1).unwrap());
        Self {
            limiter: GovernorLimiter::direct(quota),
        }
    }

    /// Waits until a token is available, blocking the current async task.
    pub async fn until_ready(&self) {
        self.limiter.until_ready().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_edm_general_limiter_allows_burst() {
        let limiter = NlRateLimiter::edm_general();
        // The first 2 requests should pass without delay.
        limiter.until_ready().await;
        limiter.until_ready().await;
    }

    #[tokio::test]
    async fn test_report_limiter_allows_one() {
        let limiter = NlRateLimiter::report_export();
        // The first request should pass without delay.
        limiter.until_ready().await;
    }
}
