//! Terms of payment API endpoint.

use crate::client::Client;
use crate::error::Result;
use crate::types::{PaginatedResponse, PaginationParams, TermsOfPayment};

pub struct TermsOfPaymentEndpoint<'a> {
    client: &'a Client,
}

impl<'a> TermsOfPaymentEndpoint<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn list(
        &self,
        params: Option<PaginationParams>,
    ) -> Result<PaginatedResponse<TermsOfPayment>> {
        if let Some(params) = params {
            self.client
                .get_with_params("/termsofpayments", &params)
                .await
        } else {
            self.client.get("/termsofpayments").await
        }
    }

    pub async fn get(&self, id: &str) -> Result<TermsOfPayment> {
        self.client.get(&format!("/termsofpayments/{}", id)).await
    }

    pub async fn create(&self, terms: &TermsOfPayment) -> Result<TermsOfPayment> {
        self.client.post("/termsofpayments", terms).await
    }

    pub async fn update(&self, id: &str, terms: &TermsOfPayment) -> Result<TermsOfPayment> {
        self.client
            .put(&format!("/termsofpayments/{}", id), terms)
            .await
    }

    pub async fn delete(&self, id: &str) -> Result<()> {
        self.client
            .delete(&format!("/termsofpayments/{}", id))
            .await
    }
}
