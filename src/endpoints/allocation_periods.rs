//! Allocation periods API endpoint.

use crate::client::Client;
use crate::error::Result;
use crate::types::{AllocationPeriod, PaginatedResponse, PaginationParams};

pub struct AllocationPeriodsEndpoint<'a> {
    client: &'a Client,
}

impl<'a> AllocationPeriodsEndpoint<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn list(
        &self,
        params: Option<PaginationParams>,
    ) -> Result<PaginatedResponse<AllocationPeriod>> {
        if let Some(params) = params {
            self.client
                .get_with_params("/allocationperiods", &params)
                .await
        } else {
            self.client.get("/allocationperiods").await
        }
    }

    pub async fn get(&self, id: &str) -> Result<AllocationPeriod> {
        self.client.get(&format!("/allocationperiods/{}", id)).await
    }

    pub async fn create(&self, period: &AllocationPeriod) -> Result<AllocationPeriod> {
        self.client.post("/allocationperiods", period).await
    }
}
