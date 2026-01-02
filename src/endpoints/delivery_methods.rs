//! Delivery methods API endpoint.

use crate::client::Client;
use crate::error::Result;
use crate::types::{DeliveryMethod, PaginatedResponse, PaginationParams};

pub struct DeliveryMethodsEndpoint<'a> {
    client: &'a Client,
}

impl<'a> DeliveryMethodsEndpoint<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn list(
        &self,
        params: Option<PaginationParams>,
    ) -> Result<PaginatedResponse<DeliveryMethod>> {
        if let Some(params) = params {
            self.client
                .get_with_params("/deliverymethods", &params)
                .await
        } else {
            self.client.get("/deliverymethods").await
        }
    }

    pub async fn get(&self, id: &str) -> Result<DeliveryMethod> {
        self.client.get(&format!("/deliverymethods/{}", id)).await
    }
}
