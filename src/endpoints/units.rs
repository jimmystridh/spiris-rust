//! Units API endpoint.

use crate::client::Client;
use crate::error::Result;
use crate::types::{PaginatedResponse, PaginationParams, Unit};

pub struct UnitsEndpoint<'a> {
    client: &'a Client,
}

impl<'a> UnitsEndpoint<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn list(&self, params: Option<PaginationParams>) -> Result<PaginatedResponse<Unit>> {
        if let Some(params) = params {
            self.client.get_with_params("/units", &params).await
        } else {
            self.client.get("/units").await
        }
    }

    pub async fn get(&self, id: &str) -> Result<Unit> {
        self.client.get(&format!("/units/{}", id)).await
    }

    pub async fn create(&self, unit: &Unit) -> Result<Unit> {
        self.client.post("/units", unit).await
    }

    pub async fn update(&self, id: &str, unit: &Unit) -> Result<Unit> {
        self.client.put(&format!("/units/{}", id), unit).await
    }

    pub async fn delete(&self, id: &str) -> Result<()> {
        self.client.delete(&format!("/units/{}", id)).await
    }
}
