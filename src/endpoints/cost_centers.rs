//! Cost centers API endpoint.

use crate::client::Client;
use crate::error::Result;
use crate::types::{CostCenter, CostCenterItem, PaginatedResponse, PaginationParams};

pub struct CostCentersEndpoint<'a> {
    client: &'a Client,
}

impl<'a> CostCentersEndpoint<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn list(
        &self,
        params: Option<PaginationParams>,
    ) -> Result<PaginatedResponse<CostCenter>> {
        if let Some(params) = params {
            self.client.get_with_params("/costcenters", &params).await
        } else {
            self.client.get("/costcenters").await
        }
    }

    pub async fn update(&self, id: &str, cost_center: &CostCenter) -> Result<CostCenter> {
        self.client
            .put(&format!("/costcenters/{}", id), cost_center)
            .await
    }

    pub async fn list_items(
        &self,
        params: Option<PaginationParams>,
    ) -> Result<PaginatedResponse<CostCenterItem>> {
        if let Some(params) = params {
            self.client
                .get_with_params("/costcenteritems", &params)
                .await
        } else {
            self.client.get("/costcenteritems").await
        }
    }

    pub async fn get_item(&self, id: &str) -> Result<CostCenterItem> {
        self.client.get(&format!("/costcenteritems/{}", id)).await
    }

    pub async fn create_item(&self, item: &CostCenterItem) -> Result<CostCenterItem> {
        self.client.post("/costcenteritems", item).await
    }

    pub async fn update_item(&self, id: &str, item: &CostCenterItem) -> Result<CostCenterItem> {
        self.client
            .put(&format!("/costcenteritems/{}", id), item)
            .await
    }
}
