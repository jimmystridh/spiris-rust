//! Supplier invoice drafts API endpoint.

use crate::client::Client;
use crate::error::Result;
use crate::types::{
    PaginatedResponse, PaginationParams, QueryParams, SupplierInvoice, SupplierInvoiceDraft,
};

pub struct SupplierInvoiceDraftsEndpoint<'a> {
    client: &'a Client,
}

impl<'a> SupplierInvoiceDraftsEndpoint<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn list(
        &self,
        params: Option<PaginationParams>,
    ) -> Result<PaginatedResponse<SupplierInvoiceDraft>> {
        if let Some(params) = params {
            self.client
                .get_with_params("/supplierinvoicedrafts", &params)
                .await
        } else {
            self.client.get("/supplierinvoicedrafts").await
        }
    }

    pub async fn get(&self, id: &str) -> Result<SupplierInvoiceDraft> {
        self.client
            .get(&format!("/supplierinvoicedrafts/{}", id))
            .await
    }

    pub async fn create(&self, draft: &SupplierInvoiceDraft) -> Result<SupplierInvoiceDraft> {
        self.client.post("/supplierinvoicedrafts", draft).await
    }

    pub async fn update(
        &self,
        id: &str,
        draft: &SupplierInvoiceDraft,
    ) -> Result<SupplierInvoiceDraft> {
        self.client
            .put(&format!("/supplierinvoicedrafts/{}", id), draft)
            .await
    }

    pub async fn delete(&self, id: &str) -> Result<()> {
        self.client
            .delete(&format!("/supplierinvoicedrafts/{}", id))
            .await
    }

    pub async fn convert(&self, id: &str) -> Result<SupplierInvoice> {
        self.client
            .post(&format!("/supplierinvoicedrafts/{}/convert", id), &())
            .await
    }

    pub async fn search(
        &self,
        query: QueryParams,
        pagination: Option<PaginationParams>,
    ) -> Result<PaginatedResponse<SupplierInvoiceDraft>> {
        #[derive(serde::Serialize)]
        struct CombinedParams {
            #[serde(flatten)]
            query: QueryParams,
            #[serde(flatten)]
            pagination: Option<PaginationParams>,
        }
        self.client
            .get_with_params(
                "/supplierinvoicedrafts",
                &CombinedParams { query, pagination },
            )
            .await
    }
}
