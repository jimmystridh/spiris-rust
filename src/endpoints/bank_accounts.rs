//! Bank accounts API endpoint.

use crate::client::Client;
use crate::error::Result;
use crate::types::{BankAccount, PaginatedResponse, PaginationParams};

/// Bank accounts endpoint for managing payment accounts.
pub struct BankAccountsEndpoint<'a> {
    client: &'a Client,
}

impl<'a> BankAccountsEndpoint<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// List all bank accounts with optional pagination.
    pub async fn list(
        &self,
        params: Option<PaginationParams>,
    ) -> Result<PaginatedResponse<BankAccount>> {
        if let Some(params) = params {
            self.client.get_with_params("/bankaccounts", &params).await
        } else {
            self.client.get("/bankaccounts").await
        }
    }

    /// Get a specific bank account by ID.
    pub async fn get(&self, id: &str) -> Result<BankAccount> {
        let path = format!("/bankaccounts/{}", id);
        self.client.get(&path).await
    }

    /// Create a new bank account.
    pub async fn create(&self, bank_account: &BankAccount) -> Result<BankAccount> {
        self.client.post("/bankaccounts", bank_account).await
    }

    /// Update an existing bank account.
    pub async fn update(&self, id: &str, bank_account: &BankAccount) -> Result<BankAccount> {
        let path = format!("/bankaccounts/{}", id);
        self.client.put(&path, bank_account).await
    }

    /// Delete a bank account.
    pub async fn delete(&self, id: &str) -> Result<()> {
        let path = format!("/bankaccounts/{}", id);
        self.client.delete(&path).await
    }
}
