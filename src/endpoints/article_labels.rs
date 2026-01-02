//! Article labels API endpoint.

use crate::client::Client;
use crate::error::Result;
use crate::types::{ArticleLabel, PaginatedResponse, PaginationParams};

pub struct ArticleLabelsEndpoint<'a> {
    client: &'a Client,
}

impl<'a> ArticleLabelsEndpoint<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn list(
        &self,
        params: Option<PaginationParams>,
    ) -> Result<PaginatedResponse<ArticleLabel>> {
        if let Some(params) = params {
            self.client.get_with_params("/articlelabels", &params).await
        } else {
            self.client.get("/articlelabels").await
        }
    }

    pub async fn get(&self, id: &str) -> Result<ArticleLabel> {
        self.client.get(&format!("/articlelabels/{}", id)).await
    }

    pub async fn create(&self, label: &ArticleLabel) -> Result<ArticleLabel> {
        self.client.post("/articlelabels", label).await
    }

    pub async fn update(&self, id: &str, label: &ArticleLabel) -> Result<ArticleLabel> {
        self.client
            .put(&format!("/articlelabels/{}", id), label)
            .await
    }

    pub async fn delete(&self, id: &str) -> Result<()> {
        self.client.delete(&format!("/articlelabels/{}", id)).await
    }
}
