//! Vouchers API endpoint.

use crate::client::Client;
use crate::error::Result;
use crate::types::{PaginatedResponse, PaginationParams, QueryParams, Voucher};

/// Vouchers endpoint for managing journal entries.
pub struct VouchersEndpoint<'a> {
    client: &'a Client,
}

impl<'a> VouchersEndpoint<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// List all vouchers with optional pagination.
    pub async fn list(
        &self,
        params: Option<PaginationParams>,
    ) -> Result<PaginatedResponse<Voucher>> {
        if let Some(params) = params {
            self.client.get_with_params("/vouchers", &params).await
        } else {
            self.client.get("/vouchers").await
        }
    }

    /// Get a specific voucher by ID.
    pub async fn get(&self, id: &str) -> Result<Voucher> {
        let path = format!("/vouchers/{}", id);
        self.client.get(&path).await
    }

    /// Create a new voucher.
    pub async fn create(&self, voucher: &Voucher) -> Result<Voucher> {
        self.client.post("/vouchers", voucher).await
    }

    /// Update an existing voucher.
    pub async fn update(&self, id: &str, voucher: &Voucher) -> Result<Voucher> {
        let path = format!("/vouchers/{}", id);
        self.client.put(&path, voucher).await
    }

    /// Delete a voucher.
    pub async fn delete(&self, id: &str) -> Result<()> {
        let path = format!("/vouchers/{}", id);
        self.client.delete(&path).await
    }

    /// Search vouchers with custom query parameters.
    pub async fn search(
        &self,
        query: QueryParams,
        pagination: Option<PaginationParams>,
    ) -> Result<PaginatedResponse<Voucher>> {
        #[derive(serde::Serialize)]
        struct CombinedParams {
            #[serde(flatten)]
            query: QueryParams,
            #[serde(flatten)]
            pagination: Option<PaginationParams>,
        }

        let params = CombinedParams { query, pagination };
        self.client.get_with_params("/vouchers", &params).await
    }
}
