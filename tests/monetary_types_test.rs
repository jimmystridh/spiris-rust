//! Tests for monetary value handling in the API client.
//!
//! Currently, the library uses f64 for monetary values.
//! These tests document the current behavior and potential precision issues.
//!
//! Note: When the `decimal` feature is enabled, these tests are skipped
//! and dedicated decimal tests in decimal_test.rs are run instead.

// These tests are specific to f64 behavior
#![cfg(not(feature = "decimal"))]

mod mock_server;

use mock_server::MockApi;
use spiris::{Article, Invoice, InvoiceRow};

// =============================================================================
// Current f64 Behavior Tests
// =============================================================================

#[test]
fn test_invoice_total_amount_f64() {
    let invoice = Invoice {
        total_amount: Some(1234.56),
        ..Default::default()
    };

    assert_eq!(invoice.total_amount, Some(1234.56));
}

#[test]
fn test_invoice_row_unit_price_f64() {
    let row = InvoiceRow {
        unit_price: Some(99.99),
        quantity: Some(2.0),
        ..Default::default()
    };

    assert_eq!(row.unit_price, Some(99.99));
    assert_eq!(row.quantity, Some(2.0));
}

#[test]
fn test_article_sales_price_f64() {
    let article = Article {
        sales_price: Some(49.95),
        ..Default::default()
    };

    assert_eq!(article.sales_price, Some(49.95));
}

// =============================================================================
// f64 Precision Issue Documentation
// =============================================================================

#[test]
fn test_f64_precision_issue_demonstration() {
    // This demonstrates why f64 is problematic for money
    let price: f64 = 0.1 + 0.2;

    // This is the classic floating-point issue
    assert_ne!(price, 0.3, "f64 has precision issues: 0.1 + 0.2 != 0.3");
    assert!((price - 0.3_f64).abs() < 1e-10, "But it's very close");
}

#[test]
fn test_f64_multiplication_precision() {
    let unit_price: f64 = 19.99;
    let quantity: f64 = 3.0;
    let expected_total: f64 = 59.97;

    let calculated = unit_price * quantity;

    // This may or may not work depending on the exact values
    // f64 can lose precision in calculations
    assert!(
        (calculated - expected_total).abs() < 0.0001,
        "Calculated: {}, Expected: {}",
        calculated,
        expected_total
    );
}

#[test]
fn test_f64_large_amount() {
    // Large amounts (millions) should work without issues
    let invoice = Invoice {
        total_amount: Some(1_000_000.00),
        ..Default::default()
    };

    assert_eq!(invoice.total_amount, Some(1_000_000.00));
}

#[test]
fn test_f64_small_amount() {
    // Small amounts (fractions of cents)
    let row = InvoiceRow {
        unit_price: Some(0.001),
        quantity: Some(1000.0),
        ..Default::default()
    };

    assert_eq!(row.unit_price, Some(0.001));
}

#[test]
fn test_f64_zero_amount() {
    let invoice = Invoice {
        total_amount: Some(0.0),
        ..Default::default()
    };

    assert_eq!(invoice.total_amount, Some(0.0));
}

#[test]
fn test_f64_negative_amount() {
    // Credits or refunds might be negative
    let row = InvoiceRow {
        total_amount: Some(-100.00),
        ..Default::default()
    };

    assert_eq!(row.total_amount, Some(-100.00));
}

// =============================================================================
// JSON Serialization Tests
// =============================================================================

#[test]
fn test_invoice_amount_serialization() {
    let invoice = Invoice {
        id: Some("inv-001".to_string()),
        total_amount: Some(1234.56),
        ..Default::default()
    };

    let json = serde_json::to_string(&invoice).unwrap();

    // Verify the amount is serialized as a number
    assert!(json.contains("1234.56") || json.contains("1234.5600"));
}

#[test]
fn test_invoice_amount_deserialization() {
    let json = r#"{"Id": "inv-001", "TotalAmount": 1234.56, "Rows": []}"#;

    let invoice: Invoice = serde_json::from_str(json).unwrap();

    assert_eq!(invoice.total_amount, Some(1234.56));
}

#[test]
fn test_amount_deserialization_from_integer() {
    // API might return integer for whole amounts
    let json = r#"{"Id": "inv-001", "TotalAmount": 1000, "Rows": []}"#;

    let invoice: Invoice = serde_json::from_str(json).unwrap();

    assert_eq!(invoice.total_amount, Some(1000.0));
}

#[test]
fn test_amount_deserialization_null() {
    let json = r#"{"Id": "inv-001", "TotalAmount": null, "Rows": []}"#;

    let invoice: Invoice = serde_json::from_str(json).unwrap();

    assert!(invoice.total_amount.is_none());
}

#[test]
fn test_amount_deserialization_missing() {
    let json = r#"{"Id": "inv-001", "Rows": []}"#;

    let invoice: Invoice = serde_json::from_str(json).unwrap();

    assert!(invoice.total_amount.is_none());
}

// =============================================================================
// Integration Tests with Mock Server
// =============================================================================

#[tokio::test]
async fn test_invoice_with_amount_from_api() {
    let mut api = MockApi::new().await;

    let invoice_json = r#"{
        "Id": "inv-001",
        "InvoiceNumber": "1001",
        "CustomerId": "cust-001",
        "TotalAmount": 999.99,
        "TotalVatAmount": 200.00,
        "TotalAmountIncludingVat": 1199.99,
        "Rows": []
    }"#;

    let _mock = api.mock_get("/customerinvoices/inv-001", invoice_json);

    let result = api.client.invoices().get("inv-001").await;
    assert!(result.is_ok());

    let invoice = result.unwrap();
    assert_eq!(invoice.total_amount, Some(999.99));
    assert_eq!(invoice.total_vat_amount, Some(200.00));
    assert_eq!(invoice.total_amount_including_vat, Some(1199.99));
}

#[tokio::test]
async fn test_article_with_prices_from_api() {
    let mut api = MockApi::new().await;

    let article_json = r#"{
        "Id": "art-001",
        "ArticleNumber": "ART-001",
        "Name": "Test Article",
        "SalesPrice": 149.99,
        "PurchasePrice": 75.50,
        "IsActive": true
    }"#;

    let _mock = api.mock_get("/articles/art-001", article_json);

    let result = api.client.articles().get("art-001").await;
    assert!(result.is_ok());

    let article = result.unwrap();
    assert_eq!(article.sales_price, Some(149.99));
    assert_eq!(article.purchase_price, Some(75.50));
}

#[tokio::test]
async fn test_invoice_list_with_amounts() {
    let mut api = MockApi::new().await;

    let response = r#"{
        "Data": [
            {"Id": "inv-001", "TotalAmount": 100.00, "Rows": []},
            {"Id": "inv-002", "TotalAmount": 200.50, "Rows": []},
            {"Id": "inv-003", "TotalAmount": 300.99, "Rows": []}
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

    let _mock = api.mock_get("/customerinvoices", response);

    let result = api.client.invoices().list(None).await;
    assert!(result.is_ok());

    let invoices = result.unwrap();
    assert_eq!(invoices.data.len(), 3);
    assert_eq!(invoices.data[0].total_amount, Some(100.00));
    assert_eq!(invoices.data[1].total_amount, Some(200.50));
    assert_eq!(invoices.data[2].total_amount, Some(300.99));
}

// =============================================================================
// Amount Calculation Tests (Current f64 Behavior)
// =============================================================================

#[test]
fn test_calculate_line_total() {
    let row = InvoiceRow {
        unit_price: Some(25.00),
        quantity: Some(4.0),
        ..Default::default()
    };

    if let (Some(price), Some(qty)) = (row.unit_price, row.quantity) {
        let calculated_total = price * qty;
        assert_eq!(calculated_total, 100.00);
    }
}

#[test]
fn test_calculate_with_discount() {
    let row = InvoiceRow {
        unit_price: Some(100.00),
        quantity: Some(1.0),
        discount_percentage: Some(10.0),
        ..Default::default()
    };

    if let (Some(price), Some(qty), Some(discount)) =
        (row.unit_price, row.quantity, row.discount_percentage)
    {
        let subtotal = price * qty;
        let discount_amount = subtotal * (discount / 100.0);
        let total = subtotal - discount_amount;

        assert_eq!(total, 90.00);
    }
}

#[test]
fn test_sum_invoice_rows() {
    let rows = vec![
        InvoiceRow {
            total_amount: Some(100.00),
            ..Default::default()
        },
        InvoiceRow {
            total_amount: Some(200.00),
            ..Default::default()
        },
        InvoiceRow {
            total_amount: Some(50.50),
            ..Default::default()
        },
    ];

    let total: f64 = rows
        .iter()
        .filter_map(|r| r.total_amount)
        .sum();

    assert!((total - 350.50).abs() < 0.0001);
}

// =============================================================================
// Currency and Locale Tests
// =============================================================================

#[test]
fn test_amount_with_many_decimal_places() {
    // API might return amounts with many decimal places
    let json = r#"{"Id": "inv-001", "TotalAmount": 123.456789, "Rows": []}"#;

    let invoice: Invoice = serde_json::from_str(json).unwrap();

    // f64 preserves the precision
    assert!(invoice.total_amount.is_some());
    let amount = invoice.total_amount.unwrap();
    assert!((amount - 123.456789).abs() < 0.0000001);
}

#[test]
fn test_amount_scientific_notation() {
    // JSON might use scientific notation for very small/large values
    let json = r#"{"Id": "inv-001", "TotalAmount": 1.5e6, "Rows": []}"#;

    let invoice: Invoice = serde_json::from_str(json).unwrap();

    assert_eq!(invoice.total_amount, Some(1_500_000.0));
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_very_large_amount() {
    let invoice = Invoice {
        total_amount: Some(999_999_999.99),
        ..Default::default()
    };

    let json = serde_json::to_string(&invoice).unwrap();
    let deserialized: Invoice = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.total_amount, invoice.total_amount);
}

#[test]
fn test_very_small_positive_amount() {
    let row = InvoiceRow {
        unit_price: Some(0.01), // 1 cent
        quantity: Some(1.0),
        ..Default::default()
    };

    assert_eq!(row.unit_price, Some(0.01));
}

// =============================================================================
// Future Decimal Type Tests (Documentation)
// =============================================================================
//
// Once rust_decimal is implemented, these tests should be enabled:
//
// #[test]
// fn test_decimal_precise_calculation() {
//     use rust_decimal::Decimal;
//     use std::str::FromStr;
//
//     let price = Decimal::from_str("19.99").unwrap();
//     let qty = Decimal::from_str("3").unwrap();
//     let total = price * qty;
//
//     assert_eq!(total, Decimal::from_str("59.97").unwrap());
//     // No floating-point precision issues!
// }
//
// #[test]
// fn test_decimal_serialization() {
//     let row = InvoiceRow {
//         unit_price: Some(Decimal::from_str("99.99").unwrap()),
//         quantity: Some(Decimal::from_str("2.5").unwrap()),
//         ..Default::default()
//     };
//
//     let json = serde_json::to_string(&row).unwrap();
//     // Should serialize as string or precise number
//     assert!(json.contains("99.99"));
// }
//
// #[test]
// fn test_decimal_addition_precise() {
//     use rust_decimal::Decimal;
//     use std::str::FromStr;
//
//     let a = Decimal::from_str("0.1").unwrap();
//     let b = Decimal::from_str("0.2").unwrap();
//     let sum = a + b;
//
//     // Unlike f64, this is exactly 0.3
//     assert_eq!(sum, Decimal::from_str("0.3").unwrap());
// }
