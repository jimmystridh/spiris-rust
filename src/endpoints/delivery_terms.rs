//! Delivery terms API endpoint.

use crate::client::Client;
use crate::error::Result;
use crate::types::{DeliveryTerm, PaginatedResponse, PaginationParams};

pub struct DeliveryTermsEndpoint<'a> {
    client: &'a Client,
}

impl<'a> DeliveryTermsEndpoint<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn list(
        &self,
        params: Option<PaginationParams>,
    ) -> Result<PaginatedResponse<DeliveryTerm>> {
        if let Some(params) = params {
            self.client.get_with_params("/deliveryterms", &params).await
        } else {
            self.client.get("/deliveryterms").await
        }
    }

    pub async fn get(&self, id: &str) -> Result<DeliveryTerm> {
        self.client.get(&format!("/deliveryterms/{}", id)).await
    }
}
