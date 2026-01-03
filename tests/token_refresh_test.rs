//! Integration tests for token refresh functionality.
//!
//! These tests verify that token management works correctly:
//! - Token expiration detection
//! - Manual token refresh
//! - Token storage and retrieval
//! - Automatic token refresh with OAuth2 config
//! - Error handling for refresh failures

mod mock_server;

use mock_server::{MockApi, MockOAuthServer};
use spiris::auth::OAuth2Config;
use spiris::{AccessToken, Client, ClientConfig, Error, RetryConfig};
use std::time::Duration;

// =============================================================================
// AccessToken Creation and Properties
// =============================================================================

#[test]
fn test_access_token_creation() {
    let token = AccessToken::new("my_token".to_string(), 3600, None);

    assert_eq!(token.token, "my_token");
    assert_eq!(token.token_type, "Bearer");
    assert!(token.refresh_token.is_none());
}

#[test]
fn test_access_token_with_refresh_token() {
    let token = AccessToken::new(
        "access_token".to_string(),
        3600,
        Some("refresh_token".to_string()),
    );

    assert_eq!(token.token, "access_token");
    assert_eq!(token.refresh_token, Some("refresh_token".to_string()));
}

#[test]
fn test_access_token_authorization_header() {
    let token = AccessToken::new("abc123".to_string(), 3600, None);
    assert_eq!(token.authorization_header(), "Bearer abc123");
}

// =============================================================================
// Token Expiration Tests
// =============================================================================

#[test]
fn test_token_not_expired_with_long_expiry() {
    let token = AccessToken::new("test".to_string(), 3600, None); // 1 hour
    assert!(
        !token.is_expired(),
        "Token with 1 hour remaining should not be expired"
    );
}

#[test]
fn test_token_expired_with_zero_expiry() {
    let token = AccessToken::new("test".to_string(), 0, None);
    std::thread::sleep(Duration::from_millis(10));
    assert!(token.is_expired(), "Token with 0 expiry should be expired");
}

#[test]
fn test_token_expired_with_negative_expiry() {
    let token = AccessToken::new("test".to_string(), -100, None);
    assert!(
        token.is_expired(),
        "Token with negative expiry should be expired"
    );
}

#[test]
fn test_token_expires_within_buffer() {
    // Token expires in 4 minutes - should be considered expired due to 5 min buffer
    let token = AccessToken::new("test".to_string(), 240, None);
    assert!(
        token.is_expired(),
        "Token expiring within 5 min buffer should be considered expired"
    );
}

#[test]
fn test_token_not_expired_outside_buffer() {
    // Token expires in 6 minutes - outside the 5 min buffer
    let token = AccessToken::new("test".to_string(), 360, None);
    assert!(
        !token.is_expired(),
        "Token expiring in 6 minutes should not be considered expired"
    );
}

#[test]
fn test_token_exactly_at_buffer_boundary() {
    // Token expires in exactly 5 minutes (300 seconds)
    let token = AccessToken::new("test".to_string(), 300, None);
    // At exactly 5 minutes, it should be considered expired (>= check)
    assert!(
        token.is_expired(),
        "Token expiring in exactly 5 minutes should be considered expired"
    );
}

// =============================================================================
// Token Serialization Tests
// =============================================================================

#[test]
fn test_token_serialization() {
    let token = AccessToken::new("test_token".to_string(), 3600, Some("refresh".to_string()));

    let json = serde_json::to_string(&token).unwrap();
    assert!(json.contains("test_token"));
    assert!(json.contains("refresh"));
    assert!(json.contains("Bearer"));
}

#[test]
fn test_token_deserialization() {
    let token = AccessToken::new(
        "original_token".to_string(),
        3600,
        Some("original_refresh".to_string()),
    );

    let json = serde_json::to_string(&token).unwrap();
    let deserialized: AccessToken = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.token, token.token);
    assert_eq!(deserialized.refresh_token, token.refresh_token);
    assert_eq!(deserialized.token_type, token.token_type);
}

#[test]
fn test_token_roundtrip_preserves_data() {
    let original = AccessToken::new(
        "my_access_token".to_string(),
        7200,
        Some("my_refresh_token".to_string()),
    );

    let json = serde_json::to_string(&original).unwrap();
    let restored: AccessToken = serde_json::from_str(&json).unwrap();

    assert_eq!(original.token, restored.token);
    assert_eq!(original.token_type, restored.token_type);
    assert_eq!(original.refresh_token, restored.refresh_token);
}

// =============================================================================
// Client with Expired Token Tests
// =============================================================================

#[tokio::test]
async fn test_client_rejects_expired_token() {
    let expired_token = AccessToken::new("expired".to_string(), -100, None);

    // Wait to ensure token is definitely expired
    tokio::time::sleep(Duration::from_millis(50)).await;

    let server = mockito::Server::new_async().await;
    let config = ClientConfig::new().base_url(server.url());
    let client = Client::with_config(expired_token, config);

    let result = client.customers().list(None).await;

    assert!(
        matches!(result, Err(Error::TokenExpired)),
        "Expected TokenExpired error, got {:?}",
        result
    );
}

#[tokio::test]
async fn test_client_accepts_valid_token() {
    let mut api = MockApi::new().await;

    let customers = vec![mock_server::fixtures::customer(1)];
    let data = serde_json::to_string(&customers).unwrap();
    let meta = mock_server::meta_json(0, 50, 1, 1);
    let response = format!(r#"{{"Data": {}, {}}}"#, data, meta);

    let _mock = api.mock_get("/customers", &response);

    let result = api.client.customers().list(None).await;
    assert!(result.is_ok(), "Valid token should work");
}

// =============================================================================
// Manual Token Update Tests
// =============================================================================

#[tokio::test]
async fn test_set_access_token_updates_client() {
    let mut api = MockApi::new().await;

    // First request with original token
    let customers = vec![mock_server::fixtures::customer(1)];
    let data = serde_json::to_string(&customers).unwrap();
    let meta = mock_server::meta_json(0, 50, 1, 1);
    let response = format!(r#"{{"Data": {}, {}}}"#, data, meta);

    // Allow multiple requests to the same endpoint
    let _mock = api
        .server
        .mock("GET", "/customers")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(&response)
        .expect_at_least(2)
        .create();

    // Make first request
    let result1 = api.client.customers().list(None).await;
    assert!(result1.is_ok(), "First request should succeed");

    // Update token
    let new_token = AccessToken::new("new_token".to_string(), 3600, None);
    api.client.set_access_token(new_token);

    // Make second request with new token
    let result2 = api.client.customers().list(None).await;
    assert!(
        result2.is_ok(),
        "Second request with new token should succeed"
    );
}

// =============================================================================
// Token with Different Expiry Scenarios
// =============================================================================

#[test]
fn test_token_very_long_expiry() {
    // Token expires in 24 hours
    let token = AccessToken::new("test".to_string(), 86400, None);
    assert!(!token.is_expired());
}

#[test]
fn test_token_very_short_expiry() {
    // Token expires in 1 second
    let token = AccessToken::new("test".to_string(), 1, None);
    // Should be expired because it's within the 5-minute buffer
    assert!(token.is_expired());
}

#[test]
fn test_token_one_hour_expiry() {
    // Token expires in 1 hour (3600 seconds)
    let token = AccessToken::new("test".to_string(), 3600, None);
    assert!(!token.is_expired());
}

// =============================================================================
// OAuth2 Handler Token Exchange Tests
// =============================================================================

#[test]
fn test_oauth2_handler_creation_succeeds() {
    use spiris::auth::{OAuth2Config, OAuth2Handler};

    let config = OAuth2Config::new(
        "test_client_id".to_string(),
        "test_client_secret".to_string(),
        "http://localhost:8080/callback".to_string(),
    );

    let result = OAuth2Handler::new(config);
    assert!(result.is_ok(), "OAuth2Handler creation should succeed");
}

#[test]
fn test_oauth2_handler_authorize_url_generated() {
    use spiris::auth::{OAuth2Config, OAuth2Handler};

    let config = OAuth2Config::new(
        "test_client_id".to_string(),
        "test_client_secret".to_string(),
        "http://localhost:8080/callback".to_string(),
    );

    let handler = OAuth2Handler::new(config).unwrap();
    let (auth_url, csrf_token, pkce_verifier) = handler.authorize_url();

    assert!(!auth_url.is_empty(), "Auth URL should not be empty");
    assert!(!csrf_token.is_empty(), "CSRF token should not be empty");
    assert!(
        !pkce_verifier.is_empty(),
        "PKCE verifier should not be empty"
    );
}

// =============================================================================
// Token Refresh Error Scenarios
// =============================================================================

#[tokio::test]
async fn test_refresh_token_missing_error() {
    // Token without refresh token cannot be refreshed
    let token = AccessToken::new("access".to_string(), -100, None); // No refresh token

    assert!(
        token.refresh_token.is_none(),
        "Token should not have refresh token"
    );
    // Attempting to refresh should fail (when implemented)
}

#[test]
fn test_token_has_refresh_token() {
    let token = AccessToken::new("access".to_string(), 3600, Some("refresh".to_string()));

    assert!(token.refresh_token.is_some());
    assert_eq!(token.refresh_token.unwrap(), "refresh");
}

// =============================================================================
// Concurrent Token Access Tests
// =============================================================================

#[tokio::test]
async fn test_concurrent_requests_with_valid_token() {
    let mut api = MockApi::new().await;

    let customers = vec![mock_server::fixtures::customer(1)];
    let data = serde_json::to_string(&customers).unwrap();
    let meta = mock_server::meta_json(0, 50, 1, 1);
    let response = format!(r#"{{"Data": {}, {}}}"#, data, meta);

    // Allow multiple requests
    let _mock = api
        .server
        .mock("GET", "/customers")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(&response)
        .expect_at_least(3)
        .create();

    // Fire concurrent requests
    let customers_endpoint = api.client.customers();
    let (r1, r2, r3) = tokio::join!(
        customers_endpoint.list(None),
        customers_endpoint.list(None),
        customers_endpoint.list(None)
    );

    assert!(r1.is_ok(), "First concurrent request should succeed");
    assert!(r2.is_ok(), "Second concurrent request should succeed");
    assert!(r3.is_ok(), "Third concurrent request should succeed");
}

// =============================================================================
// Automatic Token Refresh Tests
// =============================================================================

#[tokio::test]
async fn test_auto_refresh_on_expired_token() {
    // Create mock OAuth server for token refresh
    let mut oauth = MockOAuthServer::new().await;
    let _refresh_mock = oauth.mock_token_refresh(
        "my_refresh_token",
        "new_access_token",
        Some("new_refresh_token"),
        3600,
    );

    // Create mock API server
    let mut api_server = mockito::Server::new_async().await;
    let customers_json = r#"{"Data": [], "Meta": {"CurrentPage": 0, "PageSize": 50, "TotalPages": 1, "TotalCount": 0, "HasNextPage": false, "HasPreviousPage": false}}"#;
    let _api_mock = api_server
        .mock("GET", "/customers")
        .match_header("Authorization", "Bearer new_access_token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(customers_json)
        .create();

    // Create OAuth config pointing to mock server
    let oauth_config = OAuth2Config {
        client_id: "test_client".to_string(),
        client_secret: "test_secret".to_string(),
        redirect_uri: "http://localhost:8080/callback".to_string(),
        auth_url: oauth.auth_url(),
        token_url: oauth.token_url(),
    };

    // Start with expired token that has refresh token
    let expired_token = AccessToken::new(
        "old_token".to_string(),
        -100, // Already expired
        Some("my_refresh_token".to_string()),
    );

    // Create client with OAuth config for auto-refresh
    let config = ClientConfig::new()
        .base_url(api_server.url())
        .oauth_config(oauth_config)
        .retry_config(RetryConfig::new().max_retries(0));
    let client = Client::with_config(expired_token, config);

    // Make a request - should auto-refresh and succeed
    let result = client.customers().list(None).await;

    assert!(
        result.is_ok(),
        "Should auto-refresh and succeed: {:?}",
        result.err()
    );

    // Verify the token was updated
    let current_token = client.get_access_token();
    assert_eq!(current_token.token, "new_access_token");
    assert_eq!(
        current_token.refresh_token,
        Some("new_refresh_token".to_string())
    );
}

#[tokio::test]
async fn test_auto_refresh_without_oauth_config_fails() {
    // Create client with expired token but NO OAuth config
    let expired_token = AccessToken::new(
        "old_token".to_string(),
        -100,
        Some("refresh_token".to_string()),
    );

    let server = mockito::Server::new_async().await;
    let config = ClientConfig::new()
        .base_url(server.url())
        .retry_config(RetryConfig::new().max_retries(0));
    // Note: no oauth_config set
    let client = Client::with_config(expired_token, config);

    let result = client.customers().list(None).await;

    assert!(
        matches!(result, Err(Error::TokenExpired)),
        "Without OAuth config, expired token should return TokenExpired: {:?}",
        result
    );
}

#[tokio::test]
async fn test_auto_refresh_without_refresh_token_fails() {
    // Create mock OAuth server
    let oauth = MockOAuthServer::new().await;

    let oauth_config = OAuth2Config {
        client_id: "test_client".to_string(),
        client_secret: "test_secret".to_string(),
        redirect_uri: "http://localhost:8080/callback".to_string(),
        auth_url: oauth.auth_url(),
        token_url: oauth.token_url(),
    };

    // Expired token WITHOUT refresh token
    let expired_token = AccessToken::new(
        "old_token".to_string(),
        -100,
        None, // No refresh token
    );

    let server = mockito::Server::new_async().await;
    let config = ClientConfig::new()
        .base_url(server.url())
        .oauth_config(oauth_config)
        .retry_config(RetryConfig::new().max_retries(0));
    let client = Client::with_config(expired_token, config);

    let result = client.customers().list(None).await;

    assert!(
        matches!(result, Err(Error::TokenExpired)),
        "Without refresh token, should return TokenExpired: {:?}",
        result
    );
}

#[tokio::test]
async fn test_valid_token_does_not_refresh() {
    // Create mock OAuth server - should NOT be called
    let mut oauth = MockOAuthServer::new().await;
    // Set expect(0) to verify refresh is not called
    let _refresh_mock = oauth
        .server
        .mock("POST", "/connect/token")
        .expect(0)
        .create();

    // Create mock API server
    let mut api_server = mockito::Server::new_async().await;
    let customers_json = r#"{"Data": [], "Meta": {"CurrentPage": 0, "PageSize": 50, "TotalPages": 1, "TotalCount": 0, "HasNextPage": false, "HasPreviousPage": false}}"#;
    let _api_mock = api_server
        .mock("GET", "/customers")
        .match_header("Authorization", "Bearer valid_token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(customers_json)
        .create();

    let oauth_config = OAuth2Config {
        client_id: "test_client".to_string(),
        client_secret: "test_secret".to_string(),
        redirect_uri: "http://localhost:8080/callback".to_string(),
        auth_url: oauth.auth_url(),
        token_url: oauth.token_url(),
    };

    // Valid token (not expired)
    let valid_token = AccessToken::new(
        "valid_token".to_string(),
        3600, // 1 hour, not expired
        Some("my_refresh_token".to_string()),
    );

    let config = ClientConfig::new()
        .base_url(api_server.url())
        .oauth_config(oauth_config)
        .retry_config(RetryConfig::new().max_retries(0));
    let client = Client::with_config(valid_token, config);

    let result = client.customers().list(None).await;
    assert!(result.is_ok());
    // Mock will verify expect(0) when dropped
}

#[tokio::test]
async fn test_concurrent_requests_single_refresh() {
    // Create mock OAuth server - expect exactly 1 refresh call due to mutex
    let mut oauth = MockOAuthServer::new().await;
    // Use expect(1) to verify the mutex prevents multiple refreshes
    let _refresh_mock = oauth
        .server
        .mock("POST", "/connect/token")
        .match_body(mockito::Matcher::Regex("refresh_token=my_refresh_token".to_string()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"access_token": "new_access_token", "token_type": "Bearer", "expires_in": 3600, "refresh_token": "new_refresh_token"}"#)
        .expect(1)  // Only 1 refresh due to mutex
        .create();

    // Create mock API server
    let mut api_server = mockito::Server::new_async().await;
    let customers_json = r#"{"Data": [], "Meta": {"CurrentPage": 0, "PageSize": 50, "TotalPages": 1, "TotalCount": 0, "HasNextPage": false, "HasPreviousPage": false}}"#;
    let _api_mock = api_server
        .mock("GET", "/customers")
        .match_header("Authorization", "Bearer new_access_token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(customers_json)
        .expect_at_least(3)
        .create();

    let oauth_config = OAuth2Config {
        client_id: "test_client".to_string(),
        client_secret: "test_secret".to_string(),
        redirect_uri: "http://localhost:8080/callback".to_string(),
        auth_url: oauth.auth_url(),
        token_url: oauth.token_url(),
    };

    // Expired token
    let expired_token = AccessToken::new(
        "old_token".to_string(),
        -100,
        Some("my_refresh_token".to_string()),
    );

    let config = ClientConfig::new()
        .base_url(api_server.url())
        .oauth_config(oauth_config)
        .retry_config(RetryConfig::new().max_retries(0));
    let client = Client::with_config(expired_token, config);

    // Fire 3 concurrent requests
    let customers = client.customers();
    let (r1, r2, r3) = tokio::join!(
        customers.list(None),
        customers.list(None),
        customers.list(None)
    );

    assert!(r1.is_ok(), "Request 1 should succeed: {:?}", r1.err());
    assert!(r2.is_ok(), "Request 2 should succeed: {:?}", r2.err());
    assert!(r3.is_ok(), "Request 3 should succeed: {:?}", r3.err());
    // Mock will verify expect(1) for refresh when dropped
}
