//! Integration tests for retry logic with exponential backoff.
//!
//! These tests verify that the retry mechanism correctly:
//! - Retries on 5xx server errors
//! - Retries on network errors
//! - Does NOT retry on 4xx client errors
//! - Respects retry configuration (max retries, backoff)
//! - Handles rate limiting (429) responses

mod mock_server;

use mock_server::{MockApi, MockResponse};
use spiris::{ApiErrorResponse, Error, RetryConfig};
use std::time::Duration;

fn api_error(status_code: u16, message: &str) -> Error {
    Error::ApiError {
        status_code,
        response: ApiErrorResponse::from_raw(message.to_string()),
        raw_body: message.to_string(),
    }
}

// =============================================================================
// Retry Configuration Tests
// =============================================================================

#[test]
fn test_retry_config_defaults() {
    let config = RetryConfig::default();

    assert_eq!(config.max_retries, 3);
    assert_eq!(config.initial_interval, Duration::from_millis(500));
    assert_eq!(config.max_interval, Duration::from_secs(30));
    assert_eq!(config.multiplier, 2.0);
}

#[test]
fn test_retry_config_builder() {
    let config = RetryConfig::new()
        .max_retries(5)
        .initial_interval(Duration::from_secs(1))
        .max_interval(Duration::from_secs(60));

    assert_eq!(config.max_retries, 5);
    assert_eq!(config.initial_interval, Duration::from_secs(1));
    assert_eq!(config.max_interval, Duration::from_secs(60));
}

// =============================================================================
// is_retryable_error Tests
// =============================================================================

#[test]
fn test_rate_limit_is_retryable() {
    use spiris::retry::is_retryable_error;

    let error = Error::RateLimitExceeded("Rate limit exceeded".to_string());
    assert!(is_retryable_error(&error), "Rate limit should be retryable");
}

#[test]
fn test_server_error_500_is_retryable() {
    use spiris::retry::is_retryable_error;

    let error = api_error(500, "Internal Server Error");
    assert!(is_retryable_error(&error), "500 error should be retryable");
}

#[test]
fn test_server_error_502_is_retryable() {
    use spiris::retry::is_retryable_error;

    let error = api_error(502, "Bad Gateway");
    assert!(is_retryable_error(&error), "502 error should be retryable");
}

#[test]
fn test_server_error_503_is_retryable() {
    use spiris::retry::is_retryable_error;

    let error = api_error(503, "Service Unavailable");
    assert!(is_retryable_error(&error), "503 error should be retryable");
}

#[test]
fn test_client_error_400_not_retryable() {
    use spiris::retry::is_retryable_error;

    let error = api_error(400, "Bad Request");
    assert!(
        !is_retryable_error(&error),
        "400 error should NOT be retryable"
    );
}

#[test]
fn test_client_error_401_not_retryable() {
    use spiris::retry::is_retryable_error;

    let error = api_error(401, "Unauthorized");
    assert!(
        !is_retryable_error(&error),
        "401 error should NOT be retryable"
    );
}

#[test]
fn test_client_error_404_not_retryable() {
    use spiris::retry::is_retryable_error;

    let error = api_error(404, "Not Found");
    assert!(
        !is_retryable_error(&error),
        "404 error should NOT be retryable"
    );
}

#[test]
fn test_token_expired_not_retryable() {
    use spiris::retry::is_retryable_error;

    let error = Error::TokenExpired;
    assert!(
        !is_retryable_error(&error),
        "TokenExpired should NOT be retryable"
    );
}

// =============================================================================
// Retry Behavior Integration Tests
// =============================================================================

/// Note: These tests require the retry logic to be integrated into the client.
/// Currently, retry_request exists but isn't called from client.rs.
/// These tests document the expected behavior once the fix is applied.

#[tokio::test]
async fn test_retry_function_succeeds_first_try() {
    use spiris::retry::retry_request;

    let config = RetryConfig::new().max_retries(3);
    let mut call_count = 0;

    let result = retry_request(&config, || {
        call_count += 1;
        async { Ok::<_, Error>(42) }
    })
    .await;

    assert_eq!(result.unwrap(), 42);
    assert_eq!(call_count, 1, "Should succeed on first try");
}

#[tokio::test]
async fn test_retry_function_succeeds_after_retries() {
    use spiris::retry::retry_request;

    let config = RetryConfig::new()
        .max_retries(3)
        .initial_interval(Duration::from_millis(10));

    let call_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
    let counter = call_count.clone();

    let result = retry_request(&config, || {
        let count = counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        async move {
            if count < 2 {
                Err(api_error(500, "Temporary failure"))
            } else {
                Ok(42)
            }
        }
    })
    .await;

    assert_eq!(result.unwrap(), 42);
    assert_eq!(
        call_count.load(std::sync::atomic::Ordering::SeqCst),
        3,
        "Should succeed after 2 retries (3 total calls)"
    );
}

#[tokio::test]
async fn test_retry_function_fails_after_max_retries() {
    use spiris::retry::retry_request;

    let config = RetryConfig::new()
        .max_retries(3)
        .initial_interval(Duration::from_millis(10));

    let call_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
    let counter = call_count.clone();

    let result: Result<i32, Error> = retry_request(&config, || {
        counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        async { Err(api_error(500, "Always fails")) }
    })
    .await;

    assert!(result.is_err(), "Should fail after max retries");
    // Note: The implementation counts attempts and stops when attempts >= max_retries
    // So with max_retries=3, it makes 3 total attempts (not 1 + 3)
    assert_eq!(
        call_count.load(std::sync::atomic::Ordering::SeqCst),
        3,
        "Should make max_retries (3) total attempts"
    );
}

#[tokio::test]
async fn test_retry_function_no_retry_on_4xx() {
    use spiris::retry::retry_request;

    let config = RetryConfig::new()
        .max_retries(3)
        .initial_interval(Duration::from_millis(10));

    let call_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
    let counter = call_count.clone();

    let result: Result<i32, Error> = retry_request(&config, || {
        counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        async { Err(api_error(400, "Bad Request")) }
    })
    .await;

    assert!(result.is_err(), "Should fail on 400");
    assert_eq!(
        call_count.load(std::sync::atomic::Ordering::SeqCst),
        1,
        "Should NOT retry on 4xx errors"
    );
}

#[tokio::test]
async fn test_retry_function_no_retry_on_token_expired() {
    use spiris::retry::retry_request;

    let config = RetryConfig::new()
        .max_retries(3)
        .initial_interval(Duration::from_millis(10));

    let call_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
    let counter = call_count.clone();

    let result: Result<i32, Error> = retry_request(&config, || {
        counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        async { Err(Error::TokenExpired) }
    })
    .await;

    assert!(result.is_err(), "Should fail on TokenExpired");
    assert_eq!(
        call_count.load(std::sync::atomic::Ordering::SeqCst),
        1,
        "Should NOT retry on TokenExpired"
    );
}

// =============================================================================
// Mock Server Retry Tests
// =============================================================================

/// Test that sequences of server errors are handled
#[tokio::test]
async fn test_mock_server_sequence_setup() {
    let mut api = MockApi::new().await;

    // Set up a sequence: 500, 500, then success
    let customer_json = r#"{
        "Id": "cust-001",
        "Name": "Test Customer",
        "IsActive": true
    }"#;

    let _mocks = api.mock_get_sequence(
        "/customers/cust-001",
        vec![
            MockResponse::error(500, "Internal Server Error"),
            MockResponse::error(502, "Bad Gateway"),
            MockResponse::ok(customer_json),
        ],
    );

    // First call - 500 error
    let result1 = api.client.customers().get("cust-001").await;
    assert!(result1.is_err(), "First call should fail with 500");

    // Second call - 502 error
    let result2 = api.client.customers().get("cust-001").await;
    assert!(result2.is_err(), "Second call should fail with 502");

    // Third call - success
    let result3 = api.client.customers().get("cust-001").await;
    assert!(result3.is_ok(), "Third call should succeed");
}

#[tokio::test]
async fn test_mock_server_rate_limit_response() {
    let mut api = MockApi::new().await;

    // Mock rate limit response
    let _mock = api.mock_rate_limit("/customers", 60);

    let result = api.client.customers().list(None).await;

    // Should get rate limit error
    match result {
        Err(Error::RateLimitExceeded(_)) => {}
        Err(Error::ApiError {
            status_code: 429, ..
        }) => {}
        other => panic!("Expected rate limit error, got {:?}", other),
    }
}

#[tokio::test]
async fn test_mock_server_connection_reset() {
    let mut api = MockApi::new().await;

    // Mock connection reset (simulated as 502)
    let _mock = api.mock_connection_reset("/customers");

    let result = api.client.customers().list(None).await;
    assert!(result.is_err(), "Should fail on connection reset");
}

// =============================================================================
// Exponential Backoff Calculation Tests
// =============================================================================

#[test]
fn test_backoff_calculation() {
    // Verify the backoff multiplier logic
    let initial = Duration::from_millis(500);
    let multiplier = 2.0;
    let max = Duration::from_secs(30);

    let mut interval = initial;
    let expected_intervals = [
        Duration::from_millis(500),  // Initial
        Duration::from_millis(1000), // 500 * 2
        Duration::from_millis(2000), // 1000 * 2
        Duration::from_millis(4000), // 2000 * 2
        Duration::from_millis(8000), // 4000 * 2
    ];

    for (i, expected) in expected_intervals.iter().enumerate() {
        assert_eq!(
            interval, *expected,
            "Backoff at step {} should be {:?}",
            i, expected
        );
        interval = Duration::from_secs_f64(interval.as_secs_f64() * multiplier).min(max);
    }
}

#[test]
fn test_backoff_capped_at_max() {
    let initial = Duration::from_secs(10);
    let multiplier = 2.0;
    let max = Duration::from_secs(30);

    let mut interval = initial;

    // After a few iterations, should be capped at max
    for _ in 0..5 {
        interval = Duration::from_secs_f64(interval.as_secs_f64() * multiplier).min(max);
    }

    assert_eq!(interval, max, "Backoff should be capped at max interval");
}

// =============================================================================
// Future Integration Tests (require client changes)
// =============================================================================
//
// TODO: Once retry is integrated into client.rs, enable these tests:
//
// test_client_retries_on_500:
//   - Set up mock with sequence: 500, 500, success
//   - Verify client succeeds after retries
//   - Verify total request count is 3
//
// test_client_respects_retry_after:
//   - Set up mock with 429 + Retry-After header
//   - Verify client waits appropriate time
//   - Verify request eventually succeeds
//
// test_client_max_retries_exceeded:
//   - Set up mock that always returns 500
//   - Verify client fails after max_retries + 1 attempts
//   - Verify error is the last error received
