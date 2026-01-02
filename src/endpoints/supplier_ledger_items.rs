//! Supplier ledger items API endpoint.

use crate::client::Client;
use crate::error::Result;
use crate::types::{PaginatedResponse, PaginationParams, QueryParams, SupplierLedgerItem};

pub struct SupplierLedgerItemsEndpoint<'a> {
    client: &'a Client,
}

impl<'a> SupplierLedgerItemsEndpoint<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn list(
        &self,
        params: Option<PaginationParams>,
    ) -> Result<PaginatedResponse<SupplierLedgerItem>> {
        if let Some(params) = params {
            self.client
                .get_with_params("/supplierledgeritems", &params)
                .await
        } else {
            self.client.get("/supplierledgeritems").await
        }
    }

    pub async fn get(&self, id: &str) -> Result<SupplierLedgerItem> {
        self.client
            .get(&format!("/supplierledgeritems/{}", id))
            .await
    }

    pub async fn create(&self, item: &SupplierLedgerItem) -> Result<SupplierLedgerItem> {
        self.client.post("/supplierledgeritems", item).await
    }

    pub async fn search(
        &self,
        query: QueryParams,
        pagination: Option<PaginationParams>,
    ) -> Result<PaginatedResponse<SupplierLedgerItem>> {
        #[derive(serde::Serialize)]
        struct CombinedParams {
            #[serde(flatten)]
            query: QueryParams,
            #[serde(flatten)]
            pagination: Option<PaginationParams>,
        }
        self.client
            .get_with_params(
                "/supplierledgeritems",
                &CombinedParams { query, pagination },
            )
            .await
    }
}
