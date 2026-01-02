//! Users API endpoint.

use crate::client::Client;
use crate::error::Result;
use crate::types::{PaginatedResponse, PaginationParams, User};

pub struct UsersEndpoint<'a> {
    client: &'a Client,
}

impl<'a> UsersEndpoint<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn list(&self, params: Option<PaginationParams>) -> Result<PaginatedResponse<User>> {
        if let Some(params) = params {
            self.client.get_with_params("/users", &params).await
        } else {
            self.client.get("/users").await
        }
    }

    pub async fn get(&self, id: &str) -> Result<User> {
        self.client.get(&format!("/users/{}", id)).await
    }
}
