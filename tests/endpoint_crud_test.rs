// Skip these tests when decimal feature is enabled (uses f64 literals)
#![cfg(not(feature = "decimal"))]
//! Comprehensive CRUD operation tests for all endpoints.
//!
//! These tests verify that each endpoint correctly implements:
//! - List (GET collection)
//! - Get (GET single item)
//! - Create (POST)
//! - Update (PUT)
//! - Delete (DELETE)
//! - Search (GET with filters)

mod mock_server;

use mock_server::{fixtures, meta_json, MockApi};
use spiris::{Article, Customer, Invoice, InvoiceRow, PaginationParams};

// =============================================================================
// Customer Endpoint CRUD Tests
// =============================================================================

#[tokio::test]
async fn test_customer_list() {
    let mut api = MockApi::new().await;

    let customers = vec![fixtures::customer(1), fixtures::customer(2)];
    let data = serde_json::to_string(&customers).unwrap();
    let meta = meta_json(0, 50, 1, 2);
    let response = format!(r#"{{"Data": {}, {}}}"#, data, meta);

    let _mock = api.mock_get("/customers", &response);

    let result = api.client.customers().list(None).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.data.len(), 2);
    assert_eq!(response.meta.total_count, 2);
}

#[tokio::test]
async fn test_customer_list_with_pagination() {
    let mut api = MockApi::new().await;

    let customers = vec![fixtures::customer(1)];
    let data = serde_json::to_string(&customers).unwrap();
    let meta = meta_json(2, 10, 5, 45);
    let response = format!(r#"{{"Data": {}, {}}}"#, data, meta);

    let _mock = api.mock_get_with_query(
        "/customers",
        vec![("page", "2"), ("pagesize", "10")],
        &response,
    );

    let params = PaginationParams::new().page(2).pagesize(10);
    let result = api.client.customers().list(Some(params)).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.meta.current_page, 2);
    assert_eq!(response.meta.page_size, 10);
}

#[tokio::test]
async fn test_customer_get() {
    let mut api = MockApi::new().await;

    let customer = fixtures::customer(1);
    let response = serde_json::to_string(&customer).unwrap();

    let _mock = api.mock_get("/customers/cust-001", &response);

    let result = api.client.customers().get("cust-001").await;

    assert!(result.is_ok());
    let customer = result.unwrap();
    assert_eq!(customer.id, Some("cust-001".to_string()));
    assert_eq!(customer.name, Some("Test Customer 1".to_string()));
}

#[tokio::test]
async fn test_customer_create() {
    let mut api = MockApi::new().await;

    let new_customer = Customer {
        name: Some("New Customer".to_string()),
        email: Some("new@example.com".to_string()),
        is_active: Some(true),
        ..Default::default()
    };

    let created_customer = Customer {
        id: Some("cust-new".to_string()),
        name: Some("New Customer".to_string()),
        email: Some("new@example.com".to_string()),
        is_active: Some(true),
        ..Default::default()
    };

    let response = serde_json::to_string(&created_customer).unwrap();

    let _mock = api
        .server
        .mock("POST", "/customers")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(&response)
        .create();

    let result = api.client.customers().create(&new_customer).await;

    assert!(result.is_ok());
    let customer = result.unwrap();
    assert!(customer.id.is_some());
    assert_eq!(customer.name, Some("New Customer".to_string()));
}

#[tokio::test]
async fn test_customer_update() {
    let mut api = MockApi::new().await;

    let updated_customer = Customer {
        id: Some("cust-001".to_string()),
        name: Some("Updated Name".to_string()),
        email: Some("updated@example.com".to_string()),
        is_active: Some(true),
        ..Default::default()
    };

    let response = serde_json::to_string(&updated_customer).unwrap();

    let _mock = api
        .server
        .mock("PUT", "/customers/cust-001")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(&response)
        .create();

    let result = api
        .client
        .customers()
        .update("cust-001", &updated_customer)
        .await;

    assert!(result.is_ok());
    let customer = result.unwrap();
    assert_eq!(customer.name, Some("Updated Name".to_string()));
}

#[tokio::test]
async fn test_customer_delete() {
    let mut api = MockApi::new().await;

    let _mock = api
        .server
        .mock("DELETE", "/customers/cust-001")
        .with_status(204)
        .create();

    let result = api.client.customers().delete("cust-001").await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_customer_search() {
    let mut api = MockApi::new().await;

    let customers = vec![fixtures::customer(1)];
    let data = serde_json::to_string(&customers).unwrap();
    let meta = meta_json(0, 50, 1, 1);
    let response = format!(r#"{{"Data": {}, {}}}"#, data, meta);

    // Use a simpler matcher that accepts any query
    let _mock = api
        .server
        .mock("GET", "/customers")
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(&response)
        .create();

    let query = spiris::QueryParams::new().filter("IsActive eq true");
    let result = api.client.customers().search(query, None).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.data.len(), 1);
}

// =============================================================================
// Invoice Endpoint CRUD Tests
// =============================================================================

#[tokio::test]
async fn test_invoice_list() {
    let mut api = MockApi::new().await;

    let invoices = vec![
        fixtures::invoice(1, "cust-001"),
        fixtures::invoice(2, "cust-002"),
    ];
    let data = serde_json::to_string(&invoices).unwrap();
    let meta = meta_json(0, 50, 1, 2);
    let response = format!(r#"{{"Data": {}, {}}}"#, data, meta);

    let _mock = api.mock_get("/customerinvoices", &response);

    let result = api.client.invoices().list(None).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.data.len(), 2);
}

#[tokio::test]
async fn test_invoice_get() {
    let mut api = MockApi::new().await;

    let invoice = fixtures::invoice(1, "cust-001");
    let response = serde_json::to_string(&invoice).unwrap();

    let _mock = api.mock_get("/customerinvoices/inv-001", &response);

    let result = api.client.invoices().get("inv-001").await;

    assert!(result.is_ok());
    let invoice = result.unwrap();
    assert_eq!(invoice.id, Some("inv-001".to_string()));
}

#[tokio::test]
async fn test_invoice_create() {
    let mut api = MockApi::new().await;

    let new_invoice = Invoice {
        customer_id: Some("cust-001".to_string()),
        rows: vec![InvoiceRow {
            article_id: Some("art-001".to_string()),
            quantity: Some(2.0),
            unit_price: Some(100.0),
            ..Default::default()
        }],
        ..Default::default()
    };

    let created_invoice = Invoice {
        id: Some("inv-new".to_string()),
        invoice_number: Some("1001".to_string()),
        customer_id: Some("cust-001".to_string()),
        total_amount: Some(200.0),
        rows: vec![],
        ..Default::default()
    };

    let response = serde_json::to_string(&created_invoice).unwrap();

    let _mock = api
        .server
        .mock("POST", "/customerinvoices")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(&response)
        .create();

    let result = api.client.invoices().create(&new_invoice).await;

    assert!(result.is_ok());
    let invoice = result.unwrap();
    assert!(invoice.id.is_some());
}

#[tokio::test]
async fn test_invoice_update() {
    let mut api = MockApi::new().await;

    let updated_invoice = Invoice {
        id: Some("inv-001".to_string()),
        customer_id: Some("cust-001".to_string()),
        rows: vec![],
        ..Default::default()
    };

    let response = serde_json::to_string(&updated_invoice).unwrap();

    let _mock = api
        .server
        .mock("PUT", "/customerinvoices/inv-001")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(&response)
        .create();

    let result = api
        .client
        .invoices()
        .update("inv-001", &updated_invoice)
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_invoice_delete() {
    let mut api = MockApi::new().await;

    let _mock = api
        .server
        .mock("DELETE", "/customerinvoices/inv-001")
        .with_status(204)
        .create();

    let result = api.client.invoices().delete("inv-001").await;

    assert!(result.is_ok());
}

// =============================================================================
// Article Endpoint CRUD Tests
// =============================================================================

#[tokio::test]
async fn test_article_list() {
    let mut api = MockApi::new().await;

    let articles = vec![fixtures::article(1), fixtures::article(2)];
    let data = serde_json::to_string(&articles).unwrap();
    let meta = meta_json(0, 50, 1, 2);
    let response = format!(r#"{{"Data": {}, {}}}"#, data, meta);

    let _mock = api.mock_get("/articles", &response);

    let result = api.client.articles().list(None).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.data.len(), 2);
}

#[tokio::test]
async fn test_article_get() {
    let mut api = MockApi::new().await;

    let article = fixtures::article(1);
    let response = serde_json::to_string(&article).unwrap();

    let _mock = api.mock_get("/articles/art-001", &response);

    let result = api.client.articles().get("art-001").await;

    assert!(result.is_ok());
    let article = result.unwrap();
    assert_eq!(article.id, Some("art-001".to_string()));
}

#[tokio::test]
async fn test_article_create() {
    let mut api = MockApi::new().await;

    let new_article = Article {
        name: Some("New Article".to_string()),
        article_number: Some("ART-NEW".to_string()),
        sales_price: Some(99.99),
        is_active: Some(true),
        ..Default::default()
    };

    let created_article = Article {
        id: Some("art-new".to_string()),
        name: Some("New Article".to_string()),
        article_number: Some("ART-NEW".to_string()),
        sales_price: Some(99.99),
        is_active: Some(true),
        ..Default::default()
    };

    let response = serde_json::to_string(&created_article).unwrap();

    let _mock = api
        .server
        .mock("POST", "/articles")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(&response)
        .create();

    let result = api.client.articles().create(&new_article).await;

    assert!(result.is_ok());
    let article = result.unwrap();
    assert!(article.id.is_some());
}

#[tokio::test]
async fn test_article_update() {
    let mut api = MockApi::new().await;

    let updated_article = Article {
        id: Some("art-001".to_string()),
        name: Some("Updated Article".to_string()),
        sales_price: Some(149.99),
        is_active: Some(true),
        ..Default::default()
    };

    let response = serde_json::to_string(&updated_article).unwrap();

    let _mock = api
        .server
        .mock("PUT", "/articles/art-001")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(&response)
        .create();

    let result = api
        .client
        .articles()
        .update("art-001", &updated_article)
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_article_delete() {
    let mut api = MockApi::new().await;

    let _mock = api
        .server
        .mock("DELETE", "/articles/art-001")
        .with_status(204)
        .create();

    let result = api.client.articles().delete("art-001").await;

    assert!(result.is_ok());
}

// =============================================================================
// Empty Results Tests
// =============================================================================

#[tokio::test]
async fn test_customer_list_empty() {
    let mut api = MockApi::new().await;

    let meta = meta_json(0, 50, 0, 0);
    let response = format!(r#"{{"Data": [], {}}}"#, meta);

    let _mock = api.mock_get("/customers", &response);

    let result = api.client.customers().list(None).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.data.is_empty());
    assert_eq!(response.meta.total_count, 0);
}

#[tokio::test]
async fn test_invoice_list_empty() {
    let mut api = MockApi::new().await;

    let meta = meta_json(0, 50, 0, 0);
    let response = format!(r#"{{"Data": [], {}}}"#, meta);

    let _mock = api.mock_get("/customerinvoices", &response);

    let result = api.client.invoices().list(None).await;

    assert!(result.is_ok());
    assert!(result.unwrap().data.is_empty());
}

#[tokio::test]
async fn test_article_list_empty() {
    let mut api = MockApi::new().await;

    let meta = meta_json(0, 50, 0, 0);
    let response = format!(r#"{{"Data": [], {}}}"#, meta);

    let _mock = api.mock_get("/articles", &response);

    let result = api.client.articles().list(None).await;

    assert!(result.is_ok());
    assert!(result.unwrap().data.is_empty());
}

// =============================================================================
// Not Found Tests
// =============================================================================

#[tokio::test]
async fn test_customer_get_not_found() {
    let mut api = MockApi::new().await;

    let _mock = api.mock_error(
        "GET",
        "/customers/nonexistent",
        404,
        r#"{"Message": "Customer not found"}"#,
    );

    let result = api.client.customers().get("nonexistent").await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_invoice_get_not_found() {
    let mut api = MockApi::new().await;

    let _mock = api.mock_error(
        "GET",
        "/customerinvoices/nonexistent",
        404,
        r#"{"Message": "Invoice not found"}"#,
    );

    let result = api.client.invoices().get("nonexistent").await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_article_get_not_found() {
    let mut api = MockApi::new().await;

    let _mock = api.mock_error(
        "GET",
        "/articles/nonexistent",
        404,
        r#"{"Message": "Article not found"}"#,
    );

    let result = api.client.articles().get("nonexistent").await;

    assert!(result.is_err());
}

// =============================================================================
// Create Validation Error Tests
// =============================================================================

#[tokio::test]
async fn test_customer_create_validation_error() {
    let mut api = MockApi::new().await;

    let _mock = api.mock_error(
        "POST",
        "/customers",
        400,
        r#"{"Message": "Name is required"}"#,
    );

    let invalid_customer = Customer::default();
    let result = api.client.customers().create(&invalid_customer).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_invoice_create_validation_error() {
    let mut api = MockApi::new().await;

    let _mock = api.mock_error(
        "POST",
        "/customerinvoices",
        400,
        r#"{"Message": "CustomerId is required"}"#,
    );

    let invalid_invoice = Invoice {
        rows: vec![],
        ..Default::default()
    };
    let result = api.client.invoices().create(&invalid_invoice).await;

    assert!(result.is_err());
}

// =============================================================================
// Update Conflict Tests
// =============================================================================

#[tokio::test]
async fn test_customer_update_conflict() {
    let mut api = MockApi::new().await;

    let _mock = api.mock_error(
        "PUT",
        "/customers/cust-001",
        409,
        r#"{"Message": "Resource was modified"}"#,
    );

    let customer = Customer {
        id: Some("cust-001".to_string()),
        ..Default::default()
    };
    let result = api.client.customers().update("cust-001", &customer).await;

    assert!(result.is_err());
}

// =============================================================================
// Delete Already Deleted Tests
// =============================================================================

#[tokio::test]
async fn test_customer_delete_not_found() {
    let mut api = MockApi::new().await;

    let _mock = api.mock_error(
        "DELETE",
        "/customers/nonexistent",
        404,
        r#"{"Message": "Customer not found"}"#,
    );

    let result = api.client.customers().delete("nonexistent").await;

    assert!(result.is_err());
}

// =============================================================================
// Large Dataset Tests
// =============================================================================

#[tokio::test]
async fn test_customer_list_large_page() {
    let mut api = MockApi::new().await;

    // Generate 50 customers
    let customers: Vec<Customer> = (1..=50).map(|i| fixtures::customer(i)).collect();
    let data = serde_json::to_string(&customers).unwrap();
    let meta = meta_json(0, 50, 10, 500);
    let response = format!(r#"{{"Data": {}, {}}}"#, data, meta);

    let _mock = api.mock_get("/customers", &response);

    let result = api.client.customers().list(None).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.data.len(), 50);
    assert_eq!(response.meta.total_count, 500);
    assert!(response.meta.has_next_page);
}

// =============================================================================
// Special Characters in IDs Tests
// =============================================================================

#[tokio::test]
async fn test_customer_get_with_special_id() {
    let mut api = MockApi::new().await;

    let customer = Customer {
        id: Some("cust-abc-123-xyz".to_string()),
        name: Some("Special Customer".to_string()),
        ..Default::default()
    };
    let response = serde_json::to_string(&customer).unwrap();

    let _mock = api.mock_get("/customers/cust-abc-123-xyz", &response);

    let result = api.client.customers().get("cust-abc-123-xyz").await;

    assert!(result.is_ok());
}

// =============================================================================
// Concurrent CRUD Operations Tests
// =============================================================================

#[tokio::test]
async fn test_concurrent_list_operations() {
    let mut api = MockApi::new().await;

    let customers = vec![fixtures::customer(1)];
    let cust_data = serde_json::to_string(&customers).unwrap();
    let cust_response = format!(r#"{{"Data": {}, {}}}"#, cust_data, meta_json(0, 50, 1, 1));

    let articles = vec![fixtures::article(1)];
    let art_data = serde_json::to_string(&articles).unwrap();
    let art_response = format!(r#"{{"Data": {}, {}}}"#, art_data, meta_json(0, 50, 1, 1));

    let _mock1 = api
        .server
        .mock("GET", "/customers")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(&cust_response)
        .create();

    let _mock2 = api
        .server
        .mock("GET", "/articles")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(&art_response)
        .create();

    let customers_ep = api.client.customers();
    let articles_ep = api.client.articles();

    let (cust_result, art_result) = tokio::join!(customers_ep.list(None), articles_ep.list(None));

    assert!(cust_result.is_ok());
    assert!(art_result.is_ok());
}
