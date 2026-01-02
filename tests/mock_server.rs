//! Shared mock server utilities for integration tests.

use mockito::{Matcher, Mock, Server, ServerGuard};
use spiris_bokforing::{AccessToken, Client, ClientConfig};

/// Helper to create a standard paginated Meta section
#[allow(dead_code)]
pub fn meta_json(current_page: u32, page_size: u32, total_pages: u32, total_count: u32) -> String {
    format!(
        r#""Meta": {{
            "CurrentPage": {},
            "PageSize": {},
            "TotalPages": {},
            "TotalCount": {},
            "HasNextPage": {},
            "HasPreviousPage": {}
        }}"#,
        current_page,
        page_size,
        total_pages,
        total_count,
        current_page + 1 < total_pages,
        current_page > 0
    )
}

#[allow(dead_code)]
pub struct MockApi {
    pub server: ServerGuard,
    pub client: Client,
}

#[allow(dead_code)]
impl MockApi {
    pub async fn new() -> Self {
        let server = Server::new_async().await;
        let token = AccessToken::new("test_token".to_string(), 3600, None);
        let config = ClientConfig::new().base_url(server.url());
        let client = Client::with_config(token, config);
        Self { server, client }
    }

    pub fn mock_get(&mut self, path: &str, response_body: &str) -> Mock {
        self.server
            .mock("GET", path)
            .match_header("Authorization", "Bearer test_token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response_body)
            .create()
    }

    pub fn mock_get_with_query(
        &mut self,
        path: &str,
        query: Vec<(&str, &str)>,
        response_body: &str,
    ) -> Mock {
        let mut mock = self
            .server
            .mock("GET", path)
            .match_header("Authorization", "Bearer test_token");

        for (key, value) in query {
            mock = mock.match_query(Matcher::UrlEncoded(key.into(), value.into()));
        }

        mock.with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response_body)
            .create()
    }

    pub fn mock_post(&mut self, path: &str, response_body: &str) -> Mock {
        self.server
            .mock("POST", path)
            .match_header("Authorization", "Bearer test_token")
            .match_header("Content-Type", "application/json")
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(response_body)
            .create()
    }

    pub fn mock_post_with_body(
        &mut self,
        path: &str,
        request_body: &str,
        response_body: &str,
    ) -> Mock {
        self.server
            .mock("POST", path)
            .match_header("Authorization", "Bearer test_token")
            .match_header("Content-Type", "application/json")
            .match_body(Matcher::Json(serde_json::from_str(request_body).unwrap()))
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(response_body)
            .create()
    }

    pub fn mock_put(&mut self, path: &str, response_body: &str) -> Mock {
        self.server
            .mock("PUT", path)
            .match_header("Authorization", "Bearer test_token")
            .match_header("Content-Type", "application/json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response_body)
            .create()
    }

    pub fn mock_delete(&mut self, path: &str) -> Mock {
        self.server
            .mock("DELETE", path)
            .match_header("Authorization", "Bearer test_token")
            .with_status(204)
            .create()
    }

    pub fn mock_error(&mut self, method: &str, path: &str, status: u16, body: &str) -> Mock {
        self.server
            .mock(method, path)
            .with_status(status as usize)
            .with_header("content-type", "application/json")
            .with_body(body)
            .create()
    }

    pub fn mock_get_bytes(&mut self, path: &str, data: &[u8]) -> Mock {
        self.server
            .mock("GET", path)
            .match_header("Authorization", "Bearer test_token")
            .with_status(200)
            .with_header("content-type", "application/pdf")
            .with_body(data)
            .create()
    }
}
