// Skip these tests when decimal feature is enabled (uses f64 literals)
#![cfg(not(feature = "decimal"))]
//! Integration tests for the Articles endpoint.

mod mock_server;

use mock_server::MockApi;
use spiris::{Article, PaginationParams, QueryParams};

#[tokio::test]
async fn test_list_articles() {
    let mut api = MockApi::new().await;

    let response_body = r#"{
        "Data": [
            {
                "Id": "art-001",
                "ArticleNumber": "PROD-001",
                "Name": "Widget A",
                "SalesPrice": 199.99,
                "PurchasePrice": 99.99,
                "IsActive": true
            },
            {
                "Id": "art-002",
                "ArticleNumber": "PROD-002",
                "Name": "Widget B",
                "SalesPrice": 299.99,
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

    let mock = api.mock_get("/articles", response_body);

    let result = api.client.articles().list(None).await.unwrap();

    mock.assert();
    assert_eq!(result.data.len(), 2);
    assert_eq!(result.data[0].id, Some("art-001".to_string()));
    assert_eq!(result.data[0].name, Some("Widget A".to_string()));
    assert_eq!(result.data[0].sales_price, Some(199.99));
}

#[tokio::test]
async fn test_get_article() {
    let mut api = MockApi::new().await;

    let response_body = r#"{
        "Id": "art-123",
        "ArticleNumber": "SERV-001",
        "Name": "Consulting Hour",
        "Unit": "hour",
        "SalesPrice": 1500.0,
        "PurchasePrice": 0.0,
        "IsActive": true,
        "VatRateId": "vat-25"
    }"#;

    let mock = api.mock_get("/articles/art-123", response_body);

    let result = api.client.articles().get("art-123").await.unwrap();

    mock.assert();
    assert_eq!(result.id, Some("art-123".to_string()));
    assert_eq!(result.name, Some("Consulting Hour".to_string()));
    assert_eq!(result.unit, Some("hour".to_string()));
    assert_eq!(result.sales_price, Some(1500.0));
}

#[tokio::test]
async fn test_create_article() {
    let mut api = MockApi::new().await;

    let response_body = r#"{
        "Id": "art-new-001",
        "ArticleNumber": "NEW-001",
        "Name": "New Product",
        "SalesPrice": 500.0,
        "IsActive": true
    }"#;

    let mock = api.mock_post("/articles", response_body);

    let new_article = Article {
        name: Some("New Product".to_string()),
        sales_price: Some(500.0),
        is_active: Some(true),
        ..Default::default()
    };

    let result = api.client.articles().create(&new_article).await.unwrap();

    mock.assert();
    assert_eq!(result.id, Some("art-new-001".to_string()));
    assert_eq!(result.article_number, Some("NEW-001".to_string()));
}

#[tokio::test]
async fn test_update_article() {
    let mut api = MockApi::new().await;

    let response_body = r#"{
        "Id": "art-123",
        "ArticleNumber": "PROD-001",
        "Name": "Updated Product Name",
        "SalesPrice": 599.99,
        "IsActive": true
    }"#;

    let mock = api.mock_put("/articles/art-123", response_body);

    let updated_article = Article {
        id: Some("art-123".to_string()),
        name: Some("Updated Product Name".to_string()),
        sales_price: Some(599.99),
        ..Default::default()
    };

    let result = api
        .client
        .articles()
        .update("art-123", &updated_article)
        .await
        .unwrap();

    mock.assert();
    assert_eq!(result.name, Some("Updated Product Name".to_string()));
    assert_eq!(result.sales_price, Some(599.99));
}

#[tokio::test]
async fn test_delete_article() {
    let mut api = MockApi::new().await;

    let mock = api.mock_delete("/articles/art-123");

    let result = api.client.articles().delete("art-123").await;

    mock.assert();
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_search_articles() {
    let mut api = MockApi::new().await;

    let response_body = r#"{
        "Data": [
            {"Id": "art-active-1", "Name": "Active Article", "IsActive": true}
        ],
        "Meta": {
            "CurrentPage": 0,
            "PageSize": 50,
            "TotalPages": 1,
            "TotalCount": 2, "HasNextPage": false, "HasPreviousPage": false
        }
    }"#;

    let mock = api.mock_get_with_query(
        "/articles",
        vec![("filter", "IsActive eq true")],
        response_body,
    );

    let query = QueryParams::new().filter("IsActive eq true");
    let result = api.client.articles().search(query, None).await.unwrap();

    mock.assert();
    assert_eq!(result.data.len(), 1);
}

#[tokio::test]
async fn test_list_articles_with_pagination() {
    let mut api = MockApi::new().await;

    let response_body = r#"{
        "Data": [
            {"Id": "art-101", "Name": "Article 101", "SalesPrice": 100.0}
        ],
        "Meta": {
            "CurrentPage": 2,
            "PageSize": 20,
            "TotalPages": 10,
            "TotalCount": 200, "HasNextPage": false, "HasPreviousPage": false
        }
    }"#;

    let mock = api.mock_get_with_query(
        "/articles",
        vec![("page", "2"), ("pagesize", "20")],
        response_body,
    );

    let params = PaginationParams::new().page(2).pagesize(20);
    let result = api.client.articles().list(Some(params)).await.unwrap();

    mock.assert();
    assert_eq!(result.meta.current_page, 2);
    assert_eq!(result.meta.total_count, 200);
}
