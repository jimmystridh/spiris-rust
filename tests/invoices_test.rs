// Skip these tests when decimal feature is enabled (uses f64 literals)
#![cfg(not(feature = "decimal"))]
//! Integration tests for the Invoices endpoint.

mod mock_server;

use mock_server::MockApi;
use spiris::{Invoice, InvoicePayment, InvoiceRow, PaginationParams};

#[tokio::test]
async fn test_list_invoices() {
    let mut api = MockApi::new().await;

    let response_body = r#"{
        "Data": [
            {
                "Id": "inv-001",
                "InvoiceNumber": "1001",
                "CustomerId": "cust-001",
                "TotalAmount": 10000.0,
                "TotalVatAmount": 2500.0,
                "CurrencyCode": "SEK",
                "IsSent": true,
                "Rows": []
            },
            {
                "Id": "inv-002",
                "InvoiceNumber": "1002",
                "CustomerId": "cust-002",
                "TotalAmount": 5000.0,
                "TotalVatAmount": 1250.0,
                "CurrencyCode": "SEK",
                "IsSent": false,
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

    let mock = api.mock_get("/customerinvoices", response_body);

    let result = api.client.invoices().list(None).await.unwrap();

    mock.assert();
    assert_eq!(result.data.len(), 2);
    assert_eq!(result.data[0].id, Some("inv-001".to_string()));
    assert_eq!(result.data[0].invoice_number, Some("1001".to_string()));
    assert_eq!(result.data[0].total_amount, Some(10000.0));
}

#[tokio::test]
async fn test_get_invoice() {
    let mut api = MockApi::new().await;

    let response_body = r#"{
        "Id": "inv-123",
        "InvoiceNumber": "2001",
        "CustomerId": "cust-456",
        "TotalAmount": 15000.0,
        "TotalVatAmount": 3750.0,
        "TotalAmountIncludingVat": 18750.0,
        "CurrencyCode": "SEK",
        "IsSent": true,
        "Rows": [
            {
                "Id": "row-001",
                "ArticleId": "art-001",
                "Text": "Consulting services",
                "UnitPrice": 1500.0,
                "Quantity": 10.0,
                "TotalAmount": 15000.0
            }
        ]
    }"#;

    let mock = api.mock_get("/customerinvoices/inv-123", response_body);

    let result = api.client.invoices().get("inv-123").await.unwrap();

    mock.assert();
    assert_eq!(result.id, Some("inv-123".to_string()));
    assert_eq!(result.invoice_number, Some("2001".to_string()));
    assert_eq!(result.total_amount_including_vat, Some(18750.0));
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0].text, Some("Consulting services".to_string()));
}

#[tokio::test]
async fn test_create_invoice() {
    let mut api = MockApi::new().await;

    let response_body = r#"{
        "Id": "inv-new-001",
        "InvoiceNumber": "3001",
        "CustomerId": "cust-001",
        "TotalAmount": 2000.0,
        "TotalVatAmount": 500.0,
        "TotalAmountIncludingVat": 2500.0,
        "CurrencyCode": "SEK",
        "IsSent": false,
        "Rows": [
            {
                "Id": "row-new-001",
                "Text": "Product A",
                "UnitPrice": 200.0,
                "Quantity": 10.0,
                "TotalAmount": 2000.0
            }
        ]
    }"#;

    let mock = api.mock_post("/customerinvoices", response_body);

    let new_invoice = Invoice {
        customer_id: Some("cust-001".to_string()),
        currency_code: Some("SEK".to_string()),
        rows: vec![InvoiceRow {
            text: Some("Product A".to_string()),
            unit_price: Some(200.0),
            quantity: Some(10.0),
            ..Default::default()
        }],
        ..Default::default()
    };

    let result = api.client.invoices().create(&new_invoice).await.unwrap();

    mock.assert();
    assert_eq!(result.id, Some("inv-new-001".to_string()));
    assert_eq!(result.invoice_number, Some("3001".to_string()));
    assert_eq!(result.total_amount, Some(2000.0));
}

#[tokio::test]
async fn test_update_invoice() {
    let mut api = MockApi::new().await;

    let response_body = r#"{
        "Id": "inv-123",
        "InvoiceNumber": "1001",
        "CustomerId": "cust-001",
        "Remarks": "Updated remarks",
        "TotalAmount": 10000.0,
        "CurrencyCode": "SEK",
        "Rows": []
    }"#;

    let mock = api.mock_put("/customerinvoices/inv-123", response_body);

    let updated_invoice = Invoice {
        id: Some("inv-123".to_string()),
        remarks: Some("Updated remarks".to_string()),
        ..Default::default()
    };

    let result = api
        .client
        .invoices()
        .update("inv-123", &updated_invoice)
        .await
        .unwrap();

    mock.assert();
    assert_eq!(result.remarks, Some("Updated remarks".to_string()));
}

#[tokio::test]
async fn test_delete_invoice() {
    let mut api = MockApi::new().await;

    let mock = api.mock_delete("/customerinvoices/inv-123");

    let result = api.client.invoices().delete("inv-123").await;

    mock.assert();
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_register_invoice_payment() {
    let mut api = MockApi::new().await;

    let mock = api
        .server
        .mock("POST", "/customerinvoices/inv-123/payments")
        .match_header("Authorization", "Bearer test_token")
        .match_header("Content-Type", "application/json")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body("null")
        .create();

    let payment = InvoicePayment {
        amount: Some(10000.0),
        payment_date: None,
        bank_account_id: Some("bank-001".to_string()),
        ..Default::default()
    };

    let result = api
        .client
        .invoices()
        .register_payment("inv-123", &payment)
        .await;

    mock.assert();
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_get_invoice_pdf() {
    let mut api = MockApi::new().await;

    let pdf_content = b"%PDF-1.4 fake pdf content";
    let mock = api.mock_get_bytes("/customerinvoices/inv-123/pdf", pdf_content);

    let result = api.client.invoices().get_pdf("inv-123").await.unwrap();

    mock.assert();
    assert_eq!(result, pdf_content.to_vec());
}

#[tokio::test]
async fn test_send_einvoice() {
    let mut api = MockApi::new().await;

    let mock = api
        .server
        .mock("POST", "/customerinvoices/inv-123/einvoice")
        .match_header("Authorization", "Bearer test_token")
        .match_header("Content-Type", "application/json")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("null")
        .create();

    let result = api.client.invoices().send_einvoice("inv-123").await;

    mock.assert();
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_list_invoices_with_pagination() {
    let mut api = MockApi::new().await;

    let response_body = r#"{
        "Data": [
            {"Id": "inv-page2", "InvoiceNumber": "2001", "TotalAmount": 5000.0, "Rows": []}
        ],
        "Meta": {
            "CurrentPage": 1,
            "PageSize": 25,
            "TotalPages": 4,
            "TotalCount": 100, "HasNextPage": false, "HasPreviousPage": false
        }
    }"#;

    let mock = api.mock_get_with_query(
        "/customerinvoices",
        vec![("page", "1"), ("pagesize", "25")],
        response_body,
    );

    let params = PaginationParams::new().page(1).pagesize(25);
    let result = api.client.invoices().list(Some(params)).await.unwrap();

    mock.assert();
    assert_eq!(result.data.len(), 1);
    assert_eq!(result.meta.current_page, 1);
    assert_eq!(result.meta.page_size, 25);
    assert_eq!(result.meta.total_count, 100);
}
