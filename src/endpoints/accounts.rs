//! Accounts API endpoint.

use crate::client::Client;
use crate::error::Result;
use crate::types::{Account, AccountBalance, AccountType, PaginatedResponse, PaginationParams};

/// Accounts endpoint for managing chart of accounts.
pub struct AccountsEndpoint<'a> {
    client: &'a Client,
}

impl<'a> AccountsEndpoint<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// List all accounts with optional pagination.
    pub async fn list(
        &self,
        params: Option<PaginationParams>,
    ) -> Result<PaginatedResponse<Account>> {
        if let Some(params) = params {
            self.client.get_with_params("/accounts", &params).await
        } else {
            self.client.get("/accounts").await
        }
    }

    /// List accounts for a specific fiscal year.
    pub async fn list_by_fiscal_year(
        &self,
        fiscal_year_id: &str,
        params: Option<PaginationParams>,
    ) -> Result<PaginatedResponse<Account>> {
        let path = format!("/accounts/{}", fiscal_year_id);
        if let Some(params) = params {
            self.client.get_with_params(&path, &params).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get a specific account by fiscal year and account number.
    pub async fn get(&self, fiscal_year_id: &str, account_number: &str) -> Result<Account> {
        let path = format!("/accounts/{}/{}", fiscal_year_id, account_number);
        self.client.get(&path).await
    }

    /// Create a new account.
    pub async fn create(&self, account: &Account) -> Result<Account> {
        self.client.post("/accounts", account).await
    }

    /// Update an existing account.
    pub async fn update(
        &self,
        fiscal_year_id: &str,
        account_number: &str,
        account: &Account,
    ) -> Result<Account> {
        let path = format!("/accounts/{}/{}", fiscal_year_id, account_number);
        self.client.put(&path, account).await
    }

    /// Get standard/predefined accounts.
    pub async fn get_standard_accounts(&self) -> Result<PaginatedResponse<Account>> {
        self.client.get("/accounts/standardaccounts").await
    }

    /// Get account balances at a specific date.
    pub async fn get_balances(&self, date: &str) -> Result<PaginatedResponse<AccountBalance>> {
        let path = format!("/accountbalances/{}", date);
        self.client.get(&path).await
    }

    /// Get balance for a specific account at a specific date.
    pub async fn get_balance(&self, account_number: &str, date: &str) -> Result<AccountBalance> {
        let path = format!("/accountbalances/{}/{}", account_number, date);
        self.client.get(&path).await
    }

    /// Get all account types.
    pub async fn get_account_types(&self) -> Result<PaginatedResponse<AccountType>> {
        self.client.get("/accountTypes").await
    }
}
