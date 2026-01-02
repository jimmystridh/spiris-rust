//! Fiscal years API endpoint.

use crate::client::Client;
use crate::error::Result;
use crate::types::{FiscalYear, PaginatedResponse, PaginationParams};
use serde::{Deserialize, Serialize};

/// Opening balance entry.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct OpeningBalance {
    /// Account number.
    pub account_number: String,
    /// Opening balance amount.
    pub amount: f64,
}

/// Fiscal years endpoint for managing accounting periods.
pub struct FiscalYearsEndpoint<'a> {
    client: &'a Client,
}

impl<'a> FiscalYearsEndpoint<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// List all fiscal years with optional pagination.
    pub async fn list(
        &self,
        params: Option<PaginationParams>,
    ) -> Result<PaginatedResponse<FiscalYear>> {
        if let Some(params) = params {
            self.client.get_with_params("/fiscalyears", &params).await
        } else {
            self.client.get("/fiscalyears").await
        }
    }

    /// Get a specific fiscal year by ID.
    pub async fn get(&self, id: &str) -> Result<FiscalYear> {
        let path = format!("/fiscalyears/{}", id);
        self.client.get(&path).await
    }

    /// Create a new fiscal year.
    pub async fn create(&self, fiscal_year: &FiscalYear) -> Result<FiscalYear> {
        self.client.post("/fiscalyears", fiscal_year).await
    }

    /// Get opening balances for the first fiscal year.
    pub async fn get_opening_balances(&self) -> Result<Vec<OpeningBalance>> {
        self.client.get("/fiscalyears/openingbalances").await
    }

    /// Update opening balances for the first fiscal year.
    ///
    /// Note: This replaces all opening balances, not adds to them.
    pub async fn update_opening_balances(
        &self,
        balances: &[OpeningBalance],
    ) -> Result<Vec<OpeningBalance>> {
        self.client
            .put("/fiscalyears/openingbalances", &balances)
            .await
    }
}
