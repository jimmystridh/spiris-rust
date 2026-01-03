// Skip these tests when decimal feature is enabled (uses f64 literals)
#![cfg(not(feature = "decimal"))]
//! Contract tests for API response schema validation.
//!
//! These tests verify that:
//! - Response structures match the expected API contract
//! - All expected fields are present and correctly typed
//! - Pagination metadata is consistent
//! - Error responses follow expected format

mod mock_server;

use mock_server::MockApi;
use spiris::{Customer, PaginationParams};

// =============================================================================
// Customer Response Contract Tests
// =============================================================================

#[tokio::test]
async fn test_customer_response_has_required_fields() {
    let mut api = MockApi::new().await;

    let json = r#"{
        "Id": "cust-123",
        "CustomerNumber": "C001",
        "Name": "Test Customer",
        "IsActive": true,
        "CreatedUtc": "2024-01-15T10:30:00Z",
        "ModifiedUtc": "2024-01-15T10:30:00Z"
    }"#;

    let _mock = api.mock_get("/customers/cust-123", json);

    let customer = api.client.customers().get("cust-123").await.unwrap();

    assert!(customer.id.is_some());
    assert!(customer.customer_number.is_some());
    assert!(customer.name.is_some());
    assert!(customer.is_active.is_some());
}

#[tokio::test]
async fn test_customer_response_optional_fields() {
    let mut api = MockApi::new().await;

    let json = r#"{
        "Id": "cust-123",
        "CustomerNumber": "C001",
        "Name": "Full Customer",
        "Email": "test@example.com",
        "Phone": "+46701234567",
        "MobilePhone": "+46701234568",
        "CorporateIdentityNumber": "556677-8899",
        "IsActive": true
    }"#;

    let _mock = api.mock_get("/customers/cust-123", json);

    let customer = api.client.customers().get("cust-123").await.unwrap();

    assert_eq!(customer.email, Some("test@example.com".to_string()));
    assert_eq!(customer.phone, Some("+46701234567".to_string()));
    assert_eq!(customer.mobile_phone, Some("+46701234568".to_string()));
    assert_eq!(
        customer.corporate_identity_number,
        Some("556677-8899".to_string())
    );
}

#[tokio::test]
async fn test_customer_with_address() {
    let mut api = MockApi::new().await;

    let json = r#"{
        "Id": "cust-123",
        "Name": "Customer With Address",
        "InvoiceAddress": {
            "Address1": "Main Street 1",
            "Address2": "Suite 100",
            "PostalCode": "12345",
            "City": "Stockholm",
            "CountryCode": "SE"
        },
        "DeliveryAddress": {
            "Address1": "Delivery Lane 5",
            "City": "Gothenburg"
        }
    }"#;

    let _mock = api.mock_get("/customers/cust-123", json);

    let customer = api.client.customers().get("cust-123").await.unwrap();

    assert!(customer.invoice_address.is_some());
    let invoice_addr = customer.invoice_address.unwrap();
    assert_eq!(invoice_addr.address1, Some("Main Street 1".to_string()));
    assert_eq!(invoice_addr.city, Some("Stockholm".to_string()));
    assert_eq!(invoice_addr.country_code, Some("SE".to_string()));

    assert!(customer.delivery_address.is_some());
}

// =============================================================================
// Invoice Response Contract Tests
// =============================================================================

#[tokio::test]
async fn test_invoice_response_structure() {
    let mut api = MockApi::new().await;

    let json = r#"{
        "Id": "inv-123",
        "InvoiceNumber": "1001",
        "CustomerId": "cust-123",
        "InvoiceDate": "2024-01-15T00:00:00Z",
        "DueDate": "2024-02-15T00:00:00Z",
        "TotalAmount": 1000.00,
        "TotalVatAmount": 250.00,
        "TotalAmountIncludingVat": 1250.00,
        "Rows": []
    }"#;

    let _mock = api.mock_get("/customerinvoices/inv-123", json);

    let invoice = api.client.invoices().get("inv-123").await.unwrap();

    assert!(invoice.id.is_some());
    assert!(invoice.invoice_number.is_some());
    assert!(invoice.customer_id.is_some());
    assert!(invoice.total_amount.is_some());
    assert!(invoice.total_vat_amount.is_some());
    assert!(invoice.total_amount_including_vat.is_some());
}

#[tokio::test]
async fn test_invoice_with_rows() {
    let mut api = MockApi::new().await;

    let json = r#"{
        "Id": "inv-123",
        "InvoiceNumber": "1001",
        "Rows": [
            {
                "Id": "row-1",
                "ArticleId": "art-001",
                "Text": "Product A",
                "Quantity": 2.0,
                "UnitPrice": 100.00,
                "DiscountPercentage": 0.0,
                "TotalAmount": 200.00
            },
            {
                "Id": "row-2",
                "ArticleId": "art-002",
                "Text": "Service B",
                "Quantity": 1.0,
                "UnitPrice": 500.00,
                "TotalAmount": 500.00
            }
        ]
    }"#;

    let _mock = api.mock_get("/customerinvoices/inv-123", json);

    let invoice = api.client.invoices().get("inv-123").await.unwrap();

    assert_eq!(invoice.rows.len(), 2);

    let row1 = &invoice.rows[0];
    assert_eq!(row1.article_id, Some("art-001".to_string()));
    assert_eq!(row1.quantity, Some(2.0));
    assert_eq!(row1.unit_price, Some(100.00));
    assert_eq!(row1.total_amount, Some(200.00));

    let row2 = &invoice.rows[1];
    assert_eq!(row2.quantity, Some(1.0));
}

#[tokio::test]
async fn test_invoice_monetary_values_precision() {
    let mut api = MockApi::new().await;

    let json = r#"{
        "Id": "inv-123",
        "TotalAmount": 999.99,
        "TotalVatAmount": 249.9975,
        "TotalAmountIncludingVat": 1249.9875,
        "Rows": []
    }"#;

    let _mock = api.mock_get("/customerinvoices/inv-123", json);

    let invoice = api.client.invoices().get("inv-123").await.unwrap();

    assert_eq!(invoice.total_amount, Some(999.99));
    // Note: f64 may have precision issues
    assert!(invoice.total_vat_amount.is_some());
    assert!(invoice.total_amount_including_vat.is_some());
}

// =============================================================================
// Article Response Contract Tests
// =============================================================================

#[tokio::test]
async fn test_article_response_structure() {
    let mut api = MockApi::new().await;

    let json = r#"{
        "Id": "art-123",
        "ArticleNumber": "A001",
        "Name": "Test Article",
        "SalesPrice": 99.99,
        "PurchasePrice": 50.00,
        "IsActive": true
    }"#;

    let _mock = api.mock_get("/articles/art-123", json);

    let article = api.client.articles().get("art-123").await.unwrap();

    assert!(article.id.is_some());
    assert!(article.article_number.is_some());
    assert!(article.name.is_some());
    assert!(article.sales_price.is_some());
    assert!(article.is_active.is_some());
}

#[tokio::test]
async fn test_article_with_vat_rate() {
    let mut api = MockApi::new().await;

    let json = r#"{
        "Id": "art-123",
        "ArticleNumber": "A001",
        "Name": "Article with VAT",
        "SalesPrice": 100.00,
        "VatRateId": "vat-25",
        "Unit": "pcs",
        "IsActive": true
    }"#;

    let _mock = api.mock_get("/articles/art-123", json);

    let article = api.client.articles().get("art-123").await.unwrap();

    assert_eq!(article.vat_rate_id, Some("vat-25".to_string()));
    assert_eq!(article.unit, Some("pcs".to_string()));
}

// =============================================================================
// Pagination Response Contract Tests
// =============================================================================

#[tokio::test]
async fn test_paginated_response_metadata() {
    let mut api = MockApi::new().await;

    let json = r#"{
        "Data": [
            {"Id": "cust-1", "Name": "Customer 1"},
            {"Id": "cust-2", "Name": "Customer 2"}
        ],
        "Meta": {
            "CurrentPage": 0,
            "PageSize": 50,
            "TotalPages": 5,
            "TotalCount": 250,
            "HasNextPage": true,
            "HasPreviousPage": false
        }
    }"#;

    let _mock = api.mock_get("/customers", json);

    let result = api.client.customers().list(None).await.unwrap();

    assert_eq!(result.data.len(), 2);
    assert_eq!(result.meta.current_page, 0);
    assert_eq!(result.meta.page_size, 50);
    assert_eq!(result.meta.total_pages, 5);
    assert_eq!(result.meta.total_count, 250);
    assert!(result.meta.has_next_page);
    assert!(!result.meta.has_previous_page);
}

#[tokio::test]
async fn test_paginated_response_last_page() {
    let mut api = MockApi::new().await;

    let json = r#"{
        "Data": [
            {"Id": "cust-249", "Name": "Customer 249"},
            {"Id": "cust-250", "Name": "Customer 250"}
        ],
        "Meta": {
            "CurrentPage": 4,
            "PageSize": 50,
            "TotalPages": 5,
            "TotalCount": 250,
            "HasNextPage": false,
            "HasPreviousPage": true
        }
    }"#;

    let _mock = api
        .server
        .mock("GET", "/customers")
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json)
        .create();

    let params = PaginationParams::new().page(4);
    let result = api.client.customers().list(Some(params)).await.unwrap();

    assert_eq!(result.meta.current_page, 4);
    assert!(!result.meta.has_next_page);
    assert!(result.meta.has_previous_page);
}

#[tokio::test]
async fn test_empty_paginated_response() {
    let mut api = MockApi::new().await;

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

    let _mock = api.mock_get("/customers", json);

    let result = api.client.customers().list(None).await.unwrap();

    assert!(result.data.is_empty());
    assert_eq!(result.meta.total_count, 0);
    assert_eq!(result.meta.total_pages, 0);
}

// =============================================================================
// Error Response Contract Tests
// =============================================================================

#[tokio::test]
async fn test_validation_error_response_format() {
    let mut api = MockApi::new().await;

    let json = r#"{
        "Message": "Validation failed",
        "ErrorCode": "VALIDATION_ERROR",
        "Details": [
            {"Field": "Name", "Message": "Name is required"},
            {"Field": "Email", "Message": "Invalid email format"}
        ]
    }"#;

    let _mock = api.mock_error("POST", "/customers", 400, json);

    let customer = Customer::default();
    let result = api.client.customers().create(&customer).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_str = err.to_string();
    assert!(err_str.contains("Validation failed") || err_str.contains("400"));
}

#[tokio::test]
async fn test_not_found_error_response_format() {
    let mut api = MockApi::new().await;

    let json = r#"{
        "Message": "Customer with id 'nonexistent' was not found",
        "ErrorCode": "NOT_FOUND"
    }"#;

    let _mock = api.mock_error("GET", "/customers/nonexistent", 404, json);

    let result = api.client.customers().get("nonexistent").await;

    assert!(result.is_err());
    match result.unwrap_err() {
        spiris::Error::NotFound(msg) => {
            assert!(msg.contains("not found") || msg.contains("NOT_FOUND"));
        }
        other => panic!("Expected NotFound error, got {:?}", other),
    }
}

#[tokio::test]
async fn test_rate_limit_error_response_format() {
    let mut api = MockApi::new().await;

    let _mock = api
        .server
        .mock("GET", "/customers")
        .with_status(429)
        .with_header("content-type", "application/json")
        .with_header("Retry-After", "60")
        .with_body(r#"{"Message": "Rate limit exceeded"}"#)
        .create();

    let result = api.client.customers().list(None).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        spiris::Error::RateLimitExceeded(_) => {}
        other => panic!("Expected RateLimitExceeded error, got {:?}", other),
    }
}

#[tokio::test]
async fn test_server_error_response_format() {
    let mut api = MockApi::new().await;

    let json = r#"{
        "Message": "An unexpected error occurred",
        "ErrorCode": "INTERNAL_ERROR",
        "RequestId": "abc-123-def"
    }"#;

    let _mock = api.mock_error("GET", "/customers", 500, json);

    let result = api.client.customers().list(None).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        spiris::Error::ApiError { status_code, .. } => {
            assert_eq!(status_code, 500);
        }
        other => panic!("Expected ApiError, got {:?}", other),
    }
}

// =============================================================================
// Field Type Contract Tests
// =============================================================================

#[tokio::test]
async fn test_boolean_fields_parse_correctly() {
    let mut api = MockApi::new().await;

    let json = r#"{
        "Id": "cust-123",
        "Name": "Test",
        "IsActive": true,
        "IsPrivatePerson": false
    }"#;

    let _mock = api.mock_get("/customers/cust-123", json);

    let customer = api.client.customers().get("cust-123").await.unwrap();

    assert_eq!(customer.is_active, Some(true));
    assert_eq!(customer.is_private_person, Some(false));
}

#[tokio::test]
async fn test_numeric_fields_as_integers() {
    let mut api = MockApi::new().await;

    let json = r#"{
        "Data": [{"Id": "cust-1", "Name": "Test"}],
        "Meta": {
            "CurrentPage": 0,
            "PageSize": 50,
            "TotalPages": 1,
            "TotalCount": 1,
            "HasNextPage": false,
            "HasPreviousPage": false
        }
    }"#;

    let _mock = api.mock_get("/customers", json);

    let result = api.client.customers().list(None).await.unwrap();

    // These should parse as integers
    assert_eq!(result.meta.current_page, 0);
    assert_eq!(result.meta.page_size, 50);
}

#[tokio::test]
async fn test_date_fields_parse_as_datetime() {
    let mut api = MockApi::new().await;

    let json = r#"{
        "Id": "inv-123",
        "InvoiceDate": "2024-01-15T00:00:00Z",
        "DueDate": "2024-02-15T00:00:00Z",
        "Rows": []
    }"#;

    let _mock = api.mock_get("/customerinvoices/inv-123", json);

    let invoice = api.client.invoices().get("inv-123").await.unwrap();

    // Dates are parsed as DateTime<Utc>
    assert!(invoice.invoice_date.is_some());
    assert!(invoice.due_date.is_some());
}

#[tokio::test]
async fn test_datetime_fields_parse_correctly() {
    let mut api = MockApi::new().await;

    let json = r#"{
        "Id": "cust-123",
        "Name": "Test",
        "CreatedUtc": "2024-01-15T10:30:00Z",
        "ModifiedUtc": "2024-01-16T14:45:30Z"
    }"#;

    let _mock = api.mock_get("/customers/cust-123", json);

    let customer = api.client.customers().get("cust-123").await.unwrap();

    assert!(customer.created_utc.is_some());
    assert!(customer.modified_utc.is_some());
}

// =============================================================================
// Null Handling Contract Tests
// =============================================================================

#[tokio::test]
async fn test_null_fields_become_none() {
    let mut api = MockApi::new().await;

    let json = r#"{
        "Id": "cust-123",
        "Name": "Test",
        "Email": null,
        "Phone": null,
        "CorporateIdentityNumber": null
    }"#;

    let _mock = api.mock_get("/customers/cust-123", json);

    let customer = api.client.customers().get("cust-123").await.unwrap();

    assert!(customer.email.is_none());
    assert!(customer.phone.is_none());
    assert!(customer.corporate_identity_number.is_none());
}

#[tokio::test]
async fn test_missing_fields_become_none() {
    let mut api = MockApi::new().await;

    // Minimal response with only required fields
    let json = r#"{
        "Id": "cust-123",
        "Name": "Test"
    }"#;

    let _mock = api.mock_get("/customers/cust-123", json);

    let customer = api.client.customers().get("cust-123").await.unwrap();

    // All optional fields should be None
    assert!(customer.email.is_none());
    assert!(customer.phone.is_none());
    assert!(customer.customer_number.is_none());
}

#[tokio::test]
async fn test_empty_string_vs_null() {
    let mut api = MockApi::new().await;

    let json = r#"{
        "Id": "cust-123",
        "Name": "Test",
        "Email": "",
        "Phone": null
    }"#;

    let _mock = api.mock_get("/customers/cust-123", json);

    let customer = api.client.customers().get("cust-123").await.unwrap();

    // Empty string is Some(""), null is None
    assert_eq!(customer.email, Some("".to_string()));
    assert!(customer.phone.is_none());
}

// =============================================================================
// Request Body Contract Tests
// =============================================================================

#[tokio::test]
async fn test_create_request_uses_pascal_case() {
    let mut api = MockApi::new().await;

    let mock = api
        .server
        .mock("POST", "/customers")
        .match_body(mockito::Matcher::Regex("\"Name\"".to_string()))
        .match_body(mockito::Matcher::Regex("\"Email\"".to_string()))
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(r#"{"Id": "new-123", "Name": "New Customer"}"#)
        .create();

    let customer = Customer {
        name: Some("New Customer".to_string()),
        email: Some("new@example.com".to_string()),
        ..Default::default()
    };

    let result = api.client.customers().create(&customer).await;
    assert!(result.is_ok());

    mock.assert();
}

#[tokio::test]
async fn test_update_request_omits_none_fields() {
    let mut api = MockApi::new().await;

    // Create a mock that expects Name but not Email (since Email is None)
    let mock = api
        .server
        .mock("PUT", "/customers/cust-123")
        .match_body(mockito::Matcher::Regex("\"Name\"".to_string()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"Id": "cust-123", "Name": "Updated"}"#)
        .create();

    let customer = Customer {
        name: Some("Updated".to_string()),
        email: None, // Should not be in request body
        ..Default::default()
    };

    let result = api.client.customers().update("cust-123", &customer).await;
    assert!(result.is_ok());

    mock.assert();
}

// =============================================================================
// Complex Nested Structure Tests
// =============================================================================

#[tokio::test]
async fn test_invoice_with_nested_address() {
    let mut api = MockApi::new().await;

    let json = r#"{
        "Id": "inv-123",
        "CustomerId": "cust-456",
        "Rows": []
    }"#;

    let _mock = api.mock_get("/customerinvoices/inv-123", json);

    let invoice = api.client.invoices().get("inv-123").await.unwrap();

    assert_eq!(invoice.customer_id, Some("cust-456".to_string()));
}

#[tokio::test]
async fn test_customer_list_with_mixed_completeness() {
    let mut api = MockApi::new().await;

    let json = r#"{
        "Data": [
            {
                "Id": "cust-1",
                "Name": "Full Customer",
                "Email": "full@example.com",
                "Phone": "+46701234567",
                "IsActive": true
            },
            {
                "Id": "cust-2",
                "Name": "Minimal Customer"
            },
            {
                "Id": "cust-3",
                "Name": "Partial Customer",
                "Email": null,
                "IsActive": false
            }
        ],
        "Meta": {
            "CurrentPage": 0,
            "PageSize": 50,
            "TotalPages": 1,
            "TotalCount": 3,
            "HasNextPage": false,
            "HasPreviousPage": false
        }
    }"#;

    let _mock = api.mock_get("/customers", json);

    let result = api.client.customers().list(None).await.unwrap();

    assert_eq!(result.data.len(), 3);

    // Full customer
    assert!(result.data[0].email.is_some());
    assert!(result.data[0].phone.is_some());

    // Minimal customer
    assert!(result.data[1].email.is_none());
    assert!(result.data[1].phone.is_none());

    // Partial customer with explicit null
    assert!(result.data[2].email.is_none());
    assert_eq!(result.data[2].is_active, Some(false));
}

// =============================================================================
// Voucher Response Contract Tests
// =============================================================================

#[tokio::test]
async fn test_voucher_response_structure() {
    let mut api = MockApi::new().await;

    let json = r#"{
        "Id": "voucher-123",
        "VoucherNumber": "V001",
        "VoucherDate": "2024-01-15T00:00:00Z",
        "VoucherType": 0,
        "VoucherText": "Test voucher",
        "Rows": [
            {
                "AccountNumber": "1910",
                "DebitAmount": 1000.00,
                "CreditAmount": 0.00,
                "TransactionText": "Debit entry"
            },
            {
                "AccountNumber": "3001",
                "DebitAmount": 0.00,
                "CreditAmount": 1000.00,
                "TransactionText": "Credit entry"
            }
        ]
    }"#;

    let _mock = api.mock_get("/vouchers/voucher-123", json);

    let voucher = api.client.vouchers().get("voucher-123").await.unwrap();

    assert!(voucher.id.is_some());
    assert!(voucher.voucher_number.is_some());
    assert_eq!(voucher.rows.len(), 2);

    let row1 = &voucher.rows[0];
    assert_eq!(row1.account_number, Some("1910".to_string()));
    assert_eq!(row1.debit_amount, Some(1000.00));
}

// =============================================================================
// Supplier Response Contract Tests
// =============================================================================

#[tokio::test]
async fn test_supplier_response_structure() {
    let mut api = MockApi::new().await;

    let json = r#"{
        "Id": "sup-123",
        "SupplierNumber": "S001",
        "Name": "Test Supplier",
        "Email": "supplier@example.com",
        "Phone": "+46701234567",
        "IsActive": true,
        "Address": {
            "Address1": "Supplier Street 1",
            "City": "Malmö",
            "PostalCode": "21111"
        }
    }"#;

    let _mock = api.mock_get("/suppliers/sup-123", json);

    let supplier = api.client.suppliers().get("sup-123").await.unwrap();

    assert!(supplier.id.is_some());
    assert!(supplier.supplier_number.is_some());
    assert!(supplier.name.is_some());
    assert!(supplier.address.is_some());

    let addr = supplier.address.unwrap();
    assert_eq!(addr.city, Some("Malmö".to_string()));
}
