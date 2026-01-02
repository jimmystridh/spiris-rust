//! Integration tests for the Customers endpoint.

mod mock_server;

use mock_server::MockApi;
use spiris_bokforing::{Customer, PaginationParams, QueryParams};

#[tokio::test]
async fn test_list_customers() {
    let mut api = MockApi::new().await;

    let response_body = r#"{
        "Data": [
            {
                "Id": "cust-001",
                "CustomerNumber": "1001",
                "Name": "Acme Corp",
                "Email": "contact@acme.com",
                "IsActive": true
            },
            {
                "Id": "cust-002",
                "CustomerNumber": "1002",
                "Name": "Beta Inc",
                "Email": "info@beta.com",
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

    let mock = api.mock_get("/customers", response_body);

    let result = api.client.customers().list(None).await.unwrap();

    mock.assert();
    assert_eq!(result.data.len(), 2);
    assert_eq!(result.data[0].id, Some("cust-001".to_string()));
    assert_eq!(result.data[0].name, Some("Acme Corp".to_string()));
    assert_eq!(result.data[1].customer_number, Some("1002".to_string()));
    assert_eq!(result.meta.total_count, 2);
}

#[tokio::test]
async fn test_list_customers_with_pagination() {
    let mut api = MockApi::new().await;

    let response_body = r#"{
        "Data": [
            {"Id": "cust-101", "Name": "Page 2 Customer"}
        ],
        "Meta": {
            "CurrentPage": 1,
            "PageSize": 10,
            "TotalPages": 5,
            "TotalCount": 4, "HasNextPage": false, "HasPreviousPage": false
        }
    }"#;

    let mock = api.mock_get_with_query(
        "/customers",
        vec![("page", "1"), ("pagesize", "10")],
        response_body,
    );

    let params = PaginationParams::new().page(1).pagesize(10);
    let result = api.client.customers().list(Some(params)).await.unwrap();

    mock.assert();
    assert_eq!(result.data.len(), 1);
    assert_eq!(result.meta.current_page, 1);
    assert_eq!(result.meta.page_size, 10);
    assert_eq!(result.meta.total_pages, 5);
}

#[tokio::test]
async fn test_get_customer() {
    let mut api = MockApi::new().await;

    let response_body = r#"{
        "Id": "cust-123",
        "CustomerNumber": "2001",
        "Name": "Test Customer AB",
        "Email": "test@customer.se",
        "Phone": "+46701234567",
        "IsActive": true,
        "IsPrivatePerson": false,
        "PaymentTermsInDays": 30
    }"#;

    let mock = api.mock_get("/customers/cust-123", response_body);

    let result = api.client.customers().get("cust-123").await.unwrap();

    mock.assert();
    assert_eq!(result.id, Some("cust-123".to_string()));
    assert_eq!(result.name, Some("Test Customer AB".to_string()));
    assert_eq!(result.email, Some("test@customer.se".to_string()));
    assert_eq!(result.payment_terms_in_days, Some(30));
}

#[tokio::test]
async fn test_create_customer() {
    let mut api = MockApi::new().await;

    let response_body = r#"{
        "Id": "cust-new-001",
        "CustomerNumber": "3001",
        "Name": "New Customer",
        "Email": "new@customer.com",
        "IsActive": true
    }"#;

    let mock = api.mock_post("/customers", response_body);

    let new_customer = Customer {
        name: Some("New Customer".to_string()),
        email: Some("new@customer.com".to_string()),
        is_active: Some(true),
        ..Default::default()
    };

    let result = api.client.customers().create(&new_customer).await.unwrap();

    mock.assert();
    assert_eq!(result.id, Some("cust-new-001".to_string()));
    assert_eq!(result.customer_number, Some("3001".to_string()));
    assert_eq!(result.name, Some("New Customer".to_string()));
}

#[tokio::test]
async fn test_update_customer() {
    let mut api = MockApi::new().await;

    let response_body = r#"{
        "Id": "cust-123",
        "CustomerNumber": "1001",
        "Name": "Updated Customer Name",
        "Email": "updated@email.com",
        "IsActive": true
    }"#;

    let mock = api.mock_put("/customers/cust-123", response_body);

    let updated_customer = Customer {
        id: Some("cust-123".to_string()),
        name: Some("Updated Customer Name".to_string()),
        email: Some("updated@email.com".to_string()),
        ..Default::default()
    };

    let result = api
        .client
        .customers()
        .update("cust-123", &updated_customer)
        .await
        .unwrap();

    mock.assert();
    assert_eq!(result.name, Some("Updated Customer Name".to_string()));
}

#[tokio::test]
async fn test_delete_customer() {
    let mut api = MockApi::new().await;

    let mock = api.mock_delete("/customers/cust-123");

    let result = api.client.customers().delete("cust-123").await;

    mock.assert();
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_search_customers() {
    let mut api = MockApi::new().await;

    let response_body = r#"{
        "Data": [
            {"Id": "cust-active-1", "Name": "Active Customer", "IsActive": true}
        ],
        "Meta": {
            "CurrentPage": 0,
            "PageSize": 50,
            "TotalPages": 1,
            "TotalCount": 2, "HasNextPage": false, "HasPreviousPage": false
        }
    }"#;

    let mock = api.mock_get_with_query(
        "/customers",
        vec![("filter", "IsActive eq true")],
        response_body,
    );

    let query = QueryParams::new().filter("IsActive eq true");
    let result = api.client.customers().search(query, None).await.unwrap();

    mock.assert();
    assert_eq!(result.data.len(), 1);
    assert_eq!(result.data[0].is_active, Some(true));
}

#[tokio::test]
async fn test_search_customers_with_select() {
    let mut api = MockApi::new().await;

    let response_body = r#"{
        "Data": [
            {"Id": "cust-1", "Name": "Customer 1"},
            {"Id": "cust-2", "Name": "Customer 2"}
        ],
        "Meta": {
            "CurrentPage": 0,
            "PageSize": 50,
            "TotalPages": 1,
            "TotalCount": 2, "HasNextPage": false, "HasPreviousPage": false
        }
    }"#;

    let mock = api.mock_get_with_query("/customers", vec![("select", "Id,Name")], response_body);

    let query = QueryParams::new().select("Id,Name");
    let result = api.client.customers().search(query, None).await.unwrap();

    mock.assert();
    assert_eq!(result.data.len(), 2);
}
