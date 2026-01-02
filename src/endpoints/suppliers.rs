//! Suppliers API endpoint.

use crate::client::Client;
use crate::error::Result;
use crate::types::{PaginatedResponse, PaginationParams, QueryParams, Supplier};

/// Suppliers endpoint for managing supplier records.
pub struct SuppliersEndpoint<'a> {
    client: &'a Client,
}

impl<'a> SuppliersEndpoint<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// List all suppliers with optional pagination.
    pub async fn list(
        &self,
        params: Option<PaginationParams>,
    ) -> Result<PaginatedResponse<Supplier>> {
        if let Some(params) = params {
            self.client.get_with_params("/suppliers", &params).await
        } else {
            self.client.get("/suppliers").await
        }
    }

    /// Get a specific supplier by ID.
    pub async fn get(&self, id: &str) -> Result<Supplier> {
        let path = format!("/suppliers/{}", id);
        self.client.get(&path).await
    }

    /// Create a new supplier.
    pub async fn create(&self, supplier: &Supplier) -> Result<Supplier> {
        self.client.post("/suppliers", supplier).await
    }

    /// Update an existing supplier.
    pub async fn update(&self, id: &str, supplier: &Supplier) -> Result<Supplier> {
        let path = format!("/suppliers/{}", id);
        self.client.put(&path, supplier).await
    }

    /// Delete a supplier.
    pub async fn delete(&self, id: &str) -> Result<()> {
        let path = format!("/suppliers/{}", id);
        self.client.delete(&path).await
    }

    /// Search suppliers with custom query parameters.
    pub async fn search(
        &self,
        query: QueryParams,
        pagination: Option<PaginationParams>,
    ) -> Result<PaginatedResponse<Supplier>> {
        #[derive(serde::Serialize)]
        struct CombinedParams {
            #[serde(flatten)]
            query: QueryParams,
            #[serde(flatten)]
            pagination: Option<PaginationParams>,
        }

        let params = CombinedParams { query, pagination };
        self.client.get_with_params("/suppliers", &params).await
    }
}
