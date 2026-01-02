//! Integration tests for the Accounts endpoint.

mod mock_server;

use mock_server::MockApi;
use spiris_bokforing::Account;

#[tokio::test]
async fn test_list_accounts() {
    let mut api = MockApi::new().await;

    let response_body = r#"{
        "Data": [
            {
                "AccountNumber": "1910",
                "Name": "Kassa",
                "IsActive": true,
                "VatCodeId": null
            },
            {
                "AccountNumber": "1920",
                "Name": "Bank",
                "IsActive": true,
                "VatCodeId": null
            },
            {
                "AccountNumber": "3000",
                "Name": "Försäljning",
                "IsActive": true,
                "VatCodeId": "vat-25"
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

    let mock = api.mock_get("/accounts", response_body);

    let result = api.client.accounts().list(None).await.unwrap();

    mock.assert();
    assert_eq!(result.data.len(), 3);
    assert_eq!(result.data[0].account_number, Some("1910".to_string()));
    assert_eq!(result.data[0].name, Some("Kassa".to_string()));
    assert_eq!(result.data[2].vat_code_id, Some("vat-25".to_string()));
}

#[tokio::test]
async fn test_get_account() {
    let mut api = MockApi::new().await;

    let response_body = r#"{
        "AccountNumber": "1930",
        "Name": "Företagskonto",
        "IsActive": true,
        "VatCodeId": null,
        "FiscalYearId": "fy-2024"
    }"#;

    let mock = api.mock_get("/accounts/fy-2024/1930", response_body);

    let result = api.client.accounts().get("fy-2024", "1930").await.unwrap();

    mock.assert();
    assert_eq!(result.account_number, Some("1930".to_string()));
    assert_eq!(result.name, Some("Företagskonto".to_string()));
}

#[tokio::test]
async fn test_create_account() {
    let mut api = MockApi::new().await;

    let response_body = r#"{
        "AccountNumber": "4000",
        "Name": "Inköp",
        "IsActive": true
    }"#;

    let mock = api.mock_post("/accounts", response_body);

    let new_account = Account {
        account_number: Some("4000".to_string()),
        name: Some("Inköp".to_string()),
        is_active: Some(true),
        ..Default::default()
    };

    let result = api.client.accounts().create(&new_account).await.unwrap();

    mock.assert();
    assert_eq!(result.account_number, Some("4000".to_string()));
    assert_eq!(result.name, Some("Inköp".to_string()));
}

#[tokio::test]
async fn test_update_account() {
    let mut api = MockApi::new().await;

    let response_body = r#"{
        "AccountNumber": "4000",
        "Name": "Inköp varor",
        "IsActive": true
    }"#;

    let mock = api.mock_put("/accounts/fy-2024/4000", response_body);

    let updated_account = Account {
        account_number: Some("4000".to_string()),
        name: Some("Inköp varor".to_string()),
        ..Default::default()
    };

    let result = api
        .client
        .accounts()
        .update("fy-2024", "4000", &updated_account)
        .await
        .unwrap();

    mock.assert();
    assert_eq!(result.name, Some("Inköp varor".to_string()));
}

#[tokio::test]
async fn test_get_account_balances() {
    let mut api = MockApi::new().await;

    let response_body = r#"{
        "Data": [
            {
                "AccountNumber": "1910",
                "Balance": 50000.0
            },
            {
                "AccountNumber": "1920",
                "Balance": 150000.0
            }
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

    let mock = api.mock_get("/accountbalances/2024-01-01", response_body);

    let result = api
        .client
        .accounts()
        .get_balances("2024-01-01")
        .await
        .unwrap();

    mock.assert();
    assert_eq!(result.data.len(), 2);
    assert_eq!(result.data[0].account_number, Some("1910".to_string()));
    assert_eq!(result.data[0].balance, Some(50000.0));
}

#[tokio::test]
async fn test_get_account_types() {
    let mut api = MockApi::new().await;

    let response_body = r#"{
        "Data": [
            {"Id": 1, "Name": "Tillgångar"},
            {"Id": 2, "Name": "Skulder"},
            {"Id": 3, "Name": "Intäkter"},
            {"Id": 4, "Name": "Kostnader"}
        ],
        "Meta": {
            "CurrentPage": 0,
            "PageSize": 50,
            "TotalPages": 1,
            "TotalCount": 4,
            "HasNextPage": false,
            "HasPreviousPage": false
        }
    }"#;

    let mock = api.mock_get("/accountTypes", response_body);

    let result = api.client.accounts().get_account_types().await.unwrap();

    mock.assert();
    assert_eq!(result.data.len(), 4);
}
