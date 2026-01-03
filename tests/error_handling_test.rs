//! Integration tests for error handling scenarios.

mod mock_server;

use mock_server::MockApi;
use spiris::{AccessToken, ApiErrorResponse, Client, ClientConfig, Customer, Error};

fn api_error(status_code: u16, message: &str) -> Error {
    Error::ApiError {
        status_code,
        response: ApiErrorResponse::from_raw(message.to_string()),
        raw_body: message.to_string(),
    }
}

#[tokio::test]
async fn test_not_found_error() {
    let mut api = MockApi::new().await;

    let mock = api.mock_error(
        "GET",
        "/customers/nonexistent",
        404,
        r#"{"Message": "Customer not found"}"#,
    );

    let result = api.client.customers().get("nonexistent").await;

    mock.assert();
    assert!(matches!(result, Err(Error::NotFound(_))));
    if let Err(Error::NotFound(msg)) = result {
        assert!(msg.contains("Customer not found"));
    }
}

#[tokio::test]
async fn test_unauthorized_error() {
    let mut api = MockApi::new().await;

    let mock = api
        .server
        .mock("GET", "/customers")
        .with_status(401)
        .with_body(r#"{"Message": "Invalid token"}"#)
        .create();

    let result = api.client.customers().list(None).await;

    mock.assert();
    assert!(matches!(result, Err(Error::AuthError(_))));
}

#[tokio::test]
async fn test_forbidden_error() {
    let mut api = MockApi::new().await;

    let mock = api
        .server
        .mock("GET", "/customers")
        .with_status(403)
        .with_body(r#"{"Message": "Access denied"}"#)
        .create();

    let result = api.client.customers().list(None).await;

    mock.assert();
    assert!(matches!(result, Err(Error::AuthError(_))));
}

#[tokio::test]
async fn test_rate_limit_error() {
    let mut api = MockApi::new().await;

    let mock = api.mock_error(
        "GET",
        "/customers",
        429,
        r#"{"Message": "Rate limit exceeded. Try again later."}"#,
    );

    let result = api.client.customers().list(None).await;

    mock.assert();
    assert!(matches!(result, Err(Error::RateLimitExceeded(_))));
    if let Err(Error::RateLimitExceeded(msg)) = result {
        assert!(msg.contains("Rate limit"));
    }
}

#[tokio::test]
async fn test_bad_request_error() {
    let mut api = MockApi::new().await;

    let mock = api.mock_error(
        "POST",
        "/customers",
        400,
        r#"{"Message": "Invalid customer data: Name is required"}"#,
    );

    let invalid_customer = Customer::default();
    let result = api.client.customers().create(&invalid_customer).await;

    mock.assert();
    assert!(matches!(result, Err(Error::InvalidRequest(_))));
    if let Err(Error::InvalidRequest(msg)) = result {
        assert!(msg.contains("Name is required"));
    }
}

#[tokio::test]
async fn test_server_error() {
    let mut api = MockApi::new().await;

    let mock = api.mock_error(
        "GET",
        "/customers",
        500,
        r#"{"Message": "Internal server error"}"#,
    );

    let result = api.client.customers().list(None).await;

    mock.assert();
    assert!(matches!(
        result,
        Err(Error::ApiError {
            status_code: 500,
            ..
        })
    ));
}

#[tokio::test]
async fn test_service_unavailable_error() {
    let mut api = MockApi::new().await;

    let mock = api.mock_error(
        "GET",
        "/customers",
        503,
        r#"{"Message": "Service temporarily unavailable"}"#,
    );

    let result = api.client.customers().list(None).await;

    mock.assert();
    assert!(matches!(
        result,
        Err(Error::ApiError {
            status_code: 503,
            ..
        })
    ));
}

#[tokio::test]
async fn test_expired_token_error() {
    // Create a client with an already expired token
    let expired_token = AccessToken::new("expired_token".to_string(), 0, None);

    // Wait a bit to ensure token is expired
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let server = mockito::Server::new_async().await;
    let config = ClientConfig::new().base_url(server.url());
    let client = Client::with_config(expired_token, config);

    let result = client.customers().list(None).await;

    assert!(matches!(result, Err(Error::TokenExpired)));
}

#[tokio::test]
async fn test_delete_not_found_error() {
    let mut api = MockApi::new().await;

    let mock = api.mock_error(
        "DELETE",
        "/customers/nonexistent",
        404,
        r#"{"Message": "Customer not found"}"#,
    );

    let result = api.client.customers().delete("nonexistent").await;

    mock.assert();
    assert!(matches!(result, Err(Error::NotFound(_))));
}

#[tokio::test]
async fn test_update_conflict_error() {
    let mut api = MockApi::new().await;

    let mock = api.mock_error(
        "PUT",
        "/customers/cust-123",
        409,
        r#"{"Message": "Conflict: Resource was modified by another request"}"#,
    );

    let customer = Customer {
        id: Some("cust-123".to_string()),
        name: Some("Updated".to_string()),
        ..Default::default()
    };
    let result = api.client.customers().update("cust-123", &customer).await;

    mock.assert();
    assert!(matches!(
        result,
        Err(Error::ApiError {
            status_code: 409,
            ..
        })
    ));
}

#[tokio::test]
async fn test_validation_error_details() {
    let mut api = MockApi::new().await;

    let error_response = r#"{
        "Message": "Validation failed",
        "Errors": [
            {"Field": "Email", "Message": "Invalid email format"},
            {"Field": "Phone", "Message": "Invalid phone number"}
        ]
    }"#;

    let mock = api.mock_error("POST", "/customers", 400, error_response);

    let invalid_customer = Customer {
        email: Some("invalid-email".to_string()),
        phone: Some("not-a-phone".to_string()),
        ..Default::default()
    };
    let result = api.client.customers().create(&invalid_customer).await;

    mock.assert();
    assert!(matches!(result, Err(Error::InvalidRequest(_))));
    if let Err(Error::InvalidRequest(msg)) = result {
        assert!(msg.contains("Validation failed"));
    }
}

#[tokio::test]
async fn test_empty_response_body_error() {
    let mut api = MockApi::new().await;

    let mock = api.mock_error("GET", "/customers/empty", 404, "");

    let result = api.client.customers().get("empty").await;

    mock.assert();
    assert!(matches!(result, Err(Error::NotFound(_))));
}

// =============================================================================
// Additional HTTP Status Code Tests
// =============================================================================

#[tokio::test]
async fn test_502_bad_gateway() {
    let mut api = MockApi::new().await;

    let _mock = api.mock_error("GET", "/customers", 502, r#"{"Message": "Bad Gateway"}"#);

    let result = api.client.customers().list(None).await;

    assert!(matches!(
        result,
        Err(Error::ApiError {
            status_code: 502,
            ..
        })
    ));
}

#[tokio::test]
async fn test_504_gateway_timeout() {
    let mut api = MockApi::new().await;

    let _mock = api.mock_error(
        "GET",
        "/customers",
        504,
        r#"{"Message": "Gateway timeout"}"#,
    );

    let result = api.client.customers().list(None).await;

    assert!(matches!(
        result,
        Err(Error::ApiError {
            status_code: 504,
            ..
        })
    ));
}

#[tokio::test]
async fn test_422_unprocessable_entity() {
    let mut api = MockApi::new().await;

    let _mock = api.mock_error(
        "POST",
        "/customers",
        422,
        r#"{"Message": "Unprocessable entity"}"#,
    );

    let customer = Customer::default();
    let result = api.client.customers().create(&customer).await;

    // 422 may be treated as InvalidRequest or ApiError
    assert!(result.is_err());
}

// =============================================================================
// Error Retryability Tests
// =============================================================================

// Note: Can't easily create Error::Http in tests as it wraps reqwest::Error
// The is_retryable_error behavior for Http is tested via the retry_test.rs module

#[test]
fn test_rate_limit_error_is_retryable() {
    use spiris::retry::is_retryable_error;

    let error = Error::RateLimitExceeded("Rate limit exceeded".to_string());
    assert!(
        is_retryable_error(&error),
        "Rate limit errors should be retryable"
    );
}

#[test]
fn test_5xx_errors_are_retryable() {
    use spiris::retry::is_retryable_error;

    for status in [500, 502, 503, 504] {
        let error = api_error(status, "Server error");
        assert!(
            is_retryable_error(&error),
            "{} errors should be retryable",
            status
        );
    }
}

#[test]
fn test_4xx_errors_not_retryable() {
    use spiris::retry::is_retryable_error;

    for status in [400, 401, 403, 404, 409, 422] {
        let error = api_error(status, "Client error");
        assert!(
            !is_retryable_error(&error),
            "{} errors should NOT be retryable",
            status
        );
    }
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

#[test]
fn test_not_found_not_retryable() {
    use spiris::retry::is_retryable_error;

    let error = Error::NotFound("Resource not found".to_string());
    assert!(
        !is_retryable_error(&error),
        "NotFound should NOT be retryable"
    );
}

// =============================================================================
// Error Display Tests
// =============================================================================

#[test]
fn test_api_error_display() {
    let error = api_error(400, "Bad request");

    let display = format!("{}", error);
    assert!(
        display.contains("400") || display.contains("Bad request"),
        "Display should include status or message: {}",
        display
    );
}

#[test]
fn test_not_found_display() {
    let error = Error::NotFound("Customer xyz not found".to_string());

    let display = format!("{}", error);
    assert!(
        display.contains("xyz") || display.contains("not found"),
        "Display should include resource info: {}",
        display
    );
}

#[test]
fn test_rate_limit_display() {
    let error = Error::RateLimitExceeded("Too many requests".to_string());

    let display = format!("{}", error);
    assert!(
        display.contains("rate") || display.contains("limit") || display.contains("Too many"),
        "Display should indicate rate limiting: {}",
        display
    );
}

#[test]
fn test_token_expired_display() {
    let error = Error::TokenExpired;

    let display = format!("{}", error);
    assert!(
        display.to_lowercase().contains("token")
            || display.to_lowercase().contains("expired")
            || display.to_lowercase().contains("auth"),
        "Display should indicate token issue: {}",
        display
    );
}

// =============================================================================
// Error Trait Implementation Tests
// =============================================================================

#[test]
fn test_error_implements_std_error() {
    fn assert_std_error<T: std::error::Error>() {}
    assert_std_error::<Error>();
}

#[test]
fn test_error_is_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<Error>();
    assert_sync::<Error>();
}

// =============================================================================
// Error Response Format Tests
// =============================================================================

#[tokio::test]
async fn test_error_with_plain_text_body() {
    let mut api = MockApi::new().await;

    let _mock = api.mock_error("GET", "/customers", 500, "Plain text error message");

    let result = api.client.customers().list(None).await;

    assert!(result.is_err(), "Should handle plain text error body");
}

#[tokio::test]
async fn test_error_with_html_body() {
    let mut api = MockApi::new().await;

    let _mock = api.mock_error(
        "GET",
        "/customers",
        500,
        "<html><body><h1>500 Internal Server Error</h1></body></html>",
    );

    let result = api.client.customers().list(None).await;

    assert!(result.is_err(), "Should handle HTML error body");
}

#[tokio::test]
async fn test_error_with_malformed_json() {
    let mut api = MockApi::new().await;

    let _mock = api.mock_error("GET", "/customers", 400, "{invalid json}");

    let result = api.client.customers().list(None).await;

    assert!(result.is_err(), "Should handle malformed JSON error body");
}

// =============================================================================
// Endpoint-Specific Error Tests
// =============================================================================

#[tokio::test]
async fn test_invoice_not_found() {
    let mut api = MockApi::new().await;

    let _mock = api.mock_error(
        "GET",
        "/customerinvoices/inv-nonexistent",
        404,
        r#"{"Message": "Invoice not found"}"#,
    );

    let result = api.client.invoices().get("inv-nonexistent").await;

    assert!(matches!(
        result,
        Err(Error::NotFound(_)) | Err(Error::ApiError { status_code: 404, .. })
    ));
}

#[tokio::test]
async fn test_article_not_found() {
    let mut api = MockApi::new().await;

    let _mock = api.mock_error(
        "GET",
        "/articles/art-nonexistent",
        404,
        r#"{"Message": "Article not found"}"#,
    );

    let result = api.client.articles().get("art-nonexistent").await;

    assert!(matches!(
        result,
        Err(Error::NotFound(_)) | Err(Error::ApiError { status_code: 404, .. })
    ));
}

// =============================================================================
// Error Recovery Tests
// =============================================================================

#[tokio::test]
async fn test_success_after_error() {
    use mock_server::MockResponse;

    let mut api = MockApi::new().await;

    let customer = mock_server::fixtures::customer(1);
    let customer_json = serde_json::to_string(&customer).unwrap();

    let _mocks = api.mock_get_sequence(
        "/customers/cust-001",
        vec![
            MockResponse::error(500, "Temporary error"),
            MockResponse::ok(&customer_json),
        ],
    );

    // First call fails
    let result1 = api.client.customers().get("cust-001").await;
    assert!(result1.is_err());

    // Second call succeeds
    let result2 = api.client.customers().get("cust-001").await;
    assert!(result2.is_ok());
}

#[tokio::test]
async fn test_different_errors_in_sequence() {
    use mock_server::MockResponse;

    let mut api = MockApi::new().await;

    let _mocks = api.mock_get_sequence(
        "/customers/cust-001",
        vec![
            MockResponse::error(500, "Server error"),
            MockResponse::error(503, "Service unavailable"),
            MockResponse::error(502, "Bad gateway"),
        ],
    );

    // Each call returns different error
    let result1 = api.client.customers().get("cust-001").await;
    if let Err(Error::ApiError { status_code, .. }) = result1 {
        assert_eq!(status_code, 500);
    }

    let result2 = api.client.customers().get("cust-001").await;
    if let Err(Error::ApiError { status_code, .. }) = result2 {
        assert_eq!(status_code, 503);
    }

    let result3 = api.client.customers().get("cust-001").await;
    if let Err(Error::ApiError { status_code, .. }) = result3 {
        assert_eq!(status_code, 502);
    }
}

// =============================================================================
// Concurrent Error Handling Tests
// =============================================================================

#[tokio::test]
async fn test_concurrent_errors() {
    let mut api = MockApi::new().await;

    // Set up different errors for different endpoints
    let _mock1 = api.mock_error("GET", "/customers", 500, r#"{"Message": "Error 1"}"#);
    let _mock2 = api.mock_error("GET", "/articles", 503, r#"{"Message": "Error 2"}"#);

    // Make concurrent requests - need to bind endpoints to avoid temporary lifetime issues
    let customers = api.client.customers();
    let articles = api.client.articles();
    let (result1, result2) = tokio::join!(customers.list(None), articles.list(None));

    assert!(result1.is_err());
    assert!(result2.is_err());

    // Verify different status codes
    if let (
        Err(Error::ApiError {
            status_code: s1, ..
        }),
        Err(Error::ApiError {
            status_code: s2, ..
        }),
    ) = (&result1, &result2)
    {
        assert_eq!(*s1, 500);
        assert_eq!(*s2, 503);
    }
}
