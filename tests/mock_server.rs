//! Shared mock server utilities for integration tests.

use mockito::{Matcher, Mock, Server, ServerGuard};
use spiris::{AccessToken, Client, ClientConfig, Money, RetryConfig};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Helper function to create Money from f64.
/// Works with both decimal feature (converts via string) and without (direct f64).
#[allow(dead_code)]
pub fn money(value: f64) -> Money {
    #[cfg(feature = "decimal")]
    {
        use std::str::FromStr;
        rust_decimal::Decimal::from_str(&value.to_string()).expect("Invalid decimal value")
    }
    #[cfg(not(feature = "decimal"))]
    {
        value
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Helper to create a standard paginated Meta section
#[allow(dead_code)]
pub fn meta_json(current_page: u32, page_size: u32, total_pages: u32, total_count: u32) -> String {
    format!(
        r#""Meta": {{
            "CurrentPage": {},
            "PageSize": {},
            "TotalPages": {},
            "TotalCount": {},
            "HasNextPage": {},
            "HasPreviousPage": {}
        }}"#,
        current_page,
        page_size,
        total_pages,
        total_count,
        current_page + 1 < total_pages,
        current_page > 0
    )
}

/// Create a paginated response JSON wrapper
#[allow(dead_code)]
pub fn paginated_response(data_json: &str, current_page: u32, total_count: u32) -> String {
    let page_size = 50;
    let total_pages = total_count.div_ceil(page_size);
    format!(
        r#"{{"Data": {}, {}}}"#,
        data_json,
        meta_json(current_page, page_size, total_pages, total_count)
    )
}

// =============================================================================
// Mock Response Builder
// =============================================================================

/// Represents a mock response for sequenced responses
#[derive(Clone)]
#[allow(dead_code)]
pub struct MockResponse {
    pub status: u16,
    pub body: String,
    pub headers: Vec<(String, String)>,
    pub delay: Option<Duration>,
}

#[allow(dead_code)]
impl MockResponse {
    pub fn ok(body: &str) -> Self {
        Self {
            status: 200,
            body: body.to_string(),
            headers: vec![("content-type".to_string(), "application/json".to_string())],
            delay: None,
        }
    }

    pub fn created(body: &str) -> Self {
        Self {
            status: 201,
            body: body.to_string(),
            headers: vec![("content-type".to_string(), "application/json".to_string())],
            delay: None,
        }
    }

    pub fn no_content() -> Self {
        Self {
            status: 204,
            body: String::new(),
            headers: vec![],
            delay: None,
        }
    }

    pub fn error(status: u16, message: &str) -> Self {
        Self {
            status,
            body: format!(r#"{{"Message": "{}"}}"#, message),
            headers: vec![("content-type".to_string(), "application/json".to_string())],
            delay: None,
        }
    }

    pub fn rate_limit(retry_after_secs: u32) -> Self {
        Self {
            status: 429,
            body: r#"{"Message": "Rate limit exceeded"}"#.to_string(),
            headers: vec![
                ("content-type".to_string(), "application/json".to_string()),
                ("Retry-After".to_string(), retry_after_secs.to_string()),
            ],
            delay: None,
        }
    }

    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.delay = Some(delay);
        self
    }

    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.push((key.to_string(), value.to_string()));
        self
    }
}

// =============================================================================
// Request Recorder
// =============================================================================

/// Records details of requests made to the mock server
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct RecordedRequest {
    pub method: String,
    pub path: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
    pub query_params: Vec<(String, String)>,
}

/// Thread-safe request recorder
#[derive(Default, Clone)]
#[allow(dead_code)]
pub struct RequestRecorder {
    requests: Arc<std::sync::Mutex<Vec<RecordedRequest>>>,
}

#[allow(dead_code)]
impl RequestRecorder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&self, request: RecordedRequest) {
        self.requests.lock().unwrap().push(request);
    }

    pub fn requests(&self) -> Vec<RecordedRequest> {
        self.requests.lock().unwrap().clone()
    }

    pub fn request_count(&self, path: &str) -> usize {
        self.requests
            .lock()
            .unwrap()
            .iter()
            .filter(|r| r.path == path)
            .count()
    }

    pub fn clear(&self) {
        self.requests.lock().unwrap().clear();
    }

    pub fn last_request(&self) -> Option<RecordedRequest> {
        self.requests.lock().unwrap().last().cloned()
    }

    pub fn requests_to(&self, path: &str) -> Vec<RecordedRequest> {
        self.requests
            .lock()
            .unwrap()
            .iter()
            .filter(|r| r.path == path)
            .cloned()
            .collect()
    }
}

// =============================================================================
// Main MockApi
// =============================================================================

#[allow(dead_code)]
pub struct MockApi {
    pub server: ServerGuard,
    pub client: Client,
    request_counter: Arc<AtomicUsize>,
    pub recorder: RequestRecorder,
}

#[allow(dead_code)]
impl MockApi {
    /// Create a new MockApi with default configuration
    pub async fn new() -> Self {
        Self::with_token("test_token", 3600).await
    }

    /// Create a new MockApi with a specific token
    /// Note: Retries are disabled by default to make tests predictable
    pub async fn with_token(token: &str, expires_in: i64) -> Self {
        let server = Server::new_async().await;
        let token = AccessToken::new(token.to_string(), expires_in, None);
        let config = ClientConfig::new()
            .base_url(server.url())
            .retry_config(RetryConfig::new().max_retries(0));
        let client = Client::with_config(token, config);
        Self {
            server,
            client,
            request_counter: Arc::new(AtomicUsize::new(0)),
            recorder: RequestRecorder::new(),
        }
    }

    /// Create a new MockApi with an expired token
    pub async fn with_expired_token() -> Self {
        Self::with_token("expired_token", -100).await
    }

    /// Create a new MockApi with a token that includes a refresh token
    /// Note: Retries are disabled by default to make tests predictable
    pub async fn with_refresh_token(access_token: &str, refresh_token: &str) -> Self {
        let server = Server::new_async().await;
        let mut token = AccessToken::new(access_token.to_string(), 3600, None);
        token.refresh_token = Some(refresh_token.to_string());
        let config = ClientConfig::new()
            .base_url(server.url())
            .retry_config(RetryConfig::new().max_retries(0));
        let client = Client::with_config(token, config);
        Self {
            server,
            client,
            request_counter: Arc::new(AtomicUsize::new(0)),
            recorder: RequestRecorder::new(),
        }
    }

    /// Create a client with custom retry configuration
    pub async fn with_retry_config(retry_config: RetryConfig) -> Self {
        let server = Server::new_async().await;
        let token = AccessToken::new("test_token".to_string(), 3600, None);
        let config = ClientConfig::new()
            .base_url(server.url())
            .retry_config(retry_config);
        let client = Client::with_config(token, config);
        Self {
            server,
            client,
            request_counter: Arc::new(AtomicUsize::new(0)),
            recorder: RequestRecorder::new(),
        }
    }

    /// Get the base URL of the mock server
    pub fn url(&self) -> String {
        self.server.url()
    }

    /// Get current request count
    pub fn request_count(&self) -> usize {
        self.request_counter.load(Ordering::SeqCst)
    }

    /// Reset request counter
    pub fn reset_counter(&self) {
        self.request_counter.store(0, Ordering::SeqCst);
    }

    // =========================================================================
    // Basic Mock Methods
    // =========================================================================

    pub fn mock_get(&mut self, path: &str, response_body: &str) -> Mock {
        self.server
            .mock("GET", path)
            .match_header("Authorization", "Bearer test_token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response_body)
            .create()
    }

    pub fn mock_get_with_query(
        &mut self,
        path: &str,
        query: Vec<(&str, &str)>,
        response_body: &str,
    ) -> Mock {
        let mut mock = self
            .server
            .mock("GET", path)
            .match_header("Authorization", "Bearer test_token");

        for (key, value) in query {
            mock = mock.match_query(Matcher::UrlEncoded(key.into(), value.into()));
        }

        mock.with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response_body)
            .create()
    }

    pub fn mock_get_with_auth(
        &mut self,
        path: &str,
        auth_header: &str,
        response_body: &str,
    ) -> Mock {
        self.server
            .mock("GET", path)
            .match_header("Authorization", auth_header)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response_body)
            .create()
    }

    pub fn mock_post(&mut self, path: &str, response_body: &str) -> Mock {
        self.server
            .mock("POST", path)
            .match_header("Authorization", "Bearer test_token")
            .match_header("Content-Type", "application/json")
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(response_body)
            .create()
    }

    pub fn mock_post_with_body(
        &mut self,
        path: &str,
        request_body: &str,
        response_body: &str,
    ) -> Mock {
        self.server
            .mock("POST", path)
            .match_header("Authorization", "Bearer test_token")
            .match_header("Content-Type", "application/json")
            .match_body(Matcher::Json(serde_json::from_str(request_body).unwrap()))
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(response_body)
            .create()
    }

    pub fn mock_put(&mut self, path: &str, response_body: &str) -> Mock {
        self.server
            .mock("PUT", path)
            .match_header("Authorization", "Bearer test_token")
            .match_header("Content-Type", "application/json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response_body)
            .create()
    }

    pub fn mock_delete(&mut self, path: &str) -> Mock {
        self.server
            .mock("DELETE", path)
            .match_header("Authorization", "Bearer test_token")
            .with_status(204)
            .create()
    }

    pub fn mock_error(&mut self, method: &str, path: &str, status: u16, body: &str) -> Mock {
        self.server
            .mock(method, path)
            .with_status(status as usize)
            .with_header("content-type", "application/json")
            .with_body(body)
            .create()
    }

    pub fn mock_get_bytes(&mut self, path: &str, data: &[u8]) -> Mock {
        self.server
            .mock("GET", path)
            .match_header("Authorization", "Bearer test_token")
            .with_status(200)
            .with_header("content-type", "application/pdf")
            .with_body(data)
            .create()
    }

    // =========================================================================
    // Error Response Helpers
    // =========================================================================

    /// Mock a 400 Bad Request with validation errors
    pub fn mock_validation_error(&mut self, path: &str, errors: Vec<(&str, &str)>) -> Mock {
        let validation_errors: Vec<String> = errors
            .iter()
            .map(|(field, msg)| format!(r#"{{"Field": "{}", "Message": "{}"}}"#, field, msg))
            .collect();

        let body = format!(
            r#"{{
                "ErrorCode": "VALIDATION_ERROR",
                "Message": "Validation failed",
                "ValidationErrors": [{}]
            }}"#,
            validation_errors.join(", ")
        );

        self.server
            .mock("POST", path)
            .with_status(400)
            .with_header("content-type", "application/json")
            .with_body(body)
            .create()
    }

    /// Mock a 401 Unauthorized response
    pub fn mock_unauthorized(&mut self, path: &str) -> Mock {
        self.server
            .mock("GET", path)
            .with_status(401)
            .with_header("content-type", "application/json")
            .with_body(r#"{"Message": "Unauthorized"}"#)
            .create()
    }

    /// Mock a 403 Forbidden response
    pub fn mock_forbidden(&mut self, path: &str) -> Mock {
        self.server
            .mock("GET", path)
            .with_status(403)
            .with_header("content-type", "application/json")
            .with_body(r#"{"Message": "Forbidden"}"#)
            .create()
    }

    /// Mock a 404 Not Found response
    pub fn mock_not_found(&mut self, path: &str) -> Mock {
        self.server
            .mock("GET", path)
            .with_status(404)
            .with_header("content-type", "application/json")
            .with_body(r#"{"Message": "Resource not found"}"#)
            .create()
    }

    /// Mock a 500 Internal Server Error
    pub fn mock_server_error(&mut self, method: &str, path: &str) -> Mock {
        self.server
            .mock(method, path)
            .with_status(500)
            .with_header("content-type", "application/json")
            .with_body(r#"{"Message": "Internal Server Error"}"#)
            .create()
    }

    // =========================================================================
    // Rate Limiting Simulation
    // =========================================================================

    /// Mock a rate limit (429) response with Retry-After header
    pub fn mock_rate_limit(&mut self, path: &str, retry_after_secs: u32) -> Mock {
        self.server
            .mock("GET", path)
            .with_status(429)
            .with_header("content-type", "application/json")
            .with_header("Retry-After", &retry_after_secs.to_string())
            .with_body(r#"{"Message": "Rate limit exceeded"}"#)
            .create()
    }

    /// Mock rate limiting that triggers after N requests
    /// Returns a mock that will return 429 after `threshold` requests
    pub fn mock_rate_limit_after(
        &mut self,
        path: &str,
        threshold: usize,
        retry_after_secs: u32,
        success_body: &str,
    ) -> (Mock, Mock) {
        // First N requests succeed
        let success_mock = self
            .server
            .mock("GET", path)
            .match_header("Authorization", "Bearer test_token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(success_body)
            .expect(threshold)
            .create();

        // After that, return rate limit
        let rate_limit_mock = self
            .server
            .mock("GET", path)
            .match_header("Authorization", "Bearer test_token")
            .with_status(429)
            .with_header("content-type", "application/json")
            .with_header("Retry-After", &retry_after_secs.to_string())
            .with_body(r#"{"Message": "Rate limit exceeded"}"#)
            .create();

        (success_mock, rate_limit_mock)
    }

    // =========================================================================
    // Response Sequences (for retry testing)
    // =========================================================================

    /// Create a sequence of mocks that will be matched in order
    /// Each response in the sequence will be returned once
    pub fn mock_sequence(
        &mut self,
        method: &str,
        path: &str,
        responses: Vec<MockResponse>,
    ) -> Vec<Mock> {
        responses
            .into_iter()
            .map(|response| {
                let mut mock = self.server.mock(method, path).expect(1);

                for (key, value) in &response.headers {
                    mock = mock.with_header(key, value);
                }

                mock.with_status(response.status as usize)
                    .with_body(&response.body)
                    .create()
            })
            .collect()
    }

    /// Mock GET with a sequence of responses (convenience method)
    pub fn mock_get_sequence(&mut self, path: &str, responses: Vec<MockResponse>) -> Vec<Mock> {
        self.mock_sequence("GET", path, responses)
    }

    /// Mock POST with a sequence of responses
    pub fn mock_post_sequence(&mut self, path: &str, responses: Vec<MockResponse>) -> Vec<Mock> {
        self.mock_sequence("POST", path, responses)
    }

    // =========================================================================
    // Network Failure Simulation
    // =========================================================================

    /// Mock a delayed response (for timeout testing)
    pub fn mock_slow_response(&mut self, path: &str, delay_ms: u64, response_body: &str) -> Mock {
        let body = response_body.to_string();
        self.server
            .mock("GET", path)
            .match_header("Authorization", "Bearer test_token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body_from_request(move |_| {
                std::thread::sleep(Duration::from_millis(delay_ms));
                body.as_bytes().to_vec()
            })
            .create()
    }

    /// Mock a connection that closes without response (simulates network failure)
    /// Note: mockito doesn't fully support this, so we simulate with 502
    pub fn mock_connection_reset(&mut self, path: &str) -> Mock {
        self.server
            .mock("GET", path)
            .with_status(502)
            .with_body("")
            .create()
    }

    // =========================================================================
    // Request Validation Helpers
    // =========================================================================

    /// Expect a specific header on requests to a path
    pub fn expect_header(&mut self, path: &str, header: &str, value: &str) -> Mock {
        self.server
            .mock("GET", path)
            .match_header(header, value)
            .with_status(200)
            .with_body("{}")
            .create()
    }

    /// Expect a specific query parameter
    pub fn expect_query_param(&mut self, path: &str, key: &str, value: &str) -> Mock {
        self.server
            .mock("GET", path)
            .match_query(Matcher::UrlEncoded(key.into(), value.into()))
            .with_status(200)
            .with_body("{}")
            .create()
    }

    /// Expect request body to match JSON
    pub fn expect_json_body(&mut self, method: &str, path: &str, expected_json: &str) -> Mock {
        self.server
            .mock(method, path)
            .match_body(Matcher::Json(serde_json::from_str(expected_json).unwrap()))
            .with_status(200)
            .with_body("{}")
            .create()
    }

    /// Expect request body to contain a substring
    pub fn expect_body_contains(&mut self, method: &str, path: &str, substring: &str) -> Mock {
        self.server
            .mock(method, path)
            .match_body(Matcher::Regex(substring.to_string()))
            .with_status(200)
            .with_body("{}")
            .create()
    }

    // =========================================================================
    // Pagination Helpers
    // =========================================================================

    /// Create mocks for paginated responses
    /// Each tuple is (items, has_more)
    pub fn mock_paginated<T: serde::Serialize>(
        &mut self,
        path: &str,
        pages: Vec<(Vec<T>, bool)>,
    ) -> Vec<Mock> {
        let total_count: usize = pages.iter().map(|(items, _)| items.len()).sum();
        let total_pages = pages.len() as u32;

        pages
            .into_iter()
            .enumerate()
            .map(|(page_num, (items, _has_more))| {
                let data_json = serde_json::to_string(&items).unwrap();
                let response = format!(
                    r#"{{"Data": {}, {}}}"#,
                    data_json,
                    meta_json(page_num as u32, 50, total_pages, total_count as u32)
                );

                self.server
                    .mock("GET", path)
                    .match_header("Authorization", "Bearer test_token")
                    .match_query(Matcher::UrlEncoded("page".into(), page_num.to_string()))
                    .with_status(200)
                    .with_header("content-type", "application/json")
                    .with_body(&response)
                    .expect(1)
                    .create()
            })
            .collect()
    }

    // =========================================================================
    // Assertion Helpers
    // =========================================================================

    /// Assert that a path was called exactly N times
    pub fn assert_request_count(&self, path: &str, expected: usize) {
        let actual = self.recorder.request_count(path);
        assert_eq!(
            actual, expected,
            "Expected {} requests to {}, got {}",
            expected, path, actual
        );
    }

    /// Assert that the last request to a path had a specific header
    pub fn assert_last_request_header(&self, path: &str, header: &str, value: &str) {
        let requests = self.recorder.requests_to(path);
        let last = requests.last().expect("No requests recorded");
        let found = last
            .headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(header));
        assert!(
            found.is_some() && found.unwrap().1 == value,
            "Expected header {}={}, got {:?}",
            header,
            value,
            found
        );
    }
}

// =============================================================================
// OAuth2 Mock Server
// =============================================================================

/// Mock OAuth2 server for testing authentication flows
#[allow(dead_code)]
pub struct MockOAuthServer {
    pub server: ServerGuard,
    expected_pkce_verifier: Option<String>,
}

#[allow(dead_code)]
impl MockOAuthServer {
    pub async fn new() -> Self {
        let server = Server::new_async().await;
        Self {
            server,
            expected_pkce_verifier: None,
        }
    }

    pub fn url(&self) -> String {
        self.server.url()
    }

    pub fn token_url(&self) -> String {
        format!("{}/connect/token", self.server.url())
    }

    pub fn auth_url(&self) -> String {
        format!("{}/connect/authorize", self.server.url())
    }

    /// Set expected PKCE verifier for validation
    pub fn expect_pkce_verifier(&mut self, verifier: &str) {
        self.expected_pkce_verifier = Some(verifier.to_string());
    }

    /// Mock successful token exchange
    pub fn mock_token_exchange(&mut self, access_token: &str, expires_in: u64) -> Mock {
        let response = format!(
            r#"{{
                "access_token": "{}",
                "token_type": "Bearer",
                "expires_in": {}
            }}"#,
            access_token, expires_in
        );

        self.server
            .mock("POST", "/connect/token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(&response)
            .create()
    }

    /// Mock token exchange with refresh token
    pub fn mock_token_exchange_with_refresh(
        &mut self,
        access_token: &str,
        refresh_token: &str,
        expires_in: u64,
    ) -> Mock {
        let response = format!(
            r#"{{
                "access_token": "{}",
                "token_type": "Bearer",
                "expires_in": {},
                "refresh_token": "{}"
            }}"#,
            access_token, expires_in, refresh_token
        );

        self.server
            .mock("POST", "/connect/token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(&response)
            .create()
    }

    /// Mock token exchange that validates PKCE verifier
    pub fn mock_token_exchange_with_pkce(
        &mut self,
        expected_verifier: &str,
        access_token: &str,
        expires_in: u64,
    ) -> Mock {
        let response = format!(
            r#"{{
                "access_token": "{}",
                "token_type": "Bearer",
                "expires_in": {}
            }}"#,
            access_token, expires_in
        );

        self.server
            .mock("POST", "/connect/token")
            .match_body(Matcher::Regex(format!(
                "code_verifier={}",
                expected_verifier
            )))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(&response)
            .create()
    }

    /// Mock token refresh
    pub fn mock_token_refresh(
        &mut self,
        expected_refresh_token: &str,
        new_access_token: &str,
        new_refresh_token: Option<&str>,
        expires_in: u64,
    ) -> Mock {
        let mut response = format!(
            r#"{{
                "access_token": "{}",
                "token_type": "Bearer",
                "expires_in": {}"#,
            new_access_token, expires_in
        );

        if let Some(refresh) = new_refresh_token {
            response.push_str(&format!(r#", "refresh_token": "{}""#, refresh));
        }
        response.push('}');

        self.server
            .mock("POST", "/connect/token")
            .match_body(Matcher::Regex(format!(
                "refresh_token={}",
                expected_refresh_token
            )))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(&response)
            .create()
    }

    /// Mock failed token exchange (invalid grant)
    pub fn mock_token_error(&mut self, error: &str, description: &str) -> Mock {
        let response = format!(
            r#"{{
                "error": "{}",
                "error_description": "{}"
            }}"#,
            error, description
        );

        self.server
            .mock("POST", "/connect/token")
            .with_status(400)
            .with_header("content-type", "application/json")
            .with_body(&response)
            .create()
    }

    /// Mock invalid PKCE verifier error
    pub fn mock_pkce_error(&mut self) -> Mock {
        self.mock_token_error("invalid_grant", "Invalid PKCE code verifier")
    }

    /// Mock expired refresh token error
    pub fn mock_refresh_token_expired(&mut self) -> Mock {
        self.mock_token_error("invalid_grant", "Refresh token expired")
    }
}

// =============================================================================
// Test Data Generators
// =============================================================================

#[allow(dead_code)]
pub mod fixtures {
    use spiris::{Article, Customer, Invoice, InvoiceRow};

    pub fn customer(id: u32) -> Customer {
        Customer {
            id: Some(format!("cust-{:03}", id)),
            customer_number: Some(format!("{}", 1000 + id)),
            name: Some(format!("Test Customer {}", id)),
            email: Some(format!("customer{}@test.com", id)),
            is_active: Some(true),
            ..Default::default()
        }
    }

    pub fn customer_json(id: u32) -> String {
        serde_json::to_string(&customer(id)).unwrap()
    }

    pub fn customers(count: u32) -> Vec<Customer> {
        (1..=count).map(customer).collect()
    }

    pub fn customers_json(count: u32) -> String {
        serde_json::to_string(&customers(count)).unwrap()
    }

    pub fn article(id: u32) -> Article {
        Article {
            id: Some(format!("art-{:03}", id)),
            article_number: Some(format!("ART-{}", id)),
            name: Some(format!("Test Article {}", id)),
            sales_price: Some(super::money(100.0 * id as f64)),
            is_active: Some(true),
            ..Default::default()
        }
    }

    pub fn invoice(id: u32, customer_id: &str) -> Invoice {
        Invoice {
            id: Some(format!("inv-{:03}", id)),
            invoice_number: Some(format!("{}", 2000 + id)),
            customer_id: Some(customer_id.to_string()),
            total_amount: Some(super::money(1000.0 * id as f64)),
            rows: vec![invoice_row(1)],
            ..Default::default()
        }
    }

    pub fn invoice_row(id: u32) -> InvoiceRow {
        InvoiceRow {
            id: Some(format!("row-{:03}", id)),
            text: Some(format!("Line item {}", id)),
            unit_price: Some(super::money(100.0)),
            quantity: Some(super::money(id as f64)),
            ..Default::default()
        }
    }

    pub fn expired_token() -> spiris::AccessToken {
        spiris::AccessToken::new("expired_token".to_string(), -100, None)
    }

    pub fn valid_token() -> spiris::AccessToken {
        spiris::AccessToken::new("valid_token".to_string(), 3600, None)
    }

    pub fn valid_token_with_refresh() -> spiris::AccessToken {
        let mut token = valid_token();
        token.refresh_token = Some("refresh_token".to_string());
        token
    }
}

// =============================================================================
// Tests for Mock Infrastructure
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_api_creation() {
        let api = MockApi::new().await;
        assert!(!api.url().is_empty());
    }

    #[tokio::test]
    async fn test_mock_response_builder() {
        let ok = MockResponse::ok(r#"{"test": true}"#);
        assert_eq!(ok.status, 200);

        let error = MockResponse::error(500, "Server Error");
        assert_eq!(error.status, 500);
        assert!(error.body.contains("Server Error"));

        let rate_limit = MockResponse::rate_limit(60);
        assert_eq!(rate_limit.status, 429);
        assert!(rate_limit
            .headers
            .iter()
            .any(|(k, v)| k == "Retry-After" && v == "60"));
    }

    #[tokio::test]
    async fn test_mock_oauth_server() {
        let mut oauth = MockOAuthServer::new().await;
        let _mock = oauth.mock_token_exchange("test_access_token", 3600);
        assert!(!oauth.token_url().is_empty());
    }

    #[test]
    fn test_fixtures() {
        let customer = fixtures::customer(1);
        assert_eq!(customer.id, Some("cust-001".to_string()));

        let customers = fixtures::customers(3);
        assert_eq!(customers.len(), 3);

        let invoice = fixtures::invoice(1, "cust-001");
        assert_eq!(invoice.customer_id, Some("cust-001".to_string()));
    }
}
