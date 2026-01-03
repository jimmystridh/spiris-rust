//! Tests for the decimal feature.
//!
//! These tests verify that the decimal feature works correctly when enabled.
//! Run with: `cargo test --features decimal decimal_test`

#![cfg(feature = "decimal")]

use spiris::{money, Invoice, InvoiceRow, Money};
use std::str::FromStr;

#[test]
fn test_money_type_is_decimal() {
    // Verify Money is Decimal when feature is enabled
    let value: Money = rust_decimal::Decimal::from_str("100.50").unwrap();
    assert_eq!(value.to_string(), "100.50");
}

#[test]
fn test_money_macro_creates_decimal() {
    let value: Money = money!(100.50);
    assert_eq!(value.to_string(), "100.50");
}

#[test]
fn test_money_precision() {
    // This is the classic floating-point issue that Decimal solves
    let a: Money = money!(0.1);
    let b: Money = money!(0.2);
    let sum = a + b;
    let expected: Money = money!(0.3);
    assert_eq!(
        sum, expected,
        "Decimal should handle 0.1 + 0.2 = 0.3 correctly"
    );
}

#[test]
fn test_invoice_row_with_decimal() {
    let row = InvoiceRow {
        unit_price: Some(money!(1234.56)),
        quantity: Some(money!(10.0)),
        ..Default::default()
    };

    assert_eq!(row.unit_price, Some(money!(1234.56)));
    assert_eq!(row.quantity, Some(money!(10.0)));
}

#[test]
fn test_invoice_with_decimal() {
    let invoice = Invoice {
        total_amount: Some(money!(12345.67)),
        total_vat_amount: Some(money!(2469.13)),
        ..Default::default()
    };

    assert_eq!(invoice.total_amount, Some(money!(12345.67)));
    assert_eq!(invoice.total_vat_amount, Some(money!(2469.13)));
}

#[test]
fn test_decimal_serialization() {
    let row = InvoiceRow {
        unit_price: Some(money!(100.00)),
        quantity: Some(money!(5.0)),
        ..Default::default()
    };

    // Serialize to JSON
    let json = serde_json::to_string(&row).unwrap();

    // The JSON should contain the numeric values
    assert!(json.contains("100"));
    assert!(json.contains("5"));
}

#[test]
fn test_decimal_deserialization() {
    let json = r#"{
        "UnitPrice": 1234.56,
        "Quantity": 10
    }"#;

    let row: InvoiceRow = serde_json::from_str(json).unwrap();

    assert_eq!(row.unit_price, Some(money!(1234.56)));
    assert_eq!(row.quantity, Some(money!(10.0)));
}

#[test]
fn test_decimal_from_integer_json() {
    // API sometimes returns integers for whole numbers
    let json = r#"{
        "UnitPrice": 100,
        "Quantity": 5
    }"#;

    let row: InvoiceRow = serde_json::from_str(json).unwrap();

    assert_eq!(row.unit_price, Some(money!(100.0)));
    assert_eq!(row.quantity, Some(money!(5.0)));
}

#[test]
fn test_decimal_null_handling() {
    let json = r#"{
        "UnitPrice": null,
        "Quantity": null
    }"#;

    let row: InvoiceRow = serde_json::from_str(json).unwrap();

    assert!(row.unit_price.is_none());
    assert!(row.quantity.is_none());
}

#[test]
fn test_decimal_arithmetic() {
    let price: Money = money!(99.99);
    let quantity: Money = money!(3.0);
    let subtotal = price * quantity;

    // 99.99 * 3 = 299.97
    assert_eq!(subtotal, money!(299.97));
}
