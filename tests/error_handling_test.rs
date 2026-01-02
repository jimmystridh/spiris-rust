//! Integration tests for error handling scenarios.

mod mock_server;

use mock_server::MockApi;
use spiris_bokforing::{AccessToken, Client, ClientConfig, Customer, Error};

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
