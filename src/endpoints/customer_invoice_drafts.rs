//! Customer invoice drafts API endpoint.

use crate::client::Client;
use crate::error::Result;
use crate::types::{
    ConvertDraftOptions, CustomerInvoiceDraft, Invoice, PaginatedResponse, PaginationParams,
    QueryParams,
};

/// Customer invoice drafts endpoint for managing draft invoices.
pub struct CustomerInvoiceDraftsEndpoint<'a> {
    client: &'a Client,
}

impl<'a> CustomerInvoiceDraftsEndpoint<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// List all customer invoice drafts with optional pagination.
    pub async fn list(
        &self,
        params: Option<PaginationParams>,
    ) -> Result<PaginatedResponse<CustomerInvoiceDraft>> {
        if let Some(params) = params {
            self.client
                .get_with_params("/customerinvoicedrafts", &params)
                .await
        } else {
            self.client.get("/customerinvoicedrafts").await
        }
    }

    /// Get a specific customer invoice draft by ID.
    pub async fn get(&self, id: &str) -> Result<CustomerInvoiceDraft> {
        let path = format!("/customerinvoicedrafts/{}", id);
        self.client.get(&path).await
    }

    /// Create a new customer invoice draft.
    pub async fn create(&self, draft: &CustomerInvoiceDraft) -> Result<CustomerInvoiceDraft> {
        self.client.post("/customerinvoicedrafts", draft).await
    }

    /// Update an existing customer invoice draft.
    pub async fn update(
        &self,
        id: &str,
        draft: &CustomerInvoiceDraft,
    ) -> Result<CustomerInvoiceDraft> {
        let path = format!("/customerinvoicedrafts/{}", id);
        self.client.put(&path, draft).await
    }

    /// Delete a customer invoice draft.
    pub async fn delete(&self, id: &str) -> Result<()> {
        let path = format!("/customerinvoicedrafts/{}", id);
        self.client.delete(&path).await
    }

    /// Convert a draft to a finalized invoice.
    ///
    /// # Arguments
    ///
    /// * `id` - The draft ID to convert
    /// * `options` - Optional conversion options (send type, etc.)
    pub async fn convert(&self, id: &str, options: Option<ConvertDraftOptions>) -> Result<Invoice> {
        let path = format!("/customerinvoicedrafts/{}/convert", id);
        let body = options.unwrap_or_default();
        self.client.post(&path, &body).await
    }

    /// Search customer invoice drafts with custom query parameters.
    pub async fn search(
        &self,
        query: QueryParams,
        pagination: Option<PaginationParams>,
    ) -> Result<PaginatedResponse<CustomerInvoiceDraft>> {
        #[derive(serde::Serialize)]
        struct CombinedParams {
            #[serde(flatten)]
            query: QueryParams,
            #[serde(flatten)]
            pagination: Option<PaginationParams>,
        }

        let params = CombinedParams { query, pagination };
        self.client
            .get_with_params("/customerinvoicedrafts", &params)
            .await
    }
}
