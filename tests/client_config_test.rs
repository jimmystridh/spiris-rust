//! Tests for client configuration and initialization.
//!
//! These tests verify:
//! - ClientConfig builder pattern
//! - Client creation with various configurations
//! - URL building and base URL handling
//! - Token management (get/set/expiration)
//! - Request building and header handling
//! - Thread safety (Send + Sync)

mod mock_server;

use mock_server::MockApi;
use spiris::client::{ClientConfig, DEFAULT_BASE_URL, RATE_LIMIT_PER_MINUTE};
use spiris::retry::RetryConfig;
use spiris::{AccessToken, Client, Error};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

// =============================================================================
// Constants Tests
// =============================================================================

#[test]
fn test_default_base_url() {
    assert_eq!(
        DEFAULT_BASE_URL,
        "https://eaccountingapi.vismaonline.com/v2/"
    );
}

#[test]
fn test_rate_limit_constant() {
    assert_eq!(RATE_LIMIT_PER_MINUTE, 600);
}

// =============================================================================
// ClientConfig Default Tests
// =============================================================================

#[test]
fn test_client_config_default() {
    let config = ClientConfig::default();

    assert_eq!(config.base_url, DEFAULT_BASE_URL);
    assert!(config.user_agent.contains("spiris-bokforing-rust"));
    assert_eq!(config.timeout_seconds, 30);
    assert!(config.enable_tracing);
}

#[test]
fn test_client_config_new_equals_default() {
    let config1 = ClientConfig::new();
    let config2 = ClientConfig::default();

    assert_eq!(config1.base_url, config2.base_url);
    assert_eq!(config1.user_agent, config2.user_agent);
    assert_eq!(config1.timeout_seconds, config2.timeout_seconds);
    assert_eq!(config1.enable_tracing, config2.enable_tracing);
}

#[test]
fn test_client_config_user_agent_contains_version() {
    let config = ClientConfig::new();

    // Should contain the package version from Cargo.toml
    assert!(config.user_agent.starts_with("spiris-bokforing-rust/"));
}

// =============================================================================
// ClientConfig Builder Pattern Tests
// =============================================================================

#[test]
fn test_client_config_base_url_builder() {
    let config = ClientConfig::new().base_url("https://custom.api.com/v1/");

    assert_eq!(config.base_url, "https://custom.api.com/v1/");
}

#[test]
fn test_client_config_timeout_builder() {
    let config = ClientConfig::new().timeout_seconds(60);

    assert_eq!(config.timeout_seconds, 60);
}

#[test]
fn test_client_config_tracing_builder() {
    let config = ClientConfig::new().enable_tracing(false);

    assert!(!config.enable_tracing);
}

#[test]
fn test_client_config_retry_config_builder() {
    let retry = RetryConfig::default().max_retries(5);
    let config = ClientConfig::new().retry_config(retry);

    assert_eq!(config.retry_config.max_retries, 5);
}

#[test]
fn test_client_config_chained_builders() {
    let config = ClientConfig::new()
        .base_url("https://test.api.com/")
        .timeout_seconds(120)
        .enable_tracing(false)
        .retry_config(RetryConfig::default().max_retries(10));

    assert_eq!(config.base_url, "https://test.api.com/");
    assert_eq!(config.timeout_seconds, 120);
    assert!(!config.enable_tracing);
    assert_eq!(config.retry_config.max_retries, 10);
}

#[test]
fn test_client_config_base_url_accepts_string() {
    let config = ClientConfig::new().base_url(String::from("https://example.com/"));

    assert_eq!(config.base_url, "https://example.com/");
}

#[test]
fn test_client_config_base_url_accepts_str() {
    let config = ClientConfig::new().base_url("https://example.com/");

    assert_eq!(config.base_url, "https://example.com/");
}

// =============================================================================
// ClientConfig Clone and Debug Tests
// =============================================================================

#[test]
fn test_client_config_clone() {
    let config = ClientConfig::new()
        .base_url("https://test.com/")
        .timeout_seconds(45);

    let cloned = config.clone();

    assert_eq!(cloned.base_url, "https://test.com/");
    assert_eq!(cloned.timeout_seconds, 45);
}

#[test]
fn test_client_config_debug() {
    let config = ClientConfig::new();
    let debug_str = format!("{:?}", config);

    assert!(debug_str.contains("ClientConfig"));
    assert!(debug_str.contains("base_url"));
}

// =============================================================================
// Client Creation Tests
// =============================================================================

#[test]
fn test_client_new_with_default_config() {
    let token = AccessToken::new("test_token".to_string(), 3600, None);
    let client = Client::new(token);

    // Client should be created successfully
    let retrieved_token = client.get_access_token();
    assert_eq!(retrieved_token.token, "test_token");
}

#[test]
fn test_client_with_config() {
    let token = AccessToken::new("test_token".to_string(), 3600, None);
    let config = ClientConfig::new()
        .base_url("https://custom.api.com/")
        .timeout_seconds(60);

    let client = Client::with_config(token, config);

    let retrieved_token = client.get_access_token();
    assert_eq!(retrieved_token.token, "test_token");
}

#[test]
fn test_client_clone() {
    let token = AccessToken::new("test_token".to_string(), 3600, None);
    let client = Client::new(token);
    let cloned = client.clone();

    // Both clients should share the same token
    assert_eq!(client.get_access_token().token, cloned.get_access_token().token);
}

#[test]
fn test_cloned_client_shares_token() {
    let token = AccessToken::new("original_token".to_string(), 3600, None);
    let client = Client::new(token);
    let cloned = client.clone();

    // Update token on original
    let new_token = AccessToken::new("updated_token".to_string(), 3600, None);
    client.set_access_token(new_token);

    // Cloned client should see the update (shared Arc<RwLock>)
    assert_eq!(cloned.get_access_token().token, "updated_token");
}

// =============================================================================
// Token Management Tests
// =============================================================================

#[test]
fn test_get_access_token() {
    let token = AccessToken::new("my_token".to_string(), 3600, Some("refresh".to_string()));
    let client = Client::new(token);

    let retrieved = client.get_access_token();
    assert_eq!(retrieved.token, "my_token");
    assert_eq!(retrieved.refresh_token, Some("refresh".to_string()));
}

#[test]
fn test_set_access_token() {
    let original = AccessToken::new("original".to_string(), 3600, None);
    let client = Client::new(original);

    let new_token = AccessToken::new("new_token".to_string(), 7200, Some("new_refresh".to_string()));
    client.set_access_token(new_token);

    let retrieved = client.get_access_token();
    assert_eq!(retrieved.token, "new_token");
    assert_eq!(retrieved.refresh_token, Some("new_refresh".to_string()));
}

#[test]
fn test_is_token_expired_with_valid_token() {
    let token = AccessToken::new("valid".to_string(), 3600, None);
    let client = Client::new(token);

    assert!(!client.is_token_expired());
}

#[test]
fn test_is_token_expired_with_expired_token() {
    let token = AccessToken::new("expired".to_string(), 0, None);
    let client = Client::new(token);

    std::thread::sleep(Duration::from_millis(10));
    assert!(client.is_token_expired());
}

#[test]
fn test_is_token_expired_reflects_updates() {
    let expired = AccessToken::new("expired".to_string(), 0, None);
    let client = Client::new(expired);

    std::thread::sleep(Duration::from_millis(10));
    assert!(client.is_token_expired());

    // Update to valid token
    let valid = AccessToken::new("valid".to_string(), 3600, None);
    client.set_access_token(valid);

    assert!(!client.is_token_expired());
}

// =============================================================================
// Thread Safety Tests
// =============================================================================

#[test]
fn test_client_is_send() {
    fn assert_send<T: Send>() {}
    assert_send::<Client>();
}

#[test]
fn test_client_is_sync() {
    fn assert_sync<T: Sync>() {}
    assert_sync::<Client>();
}

#[test]
fn test_client_config_is_send() {
    fn assert_send<T: Send>() {}
    assert_send::<ClientConfig>();
}

#[test]
fn test_client_config_is_sync() {
    fn assert_sync<T: Sync>() {}
    assert_sync::<ClientConfig>();
}

#[test]
fn test_client_shared_across_threads() {
    let token = AccessToken::new("shared".to_string(), 3600, None);
    let client = Arc::new(Client::new(token));

    let client1 = Arc::clone(&client);
    let client2 = Arc::clone(&client);

    let handle1 = thread::spawn(move || {
        client1.get_access_token().token
    });

    let handle2 = thread::spawn(move || {
        client2.get_access_token().token
    });

    let result1 = handle1.join().unwrap();
    let result2 = handle2.join().unwrap();

    assert_eq!(result1, "shared");
    assert_eq!(result2, "shared");
}

#[test]
fn test_concurrent_token_updates() {
    let token = AccessToken::new("initial".to_string(), 3600, None);
    let client = Arc::new(Client::new(token));

    let mut handles = vec![];

    for i in 0..10 {
        let client_clone = Arc::clone(&client);
        handles.push(thread::spawn(move || {
            let new_token = AccessToken::new(format!("token_{}", i), 3600, None);
            client_clone.set_access_token(new_token);
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Token should be one of the updated values
    let final_token = client.get_access_token();
    assert!(final_token.token.starts_with("token_"));
}

// =============================================================================
// Integration Tests with Mock Server
// =============================================================================

#[tokio::test]
async fn test_client_makes_request_with_auth_header() {
    let mut api = MockApi::new().await;

    let customers = vec![mock_server::fixtures::customer(1)];
    let data = serde_json::to_string(&customers).unwrap();
    let meta = mock_server::meta_json(0, 50, 1, 1);
    let response = format!(r#"{{"Data": {}, {}}}"#, data, meta);

    // Match any Bearer token (MockApi creates its own token)
    let mock = api
        .server
        .mock("GET", "/customers")
        .match_header("authorization", mockito::Matcher::Regex("Bearer .+".to_string()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(&response)
        .create();

    let result = api.client.customers().list(None).await;
    assert!(result.is_ok());

    mock.assert();
}

#[tokio::test]
async fn test_client_makes_request_with_user_agent() {
    let mut api = MockApi::new().await;

    let customers = vec![mock_server::fixtures::customer(1)];
    let data = serde_json::to_string(&customers).unwrap();
    let meta = mock_server::meta_json(0, 50, 1, 1);
    let response = format!(r#"{{"Data": {}, {}}}"#, data, meta);

    let mock = api
        .server
        .mock("GET", "/customers")
        .match_header("user-agent", mockito::Matcher::Regex(
            "spiris-bokforing-rust/.*".to_string()
        ))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(&response)
        .create();

    let result = api.client.customers().list(None).await;
    assert!(result.is_ok());

    mock.assert();
}

#[tokio::test]
async fn test_client_makes_request_with_accept_header() {
    let mut api = MockApi::new().await;

    let customers = vec![mock_server::fixtures::customer(1)];
    let data = serde_json::to_string(&customers).unwrap();
    let meta = mock_server::meta_json(0, 50, 1, 1);
    let response = format!(r#"{{"Data": {}, {}}}"#, data, meta);

    let mock = api
        .server
        .mock("GET", "/customers")
        .match_header("accept", "application/json")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(&response)
        .create();

    let result = api.client.customers().list(None).await;
    assert!(result.is_ok());

    mock.assert();
}

#[tokio::test]
async fn test_client_request_fails_with_expired_token() {
    let server = mockito::Server::new_async().await;
    let config = ClientConfig::new().base_url(server.url());
    let expired_token = AccessToken::new("expired".to_string(), -100, None);
    let client = Client::with_config(expired_token, config);

    tokio::time::sleep(Duration::from_millis(50)).await;

    let result = client.customers().list(None).await;

    assert!(matches!(result, Err(Error::TokenExpired)));
}

#[tokio::test]
async fn test_client_request_succeeds_after_token_update() {
    // Create server and client manually for this test
    let mut server = mockito::Server::new_async().await;
    let config = ClientConfig::new().base_url(server.url());
    let expired_token = AccessToken::new("expired".to_string(), -100, None);
    let client = Client::with_config(expired_token, config);

    let customers = vec![mock_server::fixtures::customer(1)];
    let data = serde_json::to_string(&customers).unwrap();
    let meta = mock_server::meta_json(0, 50, 1, 1);
    let response = format!(r#"{{"Data": {}, {}}}"#, data, meta);

    let _mock = server
        .mock("GET", "/customers")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(&response)
        .create();

    tokio::time::sleep(Duration::from_millis(50)).await;

    // Request should fail with expired token
    let result1 = client.customers().list(None).await;
    assert!(matches!(result1, Err(Error::TokenExpired)));

    // Update to valid token
    let valid_token = AccessToken::new("valid_token".to_string(), 3600, None);
    client.set_access_token(valid_token);

    // Request should now succeed
    let result2 = client.customers().list(None).await;
    assert!(result2.is_ok());
}

// =============================================================================
// HTTP Method Tests
// =============================================================================

#[tokio::test]
async fn test_client_get_method() {
    let mut api = MockApi::new().await;

    let customer = mock_server::fixtures::customer(1);
    let response = serde_json::to_string(&customer).unwrap();

    let mock = api
        .server
        .mock("GET", "/customers/cust-1")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(&response)
        .create();

    let result = api.client.customers().get("cust-1").await;
    assert!(result.is_ok());

    mock.assert();
}

#[tokio::test]
async fn test_client_post_method() {
    let mut api = MockApi::new().await;

    let customer = mock_server::fixtures::customer(1);
    let response = serde_json::to_string(&customer).unwrap();

    let mock = api
        .server
        .mock("POST", "/customers")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(&response)
        .create();

    let result = api.client.customers().create(&customer).await;
    assert!(result.is_ok());

    mock.assert();
}

#[tokio::test]
async fn test_client_put_method() {
    let mut api = MockApi::new().await;

    let customer = mock_server::fixtures::customer(1);
    let response = serde_json::to_string(&customer).unwrap();

    let mock = api
        .server
        .mock("PUT", "/customers/cust-1")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(&response)
        .create();

    let result = api.client.customers().update("cust-1", &customer).await;
    assert!(result.is_ok());

    mock.assert();
}

#[tokio::test]
async fn test_client_delete_method() {
    let mut api = MockApi::new().await;

    let mock = api
        .server
        .mock("DELETE", "/customers/cust-1")
        .with_status(204)
        .create();

    let result = api.client.customers().delete("cust-1").await;
    assert!(result.is_ok());

    mock.assert();
}

// =============================================================================
// Error Handling Tests
// =============================================================================

#[tokio::test]
async fn test_client_handles_401_unauthorized() {
    let mut api = MockApi::new().await;

    let _mock = api.mock_error("GET", "/customers", 401, r#"{"Message": "Unauthorized"}"#);

    let result = api.client.customers().list(None).await;

    assert!(matches!(result, Err(Error::AuthError(_))));
}

#[tokio::test]
async fn test_client_handles_403_forbidden() {
    let mut api = MockApi::new().await;

    let _mock = api.mock_error("GET", "/customers", 403, r#"{"Message": "Forbidden"}"#);

    let result = api.client.customers().list(None).await;

    assert!(matches!(result, Err(Error::AuthError(_))));
}

#[tokio::test]
async fn test_client_handles_404_not_found() {
    let mut api = MockApi::new().await;

    let _mock = api.mock_error("GET", "/customers/nonexistent", 404, r#"{"Message": "Not found"}"#);

    let result = api.client.customers().get("nonexistent").await;

    assert!(matches!(result, Err(Error::NotFound(_))));
}

#[tokio::test]
async fn test_client_handles_429_rate_limit() {
    let mut api = MockApi::new().await;

    let _mock = api.mock_error("GET", "/customers", 429, r#"{"Message": "Too many requests"}"#);

    let result = api.client.customers().list(None).await;

    assert!(matches!(result, Err(Error::RateLimitExceeded(_))));
}

#[tokio::test]
async fn test_client_handles_400_bad_request() {
    let mut api = MockApi::new().await;

    let _mock = api.mock_error("POST", "/customers", 400, r#"{"Message": "Invalid data"}"#);

    let customer = mock_server::fixtures::customer(1);
    let result = api.client.customers().create(&customer).await;

    assert!(matches!(result, Err(Error::InvalidRequest(_))));
}

#[tokio::test]
async fn test_client_handles_500_server_error() {
    let mut api = MockApi::new().await;

    let _mock = api.mock_error("GET", "/customers", 500, r#"{"Message": "Internal server error"}"#);

    let result = api.client.customers().list(None).await;

    assert!(matches!(result, Err(Error::ApiError { status_code: 500, .. })));
}

// =============================================================================
// URL Building Tests
// =============================================================================

#[tokio::test]
async fn test_client_builds_correct_url_for_list() {
    let mut api = MockApi::new().await;

    let customers = vec![mock_server::fixtures::customer(1)];
    let data = serde_json::to_string(&customers).unwrap();
    let meta = mock_server::meta_json(0, 50, 1, 1);
    let response = format!(r#"{{"Data": {}, {}}}"#, data, meta);

    let mock = api
        .server
        .mock("GET", "/customers")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(&response)
        .create();

    let result = api.client.customers().list(None).await;
    assert!(result.is_ok());

    mock.assert();
}

#[tokio::test]
async fn test_client_builds_correct_url_for_get_by_id() {
    let mut api = MockApi::new().await;

    let customer = mock_server::fixtures::customer(1);
    let response = serde_json::to_string(&customer).unwrap();

    let mock = api
        .server
        .mock("GET", "/customers/test-id-123")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(&response)
        .create();

    let result = api.client.customers().get("test-id-123").await;
    assert!(result.is_ok());

    mock.assert();
}

// =============================================================================
// Content-Type Header Tests
// =============================================================================

#[tokio::test]
async fn test_post_sends_content_type_json() {
    let mut api = MockApi::new().await;

    let customer = mock_server::fixtures::customer(1);
    let response = serde_json::to_string(&customer).unwrap();

    let mock = api
        .server
        .mock("POST", "/customers")
        .match_header("content-type", "application/json")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(&response)
        .create();

    let result = api.client.customers().create(&customer).await;
    assert!(result.is_ok());

    mock.assert();
}

#[tokio::test]
async fn test_put_sends_content_type_json() {
    let mut api = MockApi::new().await;

    let customer = mock_server::fixtures::customer(1);
    let response = serde_json::to_string(&customer).unwrap();

    let mock = api
        .server
        .mock("PUT", "/customers/cust-1")
        .match_header("content-type", "application/json")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(&response)
        .create();

    let result = api.client.customers().update("cust-1", &customer).await;
    assert!(result.is_ok());

    mock.assert();
}
