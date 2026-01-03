//! Tests for type definitions and serialization.
//!
//! These tests verify that:
//! - Types serialize correctly to JSON with PascalCase
//! - Types deserialize correctly from API responses
//! - Default values work as expected
//! - Optional fields are handled correctly
//! - All expected fields are present

use spiris::{
    AccessToken, Address, Article, Customer, Invoice, InvoiceRow, Money, PaginatedResponse,
    PaginationParams, QueryParams, ResponseMetadata,
};

/// Helper to create Option<Money> for test assertions
fn some_money(val: f64) -> Option<Money> {
    #[cfg(feature = "decimal")]
    {
        use std::str::FromStr;
        Some(rust_decimal::Decimal::from_str(&val.to_string()).expect("Invalid decimal"))
    }
    #[cfg(not(feature = "decimal"))]
    {
        Some(val)
    }
}

// =============================================================================
// Customer Type Tests
// =============================================================================

#[test]
fn test_customer_default() {
    let customer = Customer::default();

    assert!(customer.id.is_none());
    assert!(customer.name.is_none());
    assert!(customer.email.is_none());
    assert!(customer.is_active.is_none());
}

#[test]
fn test_customer_with_fields() {
    let customer = Customer {
        id: Some("cust-001".to_string()),
        name: Some("Acme Corp".to_string()),
        email: Some("info@acme.com".to_string()),
        phone: Some("+1234567890".to_string()),
        is_active: Some(true),
        customer_number: Some("C001".to_string()),
        ..Default::default()
    };

    assert_eq!(customer.id, Some("cust-001".to_string()));
    assert_eq!(customer.name, Some("Acme Corp".to_string()));
    assert_eq!(customer.is_active, Some(true));
}

#[test]
fn test_customer_serialization_pascal_case() {
    let customer = Customer {
        id: Some("cust-001".to_string()),
        name: Some("Test".to_string()),
        is_active: Some(true),
        customer_number: Some("C001".to_string()),
        ..Default::default()
    };

    let json = serde_json::to_string(&customer).unwrap();

    // Verify PascalCase field names
    assert!(json.contains("\"Id\""));
    assert!(json.contains("\"Name\""));
    assert!(json.contains("\"IsActive\""));
    assert!(json.contains("\"CustomerNumber\""));
}

#[test]
fn test_customer_deserialization_pascal_case() {
    let json = r#"{
        "Id": "cust-001",
        "Name": "Test Customer",
        "Email": "test@example.com",
        "IsActive": true,
        "CustomerNumber": "C001"
    }"#;

    let customer: Customer = serde_json::from_str(json).unwrap();

    assert_eq!(customer.id, Some("cust-001".to_string()));
    assert_eq!(customer.name, Some("Test Customer".to_string()));
    assert_eq!(customer.email, Some("test@example.com".to_string()));
    assert_eq!(customer.is_active, Some(true));
}

#[test]
fn test_customer_skip_none_fields() {
    let customer = Customer {
        name: Some("Test".to_string()),
        ..Default::default()
    };

    let json = serde_json::to_string(&customer).unwrap();

    // None fields should be skipped
    assert!(!json.contains("\"Id\""));
    assert!(json.contains("\"Name\""));
    assert!(!json.contains("\"Email\""));
}

#[test]
fn test_customer_clone() {
    let customer = Customer {
        id: Some("cust-001".to_string()),
        name: Some("Test".to_string()),
        ..Default::default()
    };

    let cloned = customer.clone();

    assert_eq!(cloned.id, customer.id);
    assert_eq!(cloned.name, customer.name);
}

#[test]
fn test_customer_debug() {
    let customer = Customer {
        id: Some("cust-001".to_string()),
        ..Default::default()
    };

    let debug = format!("{:?}", customer);

    assert!(debug.contains("Customer"));
    assert!(debug.contains("cust-001"));
}

// =============================================================================
// Invoice Type Tests
// =============================================================================

#[test]
fn test_invoice_default() {
    let invoice = Invoice::default();

    assert!(invoice.id.is_none());
    assert!(invoice.invoice_number.is_none());
    assert!(invoice.customer_id.is_none());
    assert!(invoice.rows.is_empty());
}

#[test]
fn test_invoice_with_rows() {
    let invoice = Invoice {
        id: Some("inv-001".to_string()),
        customer_id: Some("cust-001".to_string()),
        rows: vec![
            InvoiceRow {
                article_id: Some("art-001".to_string()),
                quantity: some_money(2.0),
                unit_price: some_money(100.0),
                ..Default::default()
            },
            InvoiceRow {
                article_id: Some("art-002".to_string()),
                quantity: some_money(1.0),
                unit_price: some_money(50.0),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    assert_eq!(invoice.rows.len(), 2);
    assert_eq!(invoice.rows[0].article_id, Some("art-001".to_string()));
}

#[test]
fn test_invoice_serialization() {
    let invoice = Invoice {
        id: Some("inv-001".to_string()),
        invoice_number: Some("1001".to_string()),
        total_amount: some_money(250.0),
        rows: vec![],
        ..Default::default()
    };

    let json = serde_json::to_string(&invoice).unwrap();

    assert!(json.contains("\"Id\""));
    assert!(json.contains("\"InvoiceNumber\""));
    assert!(json.contains("\"TotalAmount\""));
    assert!(json.contains("\"Rows\""));
}

#[test]
fn test_invoice_deserialization() {
    let json = r#"{
        "Id": "inv-001",
        "InvoiceNumber": "1001",
        "CustomerId": "cust-001",
        "TotalAmount": 500.00,
        "Rows": [
            {
                "ArticleId": "art-001",
                "Quantity": 5,
                "UnitPrice": 100.00
            }
        ]
    }"#;

    let invoice: Invoice = serde_json::from_str(json).unwrap();

    assert_eq!(invoice.id, Some("inv-001".to_string()));
    assert_eq!(invoice.invoice_number, Some("1001".to_string()));
    assert_eq!(invoice.total_amount, some_money(500.0));
    assert_eq!(invoice.rows.len(), 1);
}

// =============================================================================
// InvoiceRow Type Tests
// =============================================================================

#[test]
fn test_invoice_row_default() {
    let row = InvoiceRow::default();

    assert!(row.article_id.is_none());
    assert!(row.quantity.is_none());
    assert!(row.unit_price.is_none());
}

#[test]
fn test_invoice_row_with_values() {
    let row = InvoiceRow {
        article_id: Some("art-001".to_string()),
        quantity: some_money(3.0),
        unit_price: some_money(25.50),
        discount_percentage: some_money(10.0),
        ..Default::default()
    };

    assert_eq!(row.article_id, Some("art-001".to_string()));
    assert_eq!(row.quantity, some_money(3.0));
    assert_eq!(row.unit_price, some_money(25.50));
    assert_eq!(row.discount_percentage, some_money(10.0));
}

#[test]
fn test_invoice_row_serialization() {
    let row = InvoiceRow {
        article_id: Some("art-001".to_string()),
        quantity: some_money(2.0),
        unit_price: some_money(100.0),
        ..Default::default()
    };

    let json = serde_json::to_string(&row).unwrap();

    assert!(json.contains("\"ArticleId\""));
    assert!(json.contains("\"Quantity\""));
    assert!(json.contains("\"UnitPrice\""));
}

// =============================================================================
// Article Type Tests
// =============================================================================

#[test]
fn test_article_default() {
    let article = Article::default();

    assert!(article.id.is_none());
    assert!(article.name.is_none());
    assert!(article.sales_price.is_none());
}

#[test]
fn test_article_with_fields() {
    let article = Article {
        id: Some("art-001".to_string()),
        article_number: Some("ART-001".to_string()),
        name: Some("Widget".to_string()),
        sales_price: some_money(99.99),
        purchase_price: some_money(50.00),
        is_active: Some(true),
        ..Default::default()
    };

    assert_eq!(article.sales_price, some_money(99.99));
    assert_eq!(article.purchase_price, some_money(50.00));
}

#[test]
fn test_article_serialization() {
    let article = Article {
        id: Some("art-001".to_string()),
        name: Some("Test".to_string()),
        sales_price: some_money(49.99),
        is_active: Some(true),
        ..Default::default()
    };

    let json = serde_json::to_string(&article).unwrap();

    assert!(json.contains("\"Id\""));
    assert!(json.contains("\"Name\""));
    assert!(json.contains("\"SalesPrice\""));
    assert!(json.contains("\"IsActive\""));
}

#[test]
fn test_article_deserialization() {
    let json = r#"{
        "Id": "art-001",
        "ArticleNumber": "ART-001",
        "Name": "Test Article",
        "SalesPrice": 149.99,
        "IsActive": true
    }"#;

    let article: Article = serde_json::from_str(json).unwrap();

    assert_eq!(article.id, Some("art-001".to_string()));
    assert_eq!(article.article_number, Some("ART-001".to_string()));
    assert_eq!(article.sales_price, some_money(149.99));
}

// =============================================================================
// Address Type Tests
// =============================================================================

#[test]
fn test_address_default() {
    let address = Address::default();

    assert!(address.address1.is_none());
    assert!(address.city.is_none());
    assert!(address.postal_code.is_none());
}

#[test]
fn test_address_with_fields() {
    let address = Address {
        address1: Some("123 Main St".to_string()),
        city: Some("Stockholm".to_string()),
        postal_code: Some("12345".to_string()),
        country_code: Some("SE".to_string()),
        ..Default::default()
    };

    assert_eq!(address.address1, Some("123 Main St".to_string()));
    assert_eq!(address.city, Some("Stockholm".to_string()));
}

#[test]
fn test_address_serialization() {
    let address = Address {
        address1: Some("123 Main St".to_string()),
        city: Some("Stockholm".to_string()),
        ..Default::default()
    };

    let json = serde_json::to_string(&address).unwrap();

    assert!(json.contains("\"Address1\""));
    assert!(json.contains("\"City\""));
}

// =============================================================================
// PaginatedResponse Type Tests
// =============================================================================

#[test]
fn test_paginated_response_deserialization() {
    let json = r#"{
        "Data": [
            {"Id": "cust-001", "Name": "Customer 1"},
            {"Id": "cust-002", "Name": "Customer 2"}
        ],
        "Meta": {
            "CurrentPage": 0,
            "PageSize": 50,
            "TotalPages": 1,
            "TotalCount": 2,
            "HasNextPage": false,
            "HasPreviousPage": false
        }
    }"#;

    let response: PaginatedResponse<Customer> = serde_json::from_str(json).unwrap();

    assert_eq!(response.data.len(), 2);
    assert_eq!(response.meta.current_page, 0);
    assert_eq!(response.meta.total_count, 2);
    assert!(!response.meta.has_next_page);
}

#[test]
fn test_paginated_response_empty() {
    let json = r#"{
        "Data": [],
        "Meta": {
            "CurrentPage": 0,
            "PageSize": 50,
            "TotalPages": 0,
            "TotalCount": 0,
            "HasNextPage": false,
            "HasPreviousPage": false
        }
    }"#;

    let response: PaginatedResponse<Customer> = serde_json::from_str(json).unwrap();

    assert!(response.data.is_empty());
    assert_eq!(response.meta.total_count, 0);
}

// =============================================================================
// ResponseMetadata Type Tests
// =============================================================================

#[test]
fn test_response_metadata_deserialization() {
    let json = r#"{
        "CurrentPage": 2,
        "PageSize": 25,
        "TotalPages": 10,
        "TotalCount": 245,
        "HasNextPage": true,
        "HasPreviousPage": true
    }"#;

    let meta: ResponseMetadata = serde_json::from_str(json).unwrap();

    assert_eq!(meta.current_page, 2);
    assert_eq!(meta.page_size, 25);
    assert_eq!(meta.total_pages, 10);
    assert_eq!(meta.total_count, 245);
    assert!(meta.has_next_page);
    assert!(meta.has_previous_page);
}

#[test]
fn test_response_metadata_first_page() {
    let json = r#"{
        "CurrentPage": 0,
        "PageSize": 50,
        "TotalPages": 5,
        "TotalCount": 225,
        "HasNextPage": true,
        "HasPreviousPage": false
    }"#;

    let meta: ResponseMetadata = serde_json::from_str(json).unwrap();

    assert_eq!(meta.current_page, 0);
    assert!(meta.has_next_page);
    assert!(!meta.has_previous_page);
}

#[test]
fn test_response_metadata_last_page() {
    let json = r#"{
        "CurrentPage": 4,
        "PageSize": 50,
        "TotalPages": 5,
        "TotalCount": 225,
        "HasNextPage": false,
        "HasPreviousPage": true
    }"#;

    let meta: ResponseMetadata = serde_json::from_str(json).unwrap();

    assert_eq!(meta.current_page, 4);
    assert!(!meta.has_next_page);
    assert!(meta.has_previous_page);
}

// =============================================================================
// PaginationParams Type Tests
// =============================================================================

#[test]
fn test_pagination_params_default() {
    let params = PaginationParams::default();

    assert!(params.page.is_none());
    assert!(params.pagesize.is_none());
}

#[test]
fn test_pagination_params_builder() {
    let params = PaginationParams::new().page(5).pagesize(100);

    assert_eq!(params.page, Some(5));
    assert_eq!(params.pagesize, Some(100));
}

#[test]
fn test_pagination_params_serialization() {
    let params = PaginationParams::new().page(2).pagesize(25);
    let json = serde_json::to_string(&params).unwrap();

    assert!(json.contains("\"page\":2"));
    assert!(json.contains("\"pagesize\":25"));
}

// =============================================================================
// QueryParams Type Tests
// =============================================================================

#[test]
fn test_query_params_default() {
    let params = QueryParams::default();

    assert!(params.filter.is_none());
    assert!(params.select.is_none());
    assert!(params.extra.is_empty());
}

#[test]
fn test_query_params_builder() {
    let params = QueryParams::new()
        .filter("IsActive eq true")
        .select("Id,Name")
        .param("orderBy", "Name");

    assert_eq!(params.filter, Some("IsActive eq true".to_string()));
    assert_eq!(params.select, Some("Id,Name".to_string()));
    assert_eq!(params.extra.get("orderBy"), Some(&"Name".to_string()));
}

#[test]
fn test_query_params_serialization() {
    let params = QueryParams::new()
        .filter("Status eq 'Active'")
        .select("Id,Name");

    let json = serde_json::to_string(&params).unwrap();

    assert!(json.contains("filter"));
    assert!(json.contains("select"));
}

// =============================================================================
// AccessToken Type Tests
// =============================================================================

#[test]
fn test_access_token_creation() {
    let token = AccessToken::new("test_token".to_string(), 3600, None);

    assert_eq!(token.token, "test_token");
    assert_eq!(token.token_type, "Bearer");
    assert!(token.refresh_token.is_none());
}

#[test]
fn test_access_token_with_refresh() {
    let token = AccessToken::new(
        "access".to_string(),
        3600,
        Some("refresh".to_string()),
    );

    assert_eq!(token.token, "access");
    assert_eq!(token.refresh_token, Some("refresh".to_string()));
}

#[test]
fn test_access_token_authorization_header() {
    let token = AccessToken::new("abc123".to_string(), 3600, None);

    assert_eq!(token.authorization_header(), "Bearer abc123");
}

#[test]
fn test_access_token_serialization() {
    let token = AccessToken::new(
        "test".to_string(),
        3600,
        Some("refresh".to_string()),
    );

    let json = serde_json::to_string(&token).unwrap();
    let deserialized: AccessToken = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.token, token.token);
    assert_eq!(deserialized.refresh_token, token.refresh_token);
}

// =============================================================================
// Type Trait Tests
// =============================================================================

#[test]
fn test_types_are_send() {
    fn assert_send<T: Send>() {}

    assert_send::<Customer>();
    assert_send::<Invoice>();
    assert_send::<Article>();
    assert_send::<Address>();
    assert_send::<InvoiceRow>();
    assert_send::<AccessToken>();
}

#[test]
fn test_types_are_sync() {
    fn assert_sync<T: Sync>() {}

    assert_sync::<Customer>();
    assert_sync::<Invoice>();
    assert_sync::<Article>();
    assert_sync::<Address>();
    assert_sync::<InvoiceRow>();
    assert_sync::<AccessToken>();
}

#[test]
fn test_types_are_clone() {
    let customer = Customer {
        name: Some("Test".to_string()),
        ..Default::default()
    };
    let _ = customer.clone();

    let invoice = Invoice {
        rows: vec![],
        ..Default::default()
    };
    let _ = invoice.clone();

    let article = Article::default();
    let _ = article.clone();
}

#[test]
fn test_types_are_debug() {
    let customer = Customer::default();
    let _ = format!("{:?}", customer);

    let invoice = Invoice {
        rows: vec![],
        ..Default::default()
    };
    let _ = format!("{:?}", invoice);

    let article = Article::default();
    let _ = format!("{:?}", article);
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_empty_string_fields() {
    let customer = Customer {
        name: Some("".to_string()),
        email: Some("".to_string()),
        ..Default::default()
    };

    let json = serde_json::to_string(&customer).unwrap();
    let deserialized: Customer = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.name, Some("".to_string()));
}

#[test]
fn test_unicode_in_fields() {
    let customer = Customer {
        name: Some("Företag AB".to_string()),
        ..Default::default()
    };

    let json = serde_json::to_string(&customer).unwrap();
    let deserialized: Customer = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.name, Some("Företag AB".to_string()));
}

#[test]
fn test_special_characters_in_fields() {
    let customer = Customer {
        name: Some("O'Brien & Associates".to_string()),
        ..Default::default()
    };

    let json = serde_json::to_string(&customer).unwrap();
    let deserialized: Customer = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.name, Some("O'Brien & Associates".to_string()));
}

#[test]
fn test_very_long_string_field() {
    let long_name = "A".repeat(1000);
    let customer = Customer {
        name: Some(long_name.clone()),
        ..Default::default()
    };

    let json = serde_json::to_string(&customer).unwrap();
    let deserialized: Customer = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.name, Some(long_name));
}

#[test]
fn test_null_vs_missing_fields() {
    // Field explicitly set to null
    let json_with_null = r#"{"Id": "test", "Name": null}"#;
    let customer: Customer = serde_json::from_str(json_with_null).unwrap();
    assert!(customer.name.is_none());

    // Field missing entirely
    let json_missing = r#"{"Id": "test"}"#;
    let customer: Customer = serde_json::from_str(json_missing).unwrap();
    assert!(customer.name.is_none());
}
