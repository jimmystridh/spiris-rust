//! Integration tests for pagination stream functionality.
//!
//! These tests verify that pagination streaming correctly:
//! - Yields all items across multiple pages
//! - Handles empty results
//! - Propagates errors appropriately
//! - Fetches pages lazily
//!
//! Note: These tests are designed for when Stream-based pagination is implemented.
//! Currently they test the manual pagination approach.

mod mock_server;

use mock_server::{fixtures, meta_json, MockApi};
use spiris::PaginationParams;

// =============================================================================
// Manual Pagination Tests (Current Implementation)
// =============================================================================

#[tokio::test]
async fn test_list_customers_first_page() {
    let mut api = MockApi::new().await;

    let data = serde_json::to_string(&fixtures::customers(3)).unwrap();
    let meta = meta_json(0, 50, 2, 5);
    let response = format!(r#"{{"Data": {}, {}}}"#, data, meta);

    let _mock = api.mock_get("/customers", &response);

    let result = api.client.customers().list(None).await.unwrap();

    assert_eq!(result.data.len(), 3);
    assert_eq!(result.meta.current_page, 0);
    assert_eq!(result.meta.total_count, 5);
    assert!(result.meta.has_next_page);
    assert!(!result.meta.has_previous_page);
}

#[tokio::test]
async fn test_list_customers_with_pagination_params() {
    let mut api = MockApi::new().await;

    let data = serde_json::to_string(&fixtures::customers(2)).unwrap();
    let meta = meta_json(1, 10, 3, 25);
    let response = format!(r#"{{"Data": {}, {}}}"#, data, meta);

    let _mock = api.mock_get_with_query(
        "/customers",
        vec![("page", "1"), ("pagesize", "10")],
        &response,
    );

    let params = PaginationParams::new().page(1).pagesize(10);
    let result = api.client.customers().list(Some(params)).await.unwrap();

    assert_eq!(result.data.len(), 2);
    assert_eq!(result.meta.current_page, 1);
    assert_eq!(result.meta.page_size, 10);
    assert!(result.meta.has_next_page);
    assert!(result.meta.has_previous_page);
}

#[tokio::test]
async fn test_list_customers_last_page() {
    let mut api = MockApi::new().await;

    let data = serde_json::to_string(&fixtures::customers(2)).unwrap();
    let meta = meta_json(2, 10, 3, 25);
    let response = format!(r#"{{"Data": {}, {}}}"#, data, meta);

    let _mock = api.mock_get_with_query(
        "/customers",
        vec![("page", "2"), ("pagesize", "10")],
        &response,
    );

    let params = PaginationParams::new().page(2).pagesize(10);
    let result = api.client.customers().list(Some(params)).await.unwrap();

    assert!(!result.meta.has_next_page);
    assert!(result.meta.has_previous_page);
}

#[tokio::test]
async fn test_list_customers_empty_result() {
    let mut api = MockApi::new().await;

    let meta = meta_json(0, 50, 0, 0);
    let response = format!(r#"{{"Data": [], {}}}"#, meta);

    let _mock = api.mock_get("/customers", &response);

    let result = api.client.customers().list(None).await.unwrap();

    assert!(result.data.is_empty());
    assert_eq!(result.meta.total_count, 0);
    assert_eq!(result.meta.total_pages, 0);
    assert!(!result.meta.has_next_page);
    assert!(!result.meta.has_previous_page);
}

#[tokio::test]
async fn test_list_invoices_pagination() {
    let mut api = MockApi::new().await;

    let invoices = vec![
        fixtures::invoice(1, "cust-001"),
        fixtures::invoice(2, "cust-001"),
    ];

    let data = serde_json::to_string(&invoices).unwrap();
    let meta = meta_json(0, 50, 1, 2);
    let response = format!(r#"{{"Data": {}, {}}}"#, data, meta);

    let _mock = api.mock_get("/customerinvoices", &response);

    let result = api.client.invoices().list(None).await.unwrap();

    assert_eq!(result.data.len(), 2);
    assert_eq!(result.meta.total_count, 2);
}

#[tokio::test]
async fn test_list_articles_pagination() {
    let mut api = MockApi::new().await;

    let articles = vec![fixtures::article(1), fixtures::article(2), fixtures::article(3)];

    let data = serde_json::to_string(&articles).unwrap();
    let meta = meta_json(0, 50, 1, 3);
    let response = format!(r#"{{"Data": {}, {}}}"#, data, meta);

    let _mock = api.mock_get("/articles", &response);

    let result = api.client.articles().list(None).await.unwrap();

    assert_eq!(result.data.len(), 3);
}

// =============================================================================
// Manual Pagination Iteration Tests
// =============================================================================

#[tokio::test]
async fn test_manual_pagination_all_pages() {
    let mut api = MockApi::new().await;

    // Page 0
    let data0 = serde_json::to_string(&fixtures::customers(2)).unwrap();
    let page0_response = format!(r#"{{"Data": {}, {}}}"#, data0, meta_json(0, 2, 3, 5));

    // Page 1
    let data1 = serde_json::to_string(&vec![fixtures::customer(3), fixtures::customer(4)]).unwrap();
    let page1_response = format!(r#"{{"Data": {}, {}}}"#, data1, meta_json(1, 2, 3, 5));

    // Page 2 (last)
    let data2 = serde_json::to_string(&vec![fixtures::customer(5)]).unwrap();
    let page2_response = format!(r#"{{"Data": {}, {}}}"#, data2, meta_json(2, 2, 3, 5));

    let _mock0 = api.mock_get_with_query("/customers", vec![("page", "0"), ("pagesize", "2")], &page0_response);
    let _mock1 = api.mock_get_with_query("/customers", vec![("page", "1"), ("pagesize", "2")], &page1_response);
    let _mock2 = api.mock_get_with_query("/customers", vec![("page", "2"), ("pagesize", "2")], &page2_response);

    // Manually iterate through all pages
    let mut all_customers = Vec::new();
    let mut page = 0u32;
    let page_size = 2u32;

    loop {
        let params = PaginationParams::new().page(page).pagesize(page_size);
        let result = api.client.customers().list(Some(params)).await.unwrap();

        all_customers.extend(result.data);

        if !result.meta.has_next_page {
            break;
        }
        page += 1;
    }

    assert_eq!(all_customers.len(), 5);
    assert_eq!(all_customers[0].id, Some("cust-001".to_string()));
    assert_eq!(all_customers[4].id, Some("cust-005".to_string()));
}

// =============================================================================
// Pagination Edge Cases
// =============================================================================

#[tokio::test]
async fn test_pagination_single_item() {
    let mut api = MockApi::new().await;

    let data = serde_json::to_string(&vec![fixtures::customer(1)]).unwrap();
    let meta = meta_json(0, 50, 1, 1);
    let response = format!(r#"{{"Data": {}, {}}}"#, data, meta);

    let _mock = api.mock_get("/customers", &response);

    let result = api.client.customers().list(None).await.unwrap();

    assert_eq!(result.data.len(), 1);
    assert_eq!(result.meta.total_count, 1);
    assert_eq!(result.meta.total_pages, 1);
}

#[tokio::test]
async fn test_pagination_exact_page_boundary() {
    let mut api = MockApi::new().await;

    // 100 items with page size 50 = exactly 2 pages
    let data = serde_json::to_string(&fixtures::customers(50)).unwrap();
    let meta = meta_json(0, 50, 2, 100);
    let response = format!(r#"{{"Data": {}, {}}}"#, data, meta);

    let _mock = api.mock_get("/customers", &response);

    let result = api.client.customers().list(None).await.unwrap();

    assert_eq!(result.data.len(), 50);
    assert_eq!(result.meta.total_pages, 2);
    assert!(result.meta.has_next_page);
}

#[tokio::test]
async fn test_pagination_large_page_size() {
    let mut api = MockApi::new().await;

    let data = serde_json::to_string(&fixtures::customers(10)).unwrap();
    let meta = meta_json(0, 1000, 1, 10);
    let response = format!(r#"{{"Data": {}, {}}}"#, data, meta);

    let _mock = api.mock_get_with_query("/customers", vec![("page", "0"), ("pagesize", "1000")], &response);

    let params = PaginationParams::new().page(0).pagesize(1000);
    let result = api.client.customers().list(Some(params)).await.unwrap();

    assert_eq!(result.data.len(), 10);
    assert_eq!(result.meta.page_size, 1000);
}

// =============================================================================
// Pagination Error Handling
// =============================================================================

#[tokio::test]
async fn test_pagination_error_mid_iteration() {
    let mut api = MockApi::new().await;

    // Page 0 succeeds
    let data = serde_json::to_string(&fixtures::customers(2)).unwrap();
    let page0_response = format!(r#"{{"Data": {}, {}}}"#, data, meta_json(0, 2, 3, 5));

    let _mock0 = api.mock_get_with_query("/customers", vec![("page", "0"), ("pagesize", "2")], &page0_response);

    // Page 1 fails with 500
    let _mock1 = api.mock_error("GET", "/customers?page=1&pagesize=2", 500, r#"{"Message": "Server Error"}"#);

    // First page works
    let params0 = PaginationParams::new().page(0).pagesize(2);
    let result0 = api.client.customers().list(Some(params0)).await;
    assert!(result0.is_ok());

    // Second page fails
    let params1 = PaginationParams::new().page(1).pagesize(2);
    let result1 = api.client.customers().list(Some(params1)).await;
    assert!(result1.is_err());
}

#[tokio::test]
async fn test_pagination_invalid_page_number() {
    let mut api = MockApi::new().await;

    // Requesting page beyond available data
    let meta = meta_json(999, 50, 1, 10);
    let response = format!(r#"{{"Data": [], {}}}"#, meta);

    let _mock = api.mock_get_with_query("/customers", vec![("page", "999"), ("pagesize", "50")], &response);

    let params = PaginationParams::new().page(999).pagesize(50);
    let result = api.client.customers().list(Some(params)).await.unwrap();

    assert!(result.data.is_empty());
}

// =============================================================================
// PaginationParams Builder Tests
// =============================================================================

#[test]
fn test_pagination_params_builder() {
    let params = PaginationParams::new();
    assert_eq!(params.page, None);
    assert_eq!(params.pagesize, None);
}

#[test]
fn test_pagination_params_with_values() {
    let params = PaginationParams::new().page(5).pagesize(100);
    assert_eq!(params.page, Some(5));
    assert_eq!(params.pagesize, Some(100));
}

#[test]
fn test_pagination_params_chaining() {
    let params = PaginationParams::new()
        .page(0)
        .pagesize(25)
        .page(1); // Override previous page

    assert_eq!(params.page, Some(1));
    assert_eq!(params.pagesize, Some(25));
}

// =============================================================================
// Future Stream Tests (for when Stream pagination is implemented)
// =============================================================================
//
// These tests document the expected behavior for Stream-based pagination.
// Enable once the feature is implemented.
//
// #[tokio::test]
// async fn test_stream_yields_all_items() {
//     use futures::StreamExt;
//
//     let mut api = MockApi::new().await;
//     api.mock_paginated("/customers", vec![
//         (fixtures::customers(2), true),
//         (fixtures::customers(2), true),
//         (fixtures::customers(2), false),
//     ]);
//
//     let stream = api.client.customers().list_stream();
//     let customers: Vec<Customer> = stream.try_collect().await.unwrap();
//
//     assert_eq!(customers.len(), 6);
// }
//
// #[tokio::test]
// async fn test_stream_lazy_fetching() {
//     let mut stream = api.client.customers().list_stream();
//
//     // Only first page should be fetched initially
//     let _ = stream.next().await;
//     page1_mock.assert();
//     page2_mock.assert_not_called();
// }
