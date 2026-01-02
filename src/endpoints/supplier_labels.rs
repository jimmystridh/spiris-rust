//! Supplier labels API endpoint.

use crate::client::Client;
use crate::error::Result;
use crate::types::{PaginatedResponse, PaginationParams, SupplierLabel};

pub struct SupplierLabelsEndpoint<'a> {
    client: &'a Client,
}

impl<'a> SupplierLabelsEndpoint<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn list(
        &self,
        params: Option<PaginationParams>,
    ) -> Result<PaginatedResponse<SupplierLabel>> {
        if let Some(params) = params {
            self.client
                .get_with_params("/supplierlabels", &params)
                .await
        } else {
            self.client.get("/supplierlabels").await
        }
    }

    pub async fn get(&self, id: &str) -> Result<SupplierLabel> {
        self.client.get(&format!("/supplierlabels/{}", id)).await
    }

    pub async fn create(&self, label: &SupplierLabel) -> Result<SupplierLabel> {
        self.client.post("/supplierlabels", label).await
    }

    pub async fn update(&self, id: &str, label: &SupplierLabel) -> Result<SupplierLabel> {
        self.client
            .put(&format!("/supplierlabels/{}", id), label)
            .await
    }

    pub async fn delete(&self, id: &str) -> Result<()> {
        self.client.delete(&format!("/supplierlabels/{}", id)).await
    }
}
