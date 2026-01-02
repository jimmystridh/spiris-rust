//! Countries API endpoint.

use crate::client::Client;
use crate::error::Result;
use crate::types::{Country, PaginatedResponse, PaginationParams};

pub struct CountriesEndpoint<'a> {
    client: &'a Client,
}

impl<'a> CountriesEndpoint<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn list(
        &self,
        params: Option<PaginationParams>,
    ) -> Result<PaginatedResponse<Country>> {
        if let Some(params) = params {
            self.client.get_with_params("/countries", &params).await
        } else {
            self.client.get("/countries").await
        }
    }

    pub async fn get(&self, code: &str) -> Result<Country> {
        self.client.get(&format!("/countries/{}", code)).await
    }
}
