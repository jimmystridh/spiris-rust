//! Integration tests for the Client itself.

mod mock_server;

use mock_server::MockApi;
use spiris_bokforing::{AccessToken, Client, ClientConfig};

#[tokio::test]
async fn test_client_with_custom_config() {
    let mut server = mockito::Server::new_async().await;

    let token = AccessToken::new("custom_token".to_string(), 3600, None);
    let config = ClientConfig::new()
        .base_url(server.url())
        .timeout_seconds(60)
        .enable_tracing(false);

    let client = Client::with_config(token, config);

    let mock = server
        .mock("GET", "/customers")
        .match_header("Authorization", "Bearer custom_token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"Data": [], "Meta": {"CurrentPage": 0, "PageSize": 50, "TotalPages": 0, "TotalCount": 2, "HasNextPage": false, "HasPreviousPage": false}}"#)
        .create();

    let result = client.customers().list(None).await.unwrap();

    mock.assert();
    assert_eq!(result.data.len(), 0);
}

#[tokio::test]
async fn test_client_token_update() {
    let mut api = MockApi::new().await;

    // First request with original token
    let mock1 = api
        .server
        .mock("GET", "/customers/cust-1")
        .match_header("Authorization", "Bearer test_token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"Id": "cust-1", "Name": "Customer 1"}"#)
        .create();

    let result1 = api.client.customers().get("cust-1").await.unwrap();
    mock1.assert();
    assert_eq!(result1.id, Some("cust-1".to_string()));

    // Update the token
    let new_token = AccessToken::new("new_token".to_string(), 3600, None);
    api.client.set_access_token(new_token);

    // Second request with new token
    let mock2 = api
        .server
        .mock("GET", "/customers/cust-2")
        .match_header("Authorization", "Bearer new_token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"Id": "cust-2", "Name": "Customer 2"}"#)
        .create();

    let result2 = api.client.customers().get("cust-2").await.unwrap();
    mock2.assert();
    assert_eq!(result2.id, Some("cust-2".to_string()));
}

#[tokio::test]
async fn test_client_get_access_token() {
    let token = AccessToken::new(
        "my_token".to_string(),
        3600,
        Some("refresh_token".to_string()),
    );
    let client = Client::new(token);

    let retrieved_token = client.get_access_token();
    assert_eq!(retrieved_token.token, "my_token");
    assert_eq!(
        retrieved_token.refresh_token,
        Some("refresh_token".to_string())
    );
}

#[tokio::test]
async fn test_client_is_token_expired() {
    // Fresh token
    let fresh_token = AccessToken::new("fresh".to_string(), 3600, None);
    let client = Client::new(fresh_token);
    assert!(!client.is_token_expired());

    // Expired token (0 seconds TTL)
    let expired_token = AccessToken::new("expired".to_string(), 0, None);
    let expired_client = Client::new(expired_token);
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    assert!(expired_client.is_token_expired());
}

#[tokio::test]
async fn test_client_user_agent_header() {
    let mut server = mockito::Server::new_async().await;

    let token = AccessToken::new("token".to_string(), 3600, None);
    let config = ClientConfig::new().base_url(server.url());
    let client = Client::with_config(token, config);

    let mock = server
        .mock("GET", "/customers")
        .match_header(
            "User-Agent",
            mockito::Matcher::Regex(r"spiris-bokforing-rust/.*".to_string()),
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"Data": [], "Meta": {"CurrentPage": 0, "PageSize": 50, "TotalPages": 0, "TotalCount": 2, "HasNextPage": false, "HasPreviousPage": false}}"#)
        .create();

    let _ = client.customers().list(None).await.unwrap();
    mock.assert();
}

#[tokio::test]
async fn test_client_accept_json_header() {
    let mut server = mockito::Server::new_async().await;

    let token = AccessToken::new("token".to_string(), 3600, None);
    let config = ClientConfig::new().base_url(server.url());
    let client = Client::with_config(token, config);

    let mock = server
        .mock("GET", "/customers")
        .match_header("Accept", "application/json")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"Data": [], "Meta": {"CurrentPage": 0, "PageSize": 50, "TotalPages": 0, "TotalCount": 2, "HasNextPage": false, "HasPreviousPage": false}}"#)
        .create();

    let _ = client.customers().list(None).await.unwrap();
    mock.assert();
}

#[tokio::test]
async fn test_client_content_type_on_post() {
    let mut api = MockApi::new().await;

    let mock = api
        .server
        .mock("POST", "/customers")
        .match_header("Content-Type", "application/json")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(r#"{"Id": "new-cust", "Name": "Test"}"#)
        .create();

    let customer = spiris_bokforing::Customer {
        name: Some("Test".to_string()),
        ..Default::default()
    };
    let _ = api.client.customers().create(&customer).await.unwrap();
    mock.assert();
}

#[tokio::test]
async fn test_client_clone() {
    let mut api = MockApi::new().await;

    // Clone the client
    let cloned_client = api.client.clone();

    let mock = api.mock_get(
        "/customers/cust-1",
        r#"{"Id": "cust-1", "Name": "Customer"}"#,
    );

    // Use the cloned client
    let result = cloned_client.customers().get("cust-1").await.unwrap();

    mock.assert();
    assert_eq!(result.id, Some("cust-1".to_string()));
}

#[tokio::test]
async fn test_empty_list_response() {
    let mut api = MockApi::new().await;

    let response_body = r#"{
        "Data": [],
        "Meta": {
            "CurrentPage": 0,
            "PageSize": 50,
            "TotalPages": 0,
            "TotalCount": 0, "HasNextPage": false, "HasPreviousPage": false
        }
    }"#;

    let mock = api.mock_get("/customers", response_body);

    let result = api.client.customers().list(None).await.unwrap();

    mock.assert();
    assert!(result.data.is_empty());
    assert_eq!(result.meta.total_count, 0);
    assert_eq!(result.meta.total_pages, 0);
}
