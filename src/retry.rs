//! Retry logic with exponential backoff for API requests.

use crate::error::{Error, Result};
use std::time::Duration;
use tokio::time::sleep;

/// Configuration for retry behavior.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts.
    pub max_retries: u32,

    /// Initial backoff duration.
    pub initial_interval: Duration,

    /// Maximum backoff duration.
    pub max_interval: Duration,

    /// Multiplier for exponential backoff.
    pub multiplier: f64,

    /// Maximum elapsed time before giving up.
    pub max_elapsed_time: Option<Duration>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_interval: Duration::from_millis(500),
            max_interval: Duration::from_secs(30),
            multiplier: 2.0,
            max_elapsed_time: Some(Duration::from_secs(120)),
        }
    }
}

impl RetryConfig {
    /// Create a new retry configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum number of retries.
    pub fn max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Set initial backoff interval.
    pub fn initial_interval(mut self, interval: Duration) -> Self {
        self.initial_interval = interval;
        self
    }

    /// Set maximum backoff interval.
    pub fn max_interval(mut self, interval: Duration) -> Self {
        self.max_interval = interval;
        self
    }
}

/// Retry a request operation with exponential backoff.
///
/// This function will retry the operation if it fails with a retryable error
/// (network errors, rate limits, server errors).
pub async fn retry_request<T, F, Fut>(config: &RetryConfig, mut operation: F) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut current_interval = config.initial_interval;
    let mut attempts = 0;

    loop {
        match operation().await {
            Ok(value) => return Ok(value),
            Err(err) => {
                attempts += 1;

                // If not retryable or out of retries, return error
                if !is_retryable_error(&err) || attempts >= config.max_retries {
                    return Err(err);
                }

                // Wait before retrying
                sleep(current_interval).await;

                // Calculate next backoff interval
                current_interval =
                    Duration::from_secs_f64(current_interval.as_secs_f64() * config.multiplier)
                        .min(config.max_interval);
            }
        }
    }
}

/// Determine if an error is retryable.
pub fn is_retryable_error(error: &Error) -> bool {
    match error {
        Error::Http(_) => true,              // Network errors are retryable
        Error::RateLimitExceeded(_) => true, // Rate limits are retryable
        Error::ApiError { status_code, .. } => {
            // Retry on server errors (5xx) but not client errors (4xx)
            *status_code >= 500
        }
        Error::TokenExpired => false, // Need to refresh, not retry
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_config_defaults() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_interval, Duration::from_millis(500));
    }

    #[test]
    fn test_retry_config_builder() {
        let config = RetryConfig::new()
            .max_retries(5)
            .initial_interval(Duration::from_secs(1));

        assert_eq!(config.max_retries, 5);
        assert_eq!(config.initial_interval, Duration::from_secs(1));
    }

    #[test]
    fn test_is_retryable_error() {
        use crate::error::ApiErrorResponse;

        assert!(is_retryable_error(&Error::RateLimitExceeded(
            "test".to_string()
        )));
        assert!(is_retryable_error(&Error::ApiError {
            status_code: 500,
            response: ApiErrorResponse::from_raw("Server error".to_string()),
            raw_body: "Server error".to_string(),
        }));
        assert!(!is_retryable_error(&Error::ApiError {
            status_code: 400,
            response: ApiErrorResponse::from_raw("Bad request".to_string()),
            raw_body: "Bad request".to_string(),
        }));
        assert!(!is_retryable_error(&Error::TokenExpired));
    }
}
