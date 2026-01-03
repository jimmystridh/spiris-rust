// Skip these tests when decimal feature is enabled (uses f64 literals)
#![cfg(not(feature = "decimal"))]
//! Integration tests for the Vouchers endpoint.

mod mock_server;

use mock_server::MockApi;
use spiris::{PaginationParams, Voucher, VoucherRow};

#[tokio::test]
async fn test_list_vouchers() {
    let mut api = MockApi::new().await;

    let response_body = r#"{
        "Data": [
            {
                "Id": "vouch-001",
                "VoucherNumber": "1",
                "VoucherDate": "2024-01-15T00:00:00Z",
                "VoucherText": "January sales",
                "Rows": []
            },
            {
                "Id": "vouch-002",
                "VoucherNumber": "2",
                "VoucherDate": "2024-01-20T00:00:00Z",
                "VoucherText": "Office supplies",
                "Rows": []
            }
        ],
        "Meta": {
            "CurrentPage": 0,
            "PageSize": 50,
            "TotalPages": 1,
            "TotalCount": 2, "HasNextPage": false, "HasPreviousPage": false
        }
    }"#;

    let mock = api.mock_get("/vouchers", response_body);

    let result = api.client.vouchers().list(None).await.unwrap();

    mock.assert();
    assert_eq!(result.data.len(), 2);
    assert_eq!(result.data[0].id, Some("vouch-001".to_string()));
    assert_eq!(result.data[0].voucher_number, Some("1".to_string()));
}

#[tokio::test]
async fn test_get_voucher() {
    let mut api = MockApi::new().await;

    let response_body = r#"{
        "Id": "vouch-123",
        "VoucherNumber": "42",
        "VoucherDate": "2024-03-15T00:00:00Z",
        "VoucherText": "Customer payment",
        "Rows": [
            {
                "AccountNumber": "1920",
                "DebitAmount": 10000.0,
                "CreditAmount": 0.0,
                "TransactionText": "Bank deposit"
            },
            {
                "AccountNumber": "1510",
                "DebitAmount": 0.0,
                "CreditAmount": 10000.0,
                "TransactionText": "Customer receivable"
            }
        ]
    }"#;

    let mock = api.mock_get("/vouchers/vouch-123", response_body);

    let result = api.client.vouchers().get("vouch-123").await.unwrap();

    mock.assert();
    assert_eq!(result.id, Some("vouch-123".to_string()));
    assert_eq!(result.voucher_number, Some("42".to_string()));
    assert_eq!(result.rows.len(), 2);
    assert_eq!(result.rows[0].debit_amount, Some(10000.0));
    assert_eq!(result.rows[1].credit_amount, Some(10000.0));
}

#[tokio::test]
async fn test_create_voucher() {
    let mut api = MockApi::new().await;

    let response_body = r#"{
        "Id": "vouch-new-001",
        "VoucherNumber": "100",
        "VoucherDate": "2024-06-01T00:00:00Z",
        "VoucherText": "Test voucher",
        "Rows": [
            {"AccountNumber": "1920", "DebitAmount": 5000.0, "CreditAmount": 0.0},
            {"AccountNumber": "3000", "DebitAmount": 0.0, "CreditAmount": 5000.0}
        ]
    }"#;

    let mock = api.mock_post("/vouchers", response_body);

    let new_voucher = Voucher {
        voucher_text: Some("Test voucher".to_string()),
        rows: vec![
            VoucherRow {
                account_number: Some("1920".to_string()),
                debit_amount: Some(5000.0),
                credit_amount: Some(0.0),
                ..Default::default()
            },
            VoucherRow {
                account_number: Some("3000".to_string()),
                debit_amount: Some(0.0),
                credit_amount: Some(5000.0),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let result = api.client.vouchers().create(&new_voucher).await.unwrap();

    mock.assert();
    assert_eq!(result.id, Some("vouch-new-001".to_string()));
    assert_eq!(result.voucher_number, Some("100".to_string()));
}

#[tokio::test]
async fn test_update_voucher() {
    let mut api = MockApi::new().await;

    let response_body = r#"{
        "Id": "vouch-123",
        "VoucherNumber": "42",
        "VoucherText": "Updated voucher text",
        "Rows": []
    }"#;

    let mock = api.mock_put("/vouchers/vouch-123", response_body);

    let updated_voucher = Voucher {
        id: Some("vouch-123".to_string()),
        voucher_text: Some("Updated voucher text".to_string()),
        ..Default::default()
    };

    let result = api
        .client
        .vouchers()
        .update("vouch-123", &updated_voucher)
        .await
        .unwrap();

    mock.assert();
    assert_eq!(
        result.voucher_text,
        Some("Updated voucher text".to_string())
    );
}

#[tokio::test]
async fn test_delete_voucher() {
    let mut api = MockApi::new().await;

    let mock = api.mock_delete("/vouchers/vouch-123");

    let result = api.client.vouchers().delete("vouch-123").await;

    mock.assert();
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_list_vouchers_with_pagination() {
    let mut api = MockApi::new().await;

    let response_body = r#"{
        "Data": [
            {"Id": "vouch-101", "VoucherNumber": "101", "Rows": []}
        ],
        "Meta": {
            "CurrentPage": 10,
            "PageSize": 10,
            "TotalPages": 50,
            "TotalCount": 500, "HasNextPage": false, "HasPreviousPage": false
        }
    }"#;

    let mock = api.mock_get_with_query(
        "/vouchers",
        vec![("page", "10"), ("pagesize", "10")],
        response_body,
    );

    let params = PaginationParams::new().page(10).pagesize(10);
    let result = api.client.vouchers().list(Some(params)).await.unwrap();

    mock.assert();
    assert_eq!(result.meta.current_page, 10);
    assert_eq!(result.meta.total_count, 500);
}
