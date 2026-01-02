//! Article account codings API endpoint.

use crate::client::Client;
use crate::error::Result;
use crate::types::{ArticleAccountCoding, PaginatedResponse, PaginationParams};

pub struct ArticleAccountCodingsEndpoint<'a> {
    client: &'a Client,
}

impl<'a> ArticleAccountCodingsEndpoint<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn list(
        &self,
        params: Option<PaginationParams>,
    ) -> Result<PaginatedResponse<ArticleAccountCoding>> {
        if let Some(params) = params {
            self.client
                .get_with_params("/articleaccountcodings", &params)
                .await
        } else {
            self.client.get("/articleaccountcodings").await
        }
    }

    pub async fn get(&self, id: &str) -> Result<ArticleAccountCoding> {
        self.client
            .get(&format!("/articleaccountcodings/{}", id))
            .await
    }
}
