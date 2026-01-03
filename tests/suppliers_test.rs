//! Integration tests for the Suppliers endpoint.

mod mock_server;

use mock_server::MockApi;
use spiris::{PaginationParams, Supplier};

#[tokio::test]
async fn test_list_suppliers() {
    let mut api = MockApi::new().await;

    let response_body = r#"{
        "Data": [
            {
                "Id": "sup-001",
                "SupplierNumber": "S001",
                "Name": "Supplier AB",
                "Email": "info@supplier.se",
                "IsActive": true
            },
            {
                "Id": "sup-002",
                "SupplierNumber": "S002",
                "Name": "Parts Inc",
                "IsActive": true
            }
        ],
        "Meta": {
            "CurrentPage": 0,
            "PageSize": 50,
            "TotalPages": 1,
            "TotalCount": 2, "HasNextPage": false, "HasPreviousPage": false
        }
    }"#;

    let mock = api.mock_get("/suppliers", response_body);

    let result = api.client.suppliers().list(None).await.unwrap();

    mock.assert();
    assert_eq!(result.data.len(), 2);
    assert_eq!(result.data[0].id, Some("sup-001".to_string()));
    assert_eq!(result.data[0].name, Some("Supplier AB".to_string()));
}

#[tokio::test]
async fn test_get_supplier() {
    let mut api = MockApi::new().await;

    let response_body = r#"{
        "Id": "sup-123",
        "SupplierNumber": "S100",
        "Name": "Major Supplier AB",
        "CorporateIdentityNumber": "556123-4567",
        "Email": "contact@major.se",
        "Phone": "+46812345678",
        "BankAccountNumber": "1234-56789",
        "IsActive": true
    }"#;

    let mock = api.mock_get("/suppliers/sup-123", response_body);

    let result = api.client.suppliers().get("sup-123").await.unwrap();

    mock.assert();
    assert_eq!(result.id, Some("sup-123".to_string()));
    assert_eq!(result.name, Some("Major Supplier AB".to_string()));
    assert_eq!(
        result.corporate_identity_number,
        Some("556123-4567".to_string())
    );
}

#[tokio::test]
async fn test_create_supplier() {
    let mut api = MockApi::new().await;

    let response_body = r#"{
        "Id": "sup-new-001",
        "SupplierNumber": "S200",
        "Name": "New Supplier",
        "Email": "new@supplier.com",
        "IsActive": true
    }"#;

    let mock = api.mock_post("/suppliers", response_body);

    let new_supplier = Supplier {
        name: Some("New Supplier".to_string()),
        email: Some("new@supplier.com".to_string()),
        is_active: Some(true),
        ..Default::default()
    };

    let result = api.client.suppliers().create(&new_supplier).await.unwrap();

    mock.assert();
    assert_eq!(result.id, Some("sup-new-001".to_string()));
    assert_eq!(result.supplier_number, Some("S200".to_string()));
}

#[tokio::test]
async fn test_update_supplier() {
    let mut api = MockApi::new().await;

    let response_body = r#"{
        "Id": "sup-123",
        "SupplierNumber": "S100",
        "Name": "Updated Supplier Name",
        "IsActive": true
    }"#;

    let mock = api.mock_put("/suppliers/sup-123", response_body);

    let updated_supplier = Supplier {
        id: Some("sup-123".to_string()),
        name: Some("Updated Supplier Name".to_string()),
        ..Default::default()
    };

    let result = api
        .client
        .suppliers()
        .update("sup-123", &updated_supplier)
        .await
        .unwrap();

    mock.assert();
    assert_eq!(result.name, Some("Updated Supplier Name".to_string()));
}

#[tokio::test]
async fn test_delete_supplier() {
    let mut api = MockApi::new().await;

    let mock = api.mock_delete("/suppliers/sup-123");

    let result = api.client.suppliers().delete("sup-123").await;

    mock.assert();
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_list_suppliers_with_pagination() {
    let mut api = MockApi::new().await;

    let response_body = r#"{
        "Data": [
            {"Id": "sup-50", "Name": "Supplier 50"}
        ],
        "Meta": {
            "CurrentPage": 5,
            "PageSize": 10,
            "TotalPages": 10,
            "TotalCount": 100, "HasNextPage": false, "HasPreviousPage": false
        }
    }"#;

    let mock = api.mock_get_with_query(
        "/suppliers",
        vec![("page", "5"), ("pagesize", "10")],
        response_body,
    );

    let params = PaginationParams::new().page(5).pagesize(10);
    let result = api.client.suppliers().list(Some(params)).await.unwrap();

    mock.assert();
    assert_eq!(result.meta.current_page, 5);
    assert_eq!(result.meta.total_count, 100);
}
