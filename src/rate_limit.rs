//! Rate limiting for API requests.
//!
//! This module provides rate limiting to prevent exceeding API quotas.
//! The Spiris API allows 600 requests per minute per endpoint.
//!
//! # Feature Flag
//!
//! This module is only available when the `rate-limit` feature is enabled:
//!
//! ```toml
//! [dependencies]
//! spiris = { version = "0.1", features = ["rate-limit"] }
//! ```
//!
//! # Example
//!
//! ```ignore
//! use spiris::{Client, AccessToken, ClientConfig};
//! use spiris::rate_limit::RateLimitConfig;
//!
//! let token = AccessToken::new("token".to_string(), 3600, None);
//! let config = ClientConfig::new()
//!     .rate_limit(RateLimitConfig::default());
//! let client = Client::with_config(token, config);
//!
//! // All requests will now be rate-limited
//! ```

use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};
use std::num::NonZeroU32;
use std::sync::Arc;

/// Rate limiting configuration.
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum requests per minute.
    pub requests_per_minute: u32,

    /// Allow burst of requests up to this limit.
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: crate::client::RATE_LIMIT_PER_MINUTE,
            burst_size: 10,
        }
    }
}

impl RateLimitConfig {
    /// Create a new rate limit configuration.
    pub fn new(requests_per_minute: u32) -> Self {
        Self {
            requests_per_minute,
            burst_size: 10,
        }
    }

    /// Set the burst size (number of requests allowed in quick succession).
    pub fn burst_size(mut self, burst: u32) -> Self {
        self.burst_size = burst;
        self
    }
}

/// Internal rate limiter using the governor crate.
pub(crate) struct ApiRateLimiter {
    limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
}

impl ApiRateLimiter {
    /// Create a new rate limiter from configuration.
    pub fn new(config: &RateLimitConfig) -> Self {
        let quota = Quota::per_minute(
            NonZeroU32::new(config.requests_per_minute).unwrap_or(NonZeroU32::new(600).unwrap()),
        )
        .allow_burst(
            NonZeroU32::new(config.burst_size).unwrap_or(NonZeroU32::new(1).unwrap()),
        );

        Self {
            limiter: Arc::new(RateLimiter::direct(quota)),
        }
    }

    /// Wait until a request can be made.
    ///
    /// This method blocks (asynchronously) until the rate limit allows a new request.
    pub async fn acquire(&self) {
        self.limiter.until_ready().await;
    }

    /// Try to acquire a permit without waiting.
    ///
    /// Returns `true` if a request can be made immediately, `false` otherwise.
    #[allow(dead_code)]
    pub fn try_acquire(&self) -> bool {
        self.limiter.check().is_ok()
    }
}

impl Clone for ApiRateLimiter {
    fn clone(&self) -> Self {
        Self {
            limiter: Arc::clone(&self.limiter),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_config_default() {
        let config = RateLimitConfig::default();
        assert_eq!(config.requests_per_minute, 600);
        assert_eq!(config.burst_size, 10);
    }

    #[test]
    fn test_rate_limit_config_builder() {
        let config = RateLimitConfig::new(100).burst_size(5);
        assert_eq!(config.requests_per_minute, 100);
        assert_eq!(config.burst_size, 5);
    }

    #[tokio::test]
    async fn test_rate_limiter_allows_burst() {
        let config = RateLimitConfig::new(600).burst_size(5);
        let limiter = ApiRateLimiter::new(&config);

        // Should allow burst of 5 requests immediately
        for _ in 0..5 {
            assert!(limiter.try_acquire());
        }
    }

    #[tokio::test]
    async fn test_rate_limiter_acquire() {
        let config = RateLimitConfig::new(600).burst_size(2);
        let limiter = ApiRateLimiter::new(&config);

        // Should complete without blocking for first few requests
        limiter.acquire().await;
        limiter.acquire().await;
    }

    #[test]
    fn test_rate_limiter_clone() {
        let config = RateLimitConfig::new(600);
        let limiter1 = ApiRateLimiter::new(&config);
        let limiter2 = limiter1.clone();

        // Both limiters should share the same internal state
        assert!(limiter1.try_acquire());
        // After clone uses quota, original should see the effect
        // (they share the same Arc)
    }
}
