//! Supplier invoices API endpoint.

use crate::client::Client;
use crate::error::Result;
use crate::types::{
    InvoicePayment, PaginatedResponse, PaginationParams, QueryParams, SupplierInvoice,
};

/// Supplier invoices endpoint for managing accounts payable.
pub struct SupplierInvoicesEndpoint<'a> {
    client: &'a Client,
}

impl<'a> SupplierInvoicesEndpoint<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// List all supplier invoices with optional pagination.
    pub async fn list(
        &self,
        params: Option<PaginationParams>,
    ) -> Result<PaginatedResponse<SupplierInvoice>> {
        if let Some(params) = params {
            self.client
                .get_with_params("/supplierinvoices", &params)
                .await
        } else {
            self.client.get("/supplierinvoices").await
        }
    }

    /// Get a specific supplier invoice by ID.
    pub async fn get(&self, id: &str) -> Result<SupplierInvoice> {
        let path = format!("/supplierinvoices/{}", id);
        self.client.get(&path).await
    }

    /// Create a new supplier invoice.
    pub async fn create(&self, invoice: &SupplierInvoice) -> Result<SupplierInvoice> {
        self.client.post("/supplierinvoices", invoice).await
    }

    /// Update an existing supplier invoice.
    pub async fn update(&self, id: &str, invoice: &SupplierInvoice) -> Result<SupplierInvoice> {
        let path = format!("/supplierinvoices/{}", id);
        self.client.put(&path, invoice).await
    }

    /// Delete a supplier invoice.
    pub async fn delete(&self, id: &str) -> Result<()> {
        let path = format!("/supplierinvoices/{}", id);
        self.client.delete(&path).await
    }

    /// Register a payment for a supplier invoice.
    pub async fn register_payment(&self, invoice_id: &str, payment: &InvoicePayment) -> Result<()> {
        let path = format!("/supplierinvoices/{}/payments", invoice_id);
        self.client.post::<(), _>(&path, payment).await?;
        Ok(())
    }

    /// Search supplier invoices with custom query parameters.
    pub async fn search(
        &self,
        query: QueryParams,
        pagination: Option<PaginationParams>,
    ) -> Result<PaginatedResponse<SupplierInvoice>> {
        #[derive(serde::Serialize)]
        struct CombinedParams {
            #[serde(flatten)]
            query: QueryParams,
            #[serde(flatten)]
            pagination: Option<PaginationParams>,
        }

        let params = CombinedParams { query, pagination };
        self.client
            .get_with_params("/supplierinvoices", &params)
            .await
    }
}
