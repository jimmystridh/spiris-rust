//! Currencies API endpoint.

use crate::client::Client;
use crate::error::Result;
use crate::types::{Currency, PaginatedResponse, PaginationParams};

pub struct CurrenciesEndpoint<'a> {
    client: &'a Client,
}

impl<'a> CurrenciesEndpoint<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn list(
        &self,
        params: Option<PaginationParams>,
    ) -> Result<PaginatedResponse<Currency>> {
        if let Some(params) = params {
            self.client.get_with_params("/currencies", &params).await
        } else {
            self.client.get("/currencies").await
        }
    }
}
