//! VAT codes API endpoint.

use crate::client::Client;
use crate::error::Result;
use crate::types::{PaginatedResponse, PaginationParams, VatCode};

/// VAT codes endpoint for managing tax rates.
pub struct VatCodesEndpoint<'a> {
    client: &'a Client,
}

impl<'a> VatCodesEndpoint<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// List all VAT codes with optional pagination.
    pub async fn list(
        &self,
        params: Option<PaginationParams>,
    ) -> Result<PaginatedResponse<VatCode>> {
        if let Some(params) = params {
            self.client.get_with_params("/vatcodes", &params).await
        } else {
            self.client.get("/vatcodes").await
        }
    }

    /// Get a specific VAT code by ID.
    pub async fn get(&self, id: &str) -> Result<VatCode> {
        let path = format!("/vatcodes/{}", id);
        self.client.get(&path).await
    }
}
