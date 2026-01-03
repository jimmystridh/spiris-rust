//! Cost centers API endpoint.

use crate::types::{CostCenter, CostCenterItem, PaginatedResponse, PaginationParams};

crate::define_endpoint! {
    /// Cost centers endpoint for managing cost center tracking.
    CostCentersEndpoint, "/costcenters", CostCenter,
    caps: [list, update],
    extra: {
        /// List all cost center items with optional pagination.
        pub async fn list_items(
            &self,
            params: Option<PaginationParams>,
        ) -> crate::error::Result<PaginatedResponse<CostCenterItem>> {
            if let Some(params) = params {
                self.client
                    .get_with_params("/costcenteritems", &params)
                    .await
            } else {
                self.client.get("/costcenteritems").await
            }
        }

        /// Get a specific cost center item by ID.
        pub async fn get_item(&self, id: &str) -> crate::error::Result<CostCenterItem> {
            self.client.get(&format!("/costcenteritems/{}", id)).await
        }

        /// Create a new cost center item.
        pub async fn create_item(&self, item: &CostCenterItem) -> crate::error::Result<CostCenterItem> {
            self.client.post("/costcenteritems", item).await
        }

        /// Update an existing cost center item.
        pub async fn update_item(&self, id: &str, item: &CostCenterItem) -> crate::error::Result<CostCenterItem> {
            self.client
                .put(&format!("/costcenteritems/{}", id), item)
                .await
        }
    }
}
