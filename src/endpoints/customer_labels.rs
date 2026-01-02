//! Customer labels API endpoint.

use crate::client::Client;
use crate::error::Result;
use crate::types::{CustomerLabel, PaginatedResponse, PaginationParams};

/// Customer labels endpoint for managing customer categorization.
pub struct CustomerLabelsEndpoint<'a> {
    client: &'a Client,
}

impl<'a> CustomerLabelsEndpoint<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// List all customer labels with optional pagination.
    pub async fn list(
        &self,
        params: Option<PaginationParams>,
    ) -> Result<PaginatedResponse<CustomerLabel>> {
        if let Some(params) = params {
            self.client
                .get_with_params("/customerlabels", &params)
                .await
        } else {
            self.client.get("/customerlabels").await
        }
    }

    /// Get a specific customer label by ID.
    pub async fn get(&self, id: &str) -> Result<CustomerLabel> {
        let path = format!("/customerlabels/{}", id);
        self.client.get(&path).await
    }

    /// Create a new customer label.
    pub async fn create(&self, label: &CustomerLabel) -> Result<CustomerLabel> {
        self.client.post("/customerlabels", label).await
    }

    /// Update an existing customer label.
    pub async fn update(&self, id: &str, label: &CustomerLabel) -> Result<CustomerLabel> {
        let path = format!("/customerlabels/{}", id);
        self.client.put(&path, label).await
    }

    /// Delete a customer label.
    pub async fn delete(&self, id: &str) -> Result<()> {
        let path = format!("/customerlabels/{}", id);
        self.client.delete(&path).await
    }
}
