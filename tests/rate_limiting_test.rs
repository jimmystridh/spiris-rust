//! Integration tests for rate limiting behavior.
//!
//! These tests verify that the client correctly handles rate limiting:
//! - Detects 429 responses
//! - Respects Retry-After headers
//! - Implements backoff strategies
//! - Provides appropriate error information
//!
//! Note: The library currently doesn't implement automatic rate limit handling.
//! These tests document the expected behavior and verify error propagation.

mod mock_server;

use mock_server::MockApi;
use spiris::{ApiErrorResponse, Error};

fn api_error(status_code: u16, message: &str) -> Error {
    Error::ApiError {
        status_code,
        response: ApiErrorResponse::from_raw(message.to_string()),
        raw_body: message.to_string(),
    }
}

// =============================================================================
// Rate Limit Response Detection Tests
// =============================================================================

#[tokio::test]
async fn test_rate_limit_returns_429_error() {
    let mut api = MockApi::new().await;

    let _mock = api.mock_rate_limit("/customers", 60);

    let result = api.client.customers().list(None).await;

    assert!(result.is_err(), "Should fail on rate limit");

    match result {
        Err(Error::RateLimitExceeded(_)) => {
            // Expected - dedicated rate limit error
        }
        Err(Error::ApiError { status_code: 429, .. }) => {
            // Also acceptable - generic API error with 429 status
        }
        Err(e) => panic!("Expected rate limit error, got: {:?}", e),
        Ok(_) => panic!("Expected error, got success"),
    }
}

#[tokio::test]
async fn test_rate_limit_on_customer_create() {
    let mut api = MockApi::new().await;

    let _mock = api.mock_error(
        "POST",
        "/customers",
        429,
        r#"{"Message": "Rate limit exceeded"}"#,
    );

    let customer = spiris::Customer {
        name: Some("Test Customer".to_string()),
        ..Default::default()
    };

    let result = api.client.customers().create(&customer).await;

    assert!(result.is_err(), "Should fail on rate limit");
    match result {
        Err(Error::RateLimitExceeded(_)) | Err(Error::ApiError { status_code: 429, .. }) => {}
        other => panic!("Expected rate limit error, got {:?}", other),
    }
}

#[tokio::test]
async fn test_rate_limit_on_invoice_list() {
    let mut api = MockApi::new().await;

    let _mock = api.mock_rate_limit("/customerinvoices", 30);

    let result = api.client.invoices().list(None).await;

    assert!(result.is_err(), "Should fail on rate limit");
}

#[tokio::test]
async fn test_rate_limit_on_article_get() {
    let mut api = MockApi::new().await;

    let _mock = api.mock_error(
        "GET",
        "/articles/art-001",
        429,
        r#"{"Message": "Too Many Requests"}"#,
    );

    let result = api.client.articles().get("art-001").await;

    assert!(result.is_err(), "Should fail on rate limit");
}

// =============================================================================
// Retry-After Header Tests
// =============================================================================

#[tokio::test]
async fn test_rate_limit_with_retry_after_header() {
    let mut api = MockApi::new().await;

    // Mock rate limit with specific Retry-After value
    let _mock = api.mock_rate_limit("/customers", 120);

    let result = api.client.customers().list(None).await;

    // Verify the error is returned
    assert!(result.is_err());

    // Note: Once rate limit handling is implemented, we should verify
    // the Retry-After value is extracted and used
}

#[tokio::test]
async fn test_rate_limit_short_retry_after() {
    let mut api = MockApi::new().await;

    // Very short retry-after (1 second)
    let _mock = api.mock_rate_limit("/customers", 1);

    let result = api.client.customers().list(None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_rate_limit_long_retry_after() {
    let mut api = MockApi::new().await;

    // Long retry-after (10 minutes)
    let _mock = api.mock_rate_limit("/customers", 600);

    let result = api.client.customers().list(None).await;
    assert!(result.is_err());
}

// =============================================================================
// Multiple Endpoint Rate Limiting Tests
// =============================================================================

#[tokio::test]
async fn test_rate_limit_independent_per_endpoint() {
    let mut api = MockApi::new().await;

    // Rate limit customers endpoint
    let _mock_customers = api.mock_rate_limit("/customers", 60);

    // Articles endpoint works fine
    let articles = vec![mock_server::fixtures::article(1)];
    let data = serde_json::to_string(&articles).unwrap();
    let meta = mock_server::meta_json(0, 50, 1, 1);
    let response = format!(r#"{{"Data": {}, {}}}"#, data, meta);
    let _mock_articles = api.mock_get("/articles", &response);

    // Customers should fail
    let customers_result = api.client.customers().list(None).await;
    assert!(customers_result.is_err(), "Customers should be rate limited");

    // Articles should succeed
    let articles_result = api.client.articles().list(None).await;
    assert!(articles_result.is_ok(), "Articles should work normally");
}

#[tokio::test]
async fn test_multiple_requests_during_rate_limit() {
    let mut api = MockApi::new().await;

    let _mock = api.mock_rate_limit("/customers", 60);

    // Multiple requests should all fail during rate limiting
    for _ in 0..3 {
        let result = api.client.customers().list(None).await;
        assert!(result.is_err(), "All requests should fail during rate limit");
    }
}

// =============================================================================
// Rate Limit Error Information Tests
// =============================================================================

#[tokio::test]
async fn test_rate_limit_error_contains_message() {
    let mut api = MockApi::new().await;

    let _mock = api.mock_error(
        "GET",
        "/customers",
        429,
        r#"{"Message": "API rate limit exceeded. Please slow down."}"#,
    );

    let result = api.client.customers().list(None).await;

    match result {
        Err(Error::RateLimitExceeded(msg)) => {
            assert!(
                msg.contains("rate") || msg.contains("Rate") || msg.contains("limit"),
                "Error message should mention rate limiting: {}",
                msg
            );
        }
        Err(Error::ApiError { response, .. }) => {
            assert!(
                response.message.contains("rate")
                    || response.message.contains("Rate")
                    || response.message.contains("limit"),
                "Error message should mention rate limiting: {}",
                response.message
            );
        }
        other => panic!("Expected rate limit error with message, got {:?}", other),
    }
}

// =============================================================================
// Rate Limit Recovery Tests
// =============================================================================

#[tokio::test]
async fn test_recovery_after_rate_limit_sequence() {
    use mock_server::MockResponse;

    let mut api = MockApi::new().await;

    // First request: rate limited
    // Second request: success
    let customer = mock_server::fixtures::customer(1);
    let customer_json = serde_json::to_string(&customer).unwrap();

    let _mocks = api.mock_get_sequence(
        "/customers/cust-001",
        vec![
            MockResponse::rate_limit(5),
            MockResponse::ok(&customer_json),
        ],
    );

    // First call - rate limited
    let result1 = api.client.customers().get("cust-001").await;
    assert!(result1.is_err(), "First call should be rate limited");

    // Second call - success
    let result2 = api.client.customers().get("cust-001").await;
    assert!(result2.is_ok(), "Second call should succeed");
}

#[tokio::test]
async fn test_multiple_rate_limits_then_success() {
    use mock_server::MockResponse;

    let mut api = MockApi::new().await;

    let customer = mock_server::fixtures::customer(1);
    let customer_json = serde_json::to_string(&customer).unwrap();

    // Three rate limits then success
    let _mocks = api.mock_get_sequence(
        "/customers/cust-001",
        vec![
            MockResponse::rate_limit(1),
            MockResponse::rate_limit(1),
            MockResponse::rate_limit(1),
            MockResponse::ok(&customer_json),
        ],
    );

    // First three calls fail
    for i in 1..=3 {
        let result = api.client.customers().get("cust-001").await;
        assert!(result.is_err(), "Call {} should be rate limited", i);
    }

    // Fourth call succeeds
    let result = api.client.customers().get("cust-001").await;
    assert!(result.is_ok(), "Fourth call should succeed");
}

// =============================================================================
// Rate Limit is_retryable Tests
// =============================================================================

#[test]
fn test_rate_limit_error_is_retryable() {
    use spiris::retry::is_retryable_error;

    let error = Error::RateLimitExceeded("Rate limit exceeded".to_string());
    assert!(
        is_retryable_error(&error),
        "Rate limit errors should be retryable"
    );
}

/// Note: Currently 429 as ApiError is NOT retryable because it's a 4xx error.
/// The client should convert 429 responses to RateLimitExceeded error variant,
/// which IS retryable. This test documents the current behavior.
#[test]
fn test_429_api_error_behavior() {
    use spiris::retry::is_retryable_error;

    let error = api_error(429, "Too Many Requests");
    // Currently 429 as ApiError is NOT retryable (it's a 4xx code)
    // The proper fix is for the client to convert 429 to RateLimitExceeded
    assert!(
        !is_retryable_error(&error),
        "429 as ApiError is currently not retryable (should be converted to RateLimitExceeded)"
    );
}

// =============================================================================
// Rate Limit Behavior Documentation Tests
// =============================================================================

/// Documents the expected rate limit behavior once automatic handling is implemented.
///
/// Expected behavior:
/// 1. Client detects 429 response
/// 2. Client extracts Retry-After header value
/// 3. Client waits for specified duration
/// 4. Client retries the request automatically
/// 5. If rate limit persists, repeat with backoff
/// 6. After max retries, return RateLimitExceeded error
///
/// This test currently verifies error propagation.
/// Enable automatic retry verification once the feature is implemented.
#[tokio::test]
async fn test_rate_limit_behavior_documentation() {
    let mut api = MockApi::new().await;

    let _mock = api.mock_rate_limit("/customers", 60);

    let result = api.client.customers().list(None).await;

    // Currently: error is propagated immediately
    assert!(result.is_err());

    // TODO: Once automatic rate limit handling is implemented:
    // - Verify client waits for Retry-After duration
    // - Verify client retries automatically
    // - Verify success after rate limit clears
}

// =============================================================================
// Edge Cases
// =============================================================================

#[tokio::test]
async fn test_rate_limit_on_empty_response() {
    let mut api = MockApi::new().await;

    // 429 with empty body
    let _mock = api.mock_error("GET", "/customers", 429, "");

    let result = api.client.customers().list(None).await;
    assert!(result.is_err(), "Should handle 429 with empty body");
}

#[tokio::test]
async fn test_rate_limit_with_malformed_json() {
    let mut api = MockApi::new().await;

    // 429 with malformed JSON
    let _mock = api.mock_error("GET", "/customers", 429, "not valid json");

    let result = api.client.customers().list(None).await;
    assert!(result.is_err(), "Should handle 429 with malformed body");
}

#[tokio::test]
async fn test_rate_limit_zero_retry_after() {
    let mut api = MockApi::new().await;

    // Edge case: Retry-After: 0
    let _mock = api.mock_rate_limit("/customers", 0);

    let result = api.client.customers().list(None).await;
    assert!(result.is_err(), "Should handle zero retry-after");
}

// =============================================================================
// Future Automatic Rate Limit Handling Tests
// =============================================================================
//
// These tests document the expected behavior once automatic rate limit
// handling is implemented in the client.
//
// #[tokio::test]
// async fn test_automatic_retry_on_rate_limit() {
//     let mut api = MockApi::new().await;
//
//     // Set up: first request rate limited, second succeeds
//     let _mocks = api.mock_sequence(
//         "GET", "/customers",
//         vec![
//             MockResponse::rate_limit(1), // Wait 1 second
//             MockResponse::ok(&customer_list_json),
//         ],
//     );
//
//     let start = Instant::now();
//     let result = api.client.customers().list(None).await;
//     let elapsed = start.elapsed();
//
//     assert!(result.is_ok(), "Should automatically retry and succeed");
//     assert!(elapsed >= Duration::from_secs(1), "Should wait for Retry-After");
// }
//
// #[tokio::test]
// async fn test_rate_limit_max_retries_exceeded() {
//     let mut api = MockApi::new().await;
//
//     // All requests rate limited
//     for _ in 0..10 {
//         api.mock_rate_limit("/customers", 1);
//     }
//
//     let result = api.client.customers().list(None).await;
//
//     match result {
//         Err(Error::RateLimitExceeded { retry_count, .. }) => {
//             assert!(retry_count >= 3, "Should have retried multiple times");
//         }
//         _ => panic!("Expected rate limit exceeded error after max retries"),
//     }
// }
