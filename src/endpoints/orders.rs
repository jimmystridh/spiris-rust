//! Orders API endpoint.

use crate::client::Client;
use crate::error::Result;
use crate::types::{Order, PaginatedResponse, PaginationParams, QueryParams};

pub struct OrdersEndpoint<'a> {
    client: &'a Client,
}

impl<'a> OrdersEndpoint<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn list(&self, params: Option<PaginationParams>) -> Result<PaginatedResponse<Order>> {
        if let Some(params) = params {
            self.client.get_with_params("/orders", &params).await
        } else {
            self.client.get("/orders").await
        }
    }

    pub async fn get(&self, id: &str) -> Result<Order> {
        self.client.get(&format!("/orders/{}", id)).await
    }

    pub async fn create(&self, order: &Order) -> Result<Order> {
        self.client.post("/orders", order).await
    }

    pub async fn update(&self, id: &str, order: &Order) -> Result<Order> {
        self.client.put(&format!("/orders/{}", id), order).await
    }

    pub async fn delete(&self, id: &str) -> Result<()> {
        self.client.delete(&format!("/orders/{}", id)).await
    }

    pub async fn search(
        &self,
        query: QueryParams,
        pagination: Option<PaginationParams>,
    ) -> Result<PaginatedResponse<Order>> {
        #[derive(serde::Serialize)]
        struct CombinedParams {
            #[serde(flatten)]
            query: QueryParams,
            #[serde(flatten)]
            pagination: Option<PaginationParams>,
        }
        self.client
            .get_with_params("/orders", &CombinedParams { query, pagination })
            .await
    }
}
