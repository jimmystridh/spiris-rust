// Skip these tests when decimal feature is enabled (uses f64 literals)
#![cfg(not(feature = "decimal"))]
//! Test utility functions and helpers for integration tests.

use spiris::{
    AccessToken, Article, Customer, Error, Invoice, PaginatedResponse, ResponseMetadata,
};
use std::fs;
use std::path::Path;
use std::time::Duration;

// =============================================================================
// Fixture Loading
// =============================================================================

/// Load a JSON fixture file from the fixtures directory
#[allow(dead_code)]
pub fn load_fixture(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to load fixture {}: {}", name, e))
}

/// Load and parse a JSON fixture into a type
#[allow(dead_code)]
pub fn load_fixture_as<T: serde::de::DeserializeOwned>(name: &str) -> T {
    let json = load_fixture(name);
    serde_json::from_str(&json)
        .unwrap_or_else(|e| panic!("Failed to parse fixture {}: {}", name, e))
}

// =============================================================================
// Assertion Helpers
// =============================================================================

/// Assert two customers are equal (comparing key fields)
#[allow(dead_code)]
pub fn assert_customer_eq(actual: &Customer, expected: &Customer) {
    assert_eq!(actual.id, expected.id, "Customer ID mismatch");
    assert_eq!(actual.name, expected.name, "Customer name mismatch");
    assert_eq!(actual.email, expected.email, "Customer email mismatch");
    assert_eq!(
        actual.customer_number, expected.customer_number,
        "Customer number mismatch"
    );
    assert_eq!(
        actual.is_active, expected.is_active,
        "Customer is_active mismatch"
    );
}

/// Assert two articles are equal (comparing key fields)
#[allow(dead_code)]
pub fn assert_article_eq(actual: &Article, expected: &Article) {
    assert_eq!(actual.id, expected.id, "Article ID mismatch");
    assert_eq!(actual.name, expected.name, "Article name mismatch");
    assert_eq!(
        actual.article_number, expected.article_number,
        "Article number mismatch"
    );
    assert_eq!(
        actual.sales_price, expected.sales_price,
        "Article sales_price mismatch"
    );
}

/// Assert two invoices are equal (comparing key fields)
#[allow(dead_code)]
pub fn assert_invoice_eq(actual: &Invoice, expected: &Invoice) {
    assert_eq!(actual.id, expected.id, "Invoice ID mismatch");
    assert_eq!(
        actual.invoice_number, expected.invoice_number,
        "Invoice number mismatch"
    );
    assert_eq!(
        actual.customer_id, expected.customer_id,
        "Invoice customer_id mismatch"
    );
    assert_eq!(
        actual.total_amount, expected.total_amount,
        "Invoice total_amount mismatch"
    );
}

/// Assert pagination metadata is correct
#[allow(dead_code)]
pub fn assert_pagination_meta(
    meta: &ResponseMetadata,
    expected_page: u32,
    expected_total_count: u32,
) {
    assert_eq!(
        meta.current_page, expected_page,
        "Current page mismatch: expected {}, got {}",
        expected_page, meta.current_page
    );
    assert_eq!(
        meta.total_count, expected_total_count,
        "Total count mismatch: expected {}, got {}",
        expected_total_count, meta.total_count
    );
}

/// Assert that an error has a specific status code
#[allow(dead_code)]
pub fn assert_error_status(error: &Error, expected_status: u16) {
    match error {
        Error::ApiError { status_code, .. } => {
            assert_eq!(
                *status_code, expected_status,
                "Expected status {}, got {}",
                expected_status, status_code
            );
        }
        _ => panic!("Expected ApiError, got {:?}", error),
    }
}

/// Assert that an error is a specific variant
#[allow(dead_code)]
pub fn assert_error_variant<F>(error: &Error, check: F)
where
    F: FnOnce(&Error) -> bool,
{
    assert!(check(error), "Error variant mismatch: {:?}", error);
}

/// Assert that a result is an error with a specific status
#[allow(dead_code)]
pub fn assert_result_error_status<T: std::fmt::Debug>(
    result: &Result<T, Error>,
    expected_status: u16,
) {
    match result {
        Ok(v) => panic!("Expected error with status {}, got Ok({:?})", expected_status, v),
        Err(e) => assert_error_status(e, expected_status),
    }
}

// =============================================================================
// Test Data Generators
// =============================================================================

/// Generate a random customer for testing
#[allow(dead_code)]
pub fn random_customer() -> Customer {
    let id = uuid_v4();
    Customer {
        id: Some(id.clone()),
        customer_number: Some(format!("{}", rand_u32() % 10000)),
        name: Some(format!("Test Customer {}", &id[..8])),
        email: Some(format!("test-{}@example.com", &id[..8])),
        is_active: Some(true),
        ..Default::default()
    }
}

/// Generate a random invoice for testing
#[allow(dead_code)]
pub fn random_invoice(customer_id: &str) -> Invoice {
    let id = uuid_v4();
    Invoice {
        id: Some(id.clone()),
        invoice_number: Some(format!("{}", 20000 + rand_u32() % 10000)),
        customer_id: Some(customer_id.to_string()),
        total_amount: Some((rand_u32() % 100000) as f64 / 100.0),
        rows: vec![],
        ..Default::default()
    }
}

/// Generate a random article for testing
#[allow(dead_code)]
pub fn random_article() -> Article {
    let id = uuid_v4();
    Article {
        id: Some(id.clone()),
        article_number: Some(format!("ART-{}", rand_u32() % 10000)),
        name: Some(format!("Test Article {}", &id[..8])),
        sales_price: Some((rand_u32() % 10000) as f64 / 100.0),
        is_active: Some(true),
        ..Default::default()
    }
}

/// Generate an expired access token
#[allow(dead_code)]
pub fn expired_token() -> AccessToken {
    AccessToken::new("expired_test_token".to_string(), -100, None)
}

/// Generate a valid access token
#[allow(dead_code)]
pub fn valid_token() -> AccessToken {
    AccessToken::new("valid_test_token".to_string(), 3600, None)
}

/// Generate a valid access token with refresh token
#[allow(dead_code)]
pub fn valid_token_with_refresh() -> AccessToken {
    let mut token = valid_token();
    token.refresh_token = Some("test_refresh_token".to_string());
    token
}

/// Generate a token that expires in a specific number of seconds
#[allow(dead_code)]
pub fn token_expiring_in(seconds: i64) -> AccessToken {
    AccessToken::new("expiring_token".to_string(), seconds, None)
}

// =============================================================================
// Async Helpers
// =============================================================================

/// Run an async function with a timeout
#[allow(dead_code)]
pub async fn with_timeout<F, T>(duration: Duration, f: F) -> Result<T, &'static str>
where
    F: std::future::Future<Output = T>,
{
    tokio::select! {
        result = f => Ok(result),
        _ = tokio::time::sleep(duration) => Err("Operation timed out"),
    }
}

/// Run an async function with a default 5 second timeout
#[allow(dead_code)]
pub async fn with_default_timeout<F, T>(f: F) -> Result<T, &'static str>
where
    F: std::future::Future<Output = T>,
{
    with_timeout(Duration::from_secs(5), f).await
}

/// Sleep for a specified number of milliseconds
#[allow(dead_code)]
pub async fn sleep_ms(ms: u64) {
    tokio::time::sleep(Duration::from_millis(ms)).await;
}

// =============================================================================
// Utility Functions
// =============================================================================

/// Generate a simple UUID v4 (for testing purposes)
fn uuid_v4() -> String {
    format!(
        "{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}",
        rand_u32(),
        rand_u32() & 0xFFFF,
        rand_u32() & 0x0FFF,
        (rand_u32() & 0x3FFF) | 0x8000,
        rand_u64() & 0xFFFFFFFFFFFF
    )
}

/// Simple random u32 (not cryptographically secure, for testing only)
fn rand_u32() -> u32 {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    RandomState::new().build_hasher().finish() as u32
}

/// Simple random u64 (not cryptographically secure, for testing only)
fn rand_u64() -> u64 {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    RandomState::new().build_hasher().finish()
}

/// Create a paginated response wrapper
#[allow(dead_code)]
pub fn make_paginated_response<T>(
    data: Vec<T>,
    current_page: u32,
    page_size: u32,
    total_count: u32,
) -> PaginatedResponse<T> {
    let total_pages = (total_count + page_size - 1) / page_size;
    PaginatedResponse {
        data,
        meta: ResponseMetadata {
            current_page,
            page_size,
            total_pages,
            total_count,
            has_next_page: current_page + 1 < total_pages,
            has_previous_page: current_page > 0,
        },
    }
}

// =============================================================================
// JSON Helpers
// =============================================================================

/// Pretty print JSON for debugging
#[allow(dead_code)]
pub fn pretty_json<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| "Failed to serialize".to_string())
}

/// Compare two JSON values ignoring field order
#[allow(dead_code)]
pub fn json_eq(a: &str, b: &str) -> bool {
    let a: serde_json::Value = serde_json::from_str(a).unwrap_or(serde_json::Value::Null);
    let b: serde_json::Value = serde_json::from_str(b).unwrap_or(serde_json::Value::Null);
    a == b
}

// =============================================================================
// Error Checking Macros
// =============================================================================

/// Check if an error matches a pattern
#[macro_export]
macro_rules! assert_error_matches {
    ($result:expr, $pattern:pat) => {
        match $result {
            Ok(v) => panic!("Expected error matching {}, got Ok({:?})", stringify!($pattern), v),
            Err(ref e) => {
                assert!(
                    matches!(e, $pattern),
                    "Error did not match pattern {}: {:?}",
                    stringify!($pattern),
                    e
                );
            }
        }
    };
}

// =============================================================================
// Tests for Test Utils
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_fixture() {
        let json = load_fixture("customer_simple.json");
        assert!(json.contains("Acme Corporation"));
    }

    #[test]
    fn test_load_fixture_as() {
        let customer: Customer = load_fixture_as("customer_simple.json");
        assert_eq!(customer.name, Some("Acme Corporation".to_string()));
    }

    #[test]
    fn test_random_customer() {
        let c1 = random_customer();
        let c2 = random_customer();
        assert_ne!(c1.id, c2.id);
        assert!(c1.name.is_some());
    }

    #[test]
    fn test_expired_token() {
        let token = expired_token();
        // Wait a bit to ensure it's expired
        std::thread::sleep(Duration::from_millis(10));
        assert!(token.is_expired());
    }

    #[test]
    fn test_valid_token() {
        let token = valid_token();
        assert!(!token.is_expired());
    }

    #[test]
    fn test_json_eq() {
        assert!(json_eq(r#"{"a":1,"b":2}"#, r#"{"b":2,"a":1}"#));
        assert!(!json_eq(r#"{"a":1}"#, r#"{"a":2}"#));
    }

    #[test]
    fn test_make_paginated_response() {
        let response = make_paginated_response(vec![1, 2, 3], 0, 10, 25);
        assert_eq!(response.data.len(), 3);
        assert_eq!(response.meta.total_pages, 3);
        assert!(response.meta.has_next_page);
        assert!(!response.meta.has_previous_page);
    }

    #[tokio::test]
    async fn test_with_timeout_success() {
        let result = with_timeout(Duration::from_secs(1), async { 42 }).await;
        assert_eq!(result, Ok(42));
    }

    #[tokio::test]
    async fn test_with_timeout_expired() {
        let result = with_timeout(Duration::from_millis(10), async {
            tokio::time::sleep(Duration::from_secs(1)).await;
            42
        })
        .await;
        assert!(result.is_err());
    }
}
