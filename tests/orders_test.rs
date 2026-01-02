//! Integration tests for the Orders endpoint.

mod mock_server;

use mock_server::MockApi;
use spiris_bokforing::{Order, OrderRow};

#[tokio::test]
async fn test_list_orders() {
    let mut api = MockApi::new().await;

    let response_body = r#"{
        "Data": [
            {
                "Id": "ord-001",
                "OrderNumber": "1001",
                "CustomerId": "cust-001",
                "TotalAmount": 5000.0,
                "CurrencyCode": "SEK",
                "IsInvoiced": false
            },
            {
                "Id": "ord-002",
                "OrderNumber": "1002",
                "CustomerId": "cust-002",
                "TotalAmount": 10000.0,
                "CurrencyCode": "SEK",
                "IsInvoiced": true
            }
        ],
        "Meta": {
            "CurrentPage": 0,
            "PageSize": 50,
            "TotalPages": 1,
            "TotalCount": 2, "HasNextPage": false, "HasPreviousPage": false
        }
    }"#;

    let mock = api.mock_get("/orders", response_body);

    let result = api.client.orders().list(None).await.unwrap();

    mock.assert();
    assert_eq!(result.data.len(), 2);
    assert_eq!(result.data[0].id, Some("ord-001".to_string()));
    assert_eq!(result.data[0].order_number, Some("1001".to_string()));
}

#[tokio::test]
async fn test_get_order() {
    let mut api = MockApi::new().await;

    let response_body = r#"{
        "Id": "ord-123",
        "OrderNumber": "2001",
        "CustomerId": "cust-456",
        "TotalAmount": 15000.0,
        "CurrencyCode": "SEK",
        "IsInvoiced": false,
        "Rows": [
            {
                "Id": "row-001",
                "ArticleId": "art-001",
                "Text": "Product A",
                "UnitPrice": 1500.0,
                "Quantity": 10.0
            }
        ]
    }"#;

    let mock = api.mock_get("/orders/ord-123", response_body);

    let result = api.client.orders().get("ord-123").await.unwrap();

    mock.assert();
    assert_eq!(result.id, Some("ord-123".to_string()));
    assert_eq!(result.order_number, Some("2001".to_string()));
    assert_eq!(result.rows.len(), 1);
}

#[tokio::test]
async fn test_create_order() {
    let mut api = MockApi::new().await;

    let response_body = r#"{
        "Id": "ord-new-001",
        "OrderNumber": "3001",
        "CustomerId": "cust-001",
        "TotalAmount": 2000.0,
        "CurrencyCode": "SEK",
        "IsInvoiced": false
    }"#;

    let mock = api.mock_post("/orders", response_body);

    let new_order = Order {
        customer_id: Some("cust-001".to_string()),
        currency_code: Some("SEK".to_string()),
        rows: vec![OrderRow {
            text: Some("Product".to_string()),
            unit_price: Some(200.0),
            quantity: Some(10.0),
            ..Default::default()
        }],
        ..Default::default()
    };

    let result = api.client.orders().create(&new_order).await.unwrap();

    mock.assert();
    assert_eq!(result.id, Some("ord-new-001".to_string()));
    assert_eq!(result.order_number, Some("3001".to_string()));
}

#[tokio::test]
async fn test_update_order() {
    let mut api = MockApi::new().await;

    let response_body = r#"{
        "Id": "ord-123",
        "OrderNumber": "2001",
        "TotalAmount": 20000.0
    }"#;

    let mock = api.mock_put("/orders/ord-123", response_body);

    let updated_order = Order {
        id: Some("ord-123".to_string()),
        ..Default::default()
    };

    let result = api
        .client
        .orders()
        .update("ord-123", &updated_order)
        .await
        .unwrap();

    mock.assert();
    assert_eq!(result.total_amount, Some(20000.0));
}

#[tokio::test]
async fn test_delete_order() {
    let mut api = MockApi::new().await;

    let mock = api.mock_delete("/orders/ord-123");

    let result = api.client.orders().delete("ord-123").await;

    mock.assert();
    assert!(result.is_ok());
}
