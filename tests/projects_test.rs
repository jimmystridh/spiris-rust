//! Integration tests for the Projects endpoint.

mod mock_server;

use mock_server::MockApi;
use spiris::Project;

#[tokio::test]
async fn test_list_projects() {
    let mut api = MockApi::new().await;

    let response_body = r#"{
        "Data": [
            {
                "Id": "proj-001",
                "ProjectNumber": "P001",
                "Name": "Website Redesign",
                "IsCompleted": false
            },
            {
                "Id": "proj-002",
                "ProjectNumber": "P002",
                "Name": "Mobile App",
                "IsCompleted": false
            }
        ],
        "Meta": {
            "CurrentPage": 0,
            "PageSize": 50,
            "TotalPages": 1,
            "TotalCount": 2, "HasNextPage": false, "HasPreviousPage": false
        }
    }"#;

    let mock = api.mock_get("/projects", response_body);

    let result = api.client.projects().list(None).await.unwrap();

    mock.assert();
    assert_eq!(result.data.len(), 2);
    assert_eq!(result.data[0].id, Some("proj-001".to_string()));
    assert_eq!(result.data[0].name, Some("Website Redesign".to_string()));
}

#[tokio::test]
async fn test_get_project() {
    let mut api = MockApi::new().await;

    let response_body = r#"{
        "Id": "proj-123",
        "ProjectNumber": "P100",
        "Name": "Enterprise Integration",
        "StartDate": "2024-01-01T00:00:00Z",
        "EndDate": "2024-12-31T00:00:00Z",
        "CustomerId": "cust-456",
        "Notes": "Major client project",
        "IsCompleted": false
    }"#;

    let mock = api.mock_get("/projects/proj-123", response_body);

    let result = api.client.projects().get("proj-123").await.unwrap();

    mock.assert();
    assert_eq!(result.id, Some("proj-123".to_string()));
    assert_eq!(result.name, Some("Enterprise Integration".to_string()));
    assert_eq!(result.customer_id, Some("cust-456".to_string()));
}

#[tokio::test]
async fn test_create_project() {
    let mut api = MockApi::new().await;

    let response_body = r#"{
        "Id": "proj-new-001",
        "ProjectNumber": "P200",
        "Name": "New Project",
        "IsCompleted": false
    }"#;

    let mock = api.mock_post("/projects", response_body);

    let new_project = Project {
        name: Some("New Project".to_string()),
        ..Default::default()
    };

    let result = api.client.projects().create(&new_project).await.unwrap();

    mock.assert();
    assert_eq!(result.id, Some("proj-new-001".to_string()));
    assert_eq!(result.project_number, Some("P200".to_string()));
}

#[tokio::test]
async fn test_update_project() {
    let mut api = MockApi::new().await;

    let response_body = r#"{
        "Id": "proj-123",
        "ProjectNumber": "P100",
        "Name": "Updated Project Name",
        "IsCompleted": true
    }"#;

    let mock = api.mock_put("/projects/proj-123", response_body);

    let updated_project = Project {
        id: Some("proj-123".to_string()),
        name: Some("Updated Project Name".to_string()),
        is_completed: Some(true),
        ..Default::default()
    };

    let result = api
        .client
        .projects()
        .update("proj-123", &updated_project)
        .await
        .unwrap();

    mock.assert();
    assert_eq!(result.name, Some("Updated Project Name".to_string()));
    assert_eq!(result.is_completed, Some(true));
}

#[tokio::test]
async fn test_delete_project() {
    let mut api = MockApi::new().await;

    let mock = api.mock_delete("/projects/proj-123");

    let result = api.client.projects().delete("proj-123").await;

    mock.assert();
    assert!(result.is_ok());
}
