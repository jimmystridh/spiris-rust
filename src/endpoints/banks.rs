//! Banks API endpoint.

use crate::client::Client;
use crate::error::Result;
use crate::types::{Bank, ForeignPaymentCode, PaginatedResponse, PaginationParams};

pub struct BanksEndpoint<'a> {
    client: &'a Client,
}

impl<'a> BanksEndpoint<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn list(&self, params: Option<PaginationParams>) -> Result<PaginatedResponse<Bank>> {
        if let Some(params) = params {
            self.client.get_with_params("/banks", &params).await
        } else {
            self.client.get("/banks").await
        }
    }

    pub async fn list_foreign_payment_codes(
        &self,
    ) -> Result<PaginatedResponse<ForeignPaymentCode>> {
        self.client.get("/foreignpaymentcodes").await
    }
}
