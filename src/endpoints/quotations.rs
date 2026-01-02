//! Quotations API endpoint.

use crate::client::Client;
use crate::error::Result;
use crate::types::{PaginatedResponse, PaginationParams, QueryParams, Quotation};

pub struct QuotationsEndpoint<'a> {
    client: &'a Client,
}

impl<'a> QuotationsEndpoint<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn list(
        &self,
        params: Option<PaginationParams>,
    ) -> Result<PaginatedResponse<Quotation>> {
        if let Some(params) = params {
            self.client.get_with_params("/quotations", &params).await
        } else {
            self.client.get("/quotations").await
        }
    }

    pub async fn get(&self, id: &str) -> Result<Quotation> {
        self.client.get(&format!("/quotations/{}", id)).await
    }

    pub async fn create(&self, quotation: &Quotation) -> Result<Quotation> {
        self.client.post("/quotations", quotation).await
    }

    pub async fn update(&self, id: &str, quotation: &Quotation) -> Result<Quotation> {
        self.client
            .put(&format!("/quotations/{}", id), quotation)
            .await
    }

    pub async fn delete(&self, id: &str) -> Result<()> {
        self.client.delete(&format!("/quotations/{}", id)).await
    }

    pub async fn search(
        &self,
        query: QueryParams,
        pagination: Option<PaginationParams>,
    ) -> Result<PaginatedResponse<Quotation>> {
        #[derive(serde::Serialize)]
        struct CombinedParams {
            #[serde(flatten)]
            query: QueryParams,
            #[serde(flatten)]
            pagination: Option<PaginationParams>,
        }
        self.client
            .get_with_params("/quotations", &CombinedParams { query, pagination })
            .await
    }
}
