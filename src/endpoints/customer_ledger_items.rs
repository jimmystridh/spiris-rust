//! Customer ledger items API endpoint.

use crate::client::Client;
use crate::error::Result;
use crate::types::{CustomerLedgerItem, PaginatedResponse, PaginationParams, QueryParams};

/// Customer ledger items endpoint for managing payment records.
pub struct CustomerLedgerItemsEndpoint<'a> {
    client: &'a Client,
}

impl<'a> CustomerLedgerItemsEndpoint<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// List all customer ledger items with optional pagination.
    pub async fn list(
        &self,
        params: Option<PaginationParams>,
    ) -> Result<PaginatedResponse<CustomerLedgerItem>> {
        if let Some(params) = params {
            self.client
                .get_with_params("/customerledgeritems", &params)
                .await
        } else {
            self.client.get("/customerledgeritems").await
        }
    }

    /// Get a specific customer ledger item by ID.
    pub async fn get(&self, id: &str) -> Result<CustomerLedgerItem> {
        let path = format!("/customerledgeritems/{}", id);
        self.client.get(&path).await
    }

    /// Create a new customer ledger item.
    pub async fn create(&self, item: &CustomerLedgerItem) -> Result<CustomerLedgerItem> {
        self.client.post("/customerledgeritems", item).await
    }

    /// Search customer ledger items with custom query parameters.
    pub async fn search(
        &self,
        query: QueryParams,
        pagination: Option<PaginationParams>,
    ) -> Result<PaginatedResponse<CustomerLedgerItem>> {
        #[derive(serde::Serialize)]
        struct CombinedParams {
            #[serde(flatten)]
            query: QueryParams,
            #[serde(flatten)]
            pagination: Option<PaginationParams>,
        }

        let params = CombinedParams { query, pagination };
        self.client
            .get_with_params("/customerledgeritems", &params)
            .await
    }
}
