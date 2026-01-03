//! # Spiris Bokföring och Fakturering API Client for Rust
//!
//! This crate provides a Rust client for the [Spiris Bokföring och Fakturering API](https://developer.visma.com/api/eaccounting) (formerly Visma eAccounting).
//!
//! ## Features
//!
//! - **OAuth2 Authentication**: Complete OAuth2 flow support with token refresh
//! - **Type-safe API**: Strongly typed request/response models
//! - **Async/Await**: Built on tokio and reqwest for async operations
//! - **Automatic Retries**: Exponential backoff for transient failures
//! - **Request Tracing**: Built-in logging support with tracing
//! - **Rate Limiting**: Automatic handling of API rate limits
//! - **Comprehensive Coverage**: Support for customers, invoices, articles, and more
//!
//! ## Quick Start
//!
//! ```no_run
//! use spiris::{Client, AccessToken};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create an access token (usually obtained via OAuth2)
//!     let token = AccessToken::new("your_access_token".to_string(), 3600, None);
//!
//!     // Create the API client
//!     let client = Client::new(token);
//!
//!     // List customers
//!     let customers = client.customers().list(None).await?;
//!     println!("Found {} customers", customers.data.len());
//!
//!     Ok(())
//! }
//! ```
//!
//! ## OAuth2 Authentication
//!
//! ```no_run
//! use spiris::auth::{OAuth2Config, OAuth2Handler};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = OAuth2Config::new(
//!         "your_client_id".to_string(),
//!         "your_client_secret".to_string(),
//!         "http://localhost:8080/callback".to_string(),
//!     );
//!
//!     let handler = OAuth2Handler::new(config)?;
//!
//!     // Get authorization URL
//!     let (auth_url, csrf_token, pkce_verifier) = handler.authorize_url();
//!     println!("Visit this URL to authorize: {}", auth_url);
//!
//!     // After user authorizes and you receive the code...
//!     // let token = handler.exchange_code(code, pkce_verifier).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Working with Customers
//!
//! ```no_run
//! use spiris::{Client, AccessToken, Customer, PaginationParams};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let token = AccessToken::new("token".to_string(), 3600, None);
//! # let client = Client::new(token);
//! // Create a new customer
//! let new_customer = Customer {
//!     name: Some("Acme Corporation".to_string()),
//!     email: Some("contact@acme.com".to_string()),
//!     phone: Some("+1234567890".to_string()),
//!     ..Default::default()
//! };
//!
//! let created = client.customers().create(&new_customer).await?;
//! println!("Created customer with ID: {:?}", created.id);
//!
//! // List customers with pagination
//! let params = PaginationParams::new().page(0).pagesize(100);
//! let customers = client.customers().list(Some(params)).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Creating Invoices
//!
//! ```ignore
//! use spiris::{Client, AccessToken, Invoice, InvoiceRow, money};
//! use chrono::Utc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let token = AccessToken::new("token".to_string(), 3600, None);
//! # let client = Client::new(token);
//! let invoice = Invoice {
//!     customer_id: Some("customer-id-here".to_string()),
//!     invoice_date: Some(Utc::now()),
//!     rows: vec![
//!         InvoiceRow {
//!             text: Some("Consulting services".to_string()),
//!             unit_price: Some(money!(1000.0)),
//!             quantity: Some(money!(10.0)),
//!             ..Default::default()
//!         }
//!     ],
//!     ..Default::default()
//! };
//!
//! let created_invoice = client.invoices().create(&invoice).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Advanced Configuration
//!
//! ```no_run
//! use spiris::{Client, AccessToken, ClientConfig, RetryConfig};
//! use std::time::Duration;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let token = AccessToken::new("token".to_string(), 3600, None);
//!
//! // Configure retry behavior
//! let retry_config = RetryConfig::new()
//!     .max_retries(5)
//!     .initial_interval(Duration::from_millis(1000));
//!
//! // Create client with custom configuration
//! let config = ClientConfig::new()
//!     .timeout_seconds(60)
//!     .retry_config(retry_config)
//!     .enable_tracing(true);
//!
//! let client = Client::with_config(token, config);
//! # Ok(())
//! # }
//! ```

pub mod auth;
pub mod client;
pub mod endpoints;
pub mod error;
#[macro_use]
pub mod macros;
pub mod middleware;
#[cfg(feature = "stream")]
pub mod pagination;
pub mod query;
#[cfg(feature = "rate-limit")]
pub mod rate_limit;
pub mod retry;
pub mod types;
#[cfg(feature = "webhooks")]
pub mod webhooks;

// Re-export commonly used types
pub use auth::{AccessToken, OAuth2Config, OAuth2Handler};
pub use client::{Client, ClientConfig};
pub use error::{ApiErrorResponse, Error, Result, ValidationError};
#[cfg(feature = "rate-limit")]
pub use rate_limit::RateLimitConfig;
pub use retry::RetryConfig;
pub use types::{
    Account, AccountBalance, AccountType, Address, AllocationPeriod, Article, ArticleAccountCoding,
    ArticleCreate, ArticleLabel, ArticleUpdate, Attachment, AttachmentLink, Bank, BankAccount,
    CompanySettings, ConvertDraftOptions, CostCenter, CostCenterItem, Country, Currency, Customer,
    CustomerCreate, CustomerInvoiceDraft, CustomerInvoiceDraftRow, CustomerLabel, CustomerLedgerItem,
    CustomerUpdate, DeliveryMethod, DeliveryTerm, Document, FiscalYear, ForeignPaymentCode, Invoice,
    InvoiceCreate, InvoicePayment, InvoiceRow, InvoiceRowCreate, InvoiceUpdate, Message, MessageThread,
    Money, Order, OrderRow, PaginatedResponse, PaginationParams, Project, QueryParams, Quotation,
    QuotationRow, ResponseMetadata, Supplier, SupplierInvoice, SupplierInvoiceDraft, SupplierInvoiceRow,
    SupplierLabel, SupplierLedgerItem, TermsOfPayment, Unit, User, VatCode, Voucher, VoucherRow,
};

// Add endpoint accessors to the Client
impl Client {
    /// Access the customers endpoint.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use spiris::{Client, AccessToken};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let token = AccessToken::new("token".to_string(), 3600, None);
    /// let client = Client::new(token);
    /// let customers = client.customers().list(None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn customers(&self) -> endpoints::CustomersEndpoint<'_> {
        endpoints::CustomersEndpoint::new(self)
    }

    /// Access the invoices endpoint.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use spiris::{Client, AccessToken};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let token = AccessToken::new("token".to_string(), 3600, None);
    /// let client = Client::new(token);
    /// let invoices = client.invoices().list(None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn invoices(&self) -> endpoints::InvoicesEndpoint<'_> {
        endpoints::InvoicesEndpoint::new(self)
    }

    /// Access the articles endpoint.
    pub fn articles(&self) -> endpoints::ArticlesEndpoint<'_> {
        endpoints::ArticlesEndpoint::new(self)
    }

    /// Access the customer invoice drafts endpoint.
    pub fn customer_invoice_drafts(&self) -> endpoints::CustomerInvoiceDraftsEndpoint<'_> {
        endpoints::CustomerInvoiceDraftsEndpoint::new(self)
    }

    /// Access the customer ledger items endpoint.
    pub fn customer_ledger_items(&self) -> endpoints::CustomerLedgerItemsEndpoint<'_> {
        endpoints::CustomerLedgerItemsEndpoint::new(self)
    }

    /// Access the customer labels endpoint.
    pub fn customer_labels(&self) -> endpoints::CustomerLabelsEndpoint<'_> {
        endpoints::CustomerLabelsEndpoint::new(self)
    }

    /// Access the suppliers endpoint.
    pub fn suppliers(&self) -> endpoints::SuppliersEndpoint<'_> {
        endpoints::SuppliersEndpoint::new(self)
    }

    /// Access the supplier invoices endpoint.
    pub fn supplier_invoices(&self) -> endpoints::SupplierInvoicesEndpoint<'_> {
        endpoints::SupplierInvoicesEndpoint::new(self)
    }

    /// Access the accounts endpoint.
    pub fn accounts(&self) -> endpoints::AccountsEndpoint<'_> {
        endpoints::AccountsEndpoint::new(self)
    }

    /// Access the fiscal years endpoint.
    pub fn fiscal_years(&self) -> endpoints::FiscalYearsEndpoint<'_> {
        endpoints::FiscalYearsEndpoint::new(self)
    }

    /// Access the VAT codes endpoint.
    pub fn vat_codes(&self) -> endpoints::VatCodesEndpoint<'_> {
        endpoints::VatCodesEndpoint::new(self)
    }

    /// Access the vouchers endpoint.
    pub fn vouchers(&self) -> endpoints::VouchersEndpoint<'_> {
        endpoints::VouchersEndpoint::new(self)
    }

    /// Access the bank accounts endpoint.
    pub fn bank_accounts(&self) -> endpoints::BankAccountsEndpoint<'_> {
        endpoints::BankAccountsEndpoint::new(self)
    }

    /// Access the projects endpoint.
    pub fn projects(&self) -> endpoints::ProjectsEndpoint<'_> {
        endpoints::ProjectsEndpoint::new(self)
    }

    /// Access the cost centers endpoint.
    pub fn cost_centers(&self) -> endpoints::CostCentersEndpoint<'_> {
        endpoints::CostCentersEndpoint::new(self)
    }

    /// Access the allocation periods endpoint.
    pub fn allocation_periods(&self) -> endpoints::AllocationPeriodsEndpoint<'_> {
        endpoints::AllocationPeriodsEndpoint::new(self)
    }

    /// Access the orders endpoint.
    pub fn orders(&self) -> endpoints::OrdersEndpoint<'_> {
        endpoints::OrdersEndpoint::new(self)
    }

    /// Access the quotations endpoint.
    pub fn quotations(&self) -> endpoints::QuotationsEndpoint<'_> {
        endpoints::QuotationsEndpoint::new(self)
    }

    /// Access the supplier invoice drafts endpoint.
    pub fn supplier_invoice_drafts(&self) -> endpoints::SupplierInvoiceDraftsEndpoint<'_> {
        endpoints::SupplierInvoiceDraftsEndpoint::new(self)
    }

    /// Access the supplier ledger items endpoint.
    pub fn supplier_ledger_items(&self) -> endpoints::SupplierLedgerItemsEndpoint<'_> {
        endpoints::SupplierLedgerItemsEndpoint::new(self)
    }

    /// Access the supplier labels endpoint.
    pub fn supplier_labels(&self) -> endpoints::SupplierLabelsEndpoint<'_> {
        endpoints::SupplierLabelsEndpoint::new(self)
    }

    /// Access the article labels endpoint.
    pub fn article_labels(&self) -> endpoints::ArticleLabelsEndpoint<'_> {
        endpoints::ArticleLabelsEndpoint::new(self)
    }

    /// Access the article account codings endpoint.
    pub fn article_account_codings(&self) -> endpoints::ArticleAccountCodingsEndpoint<'_> {
        endpoints::ArticleAccountCodingsEndpoint::new(self)
    }

    /// Access the units endpoint.
    pub fn units(&self) -> endpoints::UnitsEndpoint<'_> {
        endpoints::UnitsEndpoint::new(self)
    }

    /// Access the delivery methods endpoint.
    pub fn delivery_methods(&self) -> endpoints::DeliveryMethodsEndpoint<'_> {
        endpoints::DeliveryMethodsEndpoint::new(self)
    }

    /// Access the delivery terms endpoint.
    pub fn delivery_terms(&self) -> endpoints::DeliveryTermsEndpoint<'_> {
        endpoints::DeliveryTermsEndpoint::new(self)
    }

    /// Access the terms of payment endpoint.
    pub fn terms_of_payment(&self) -> endpoints::TermsOfPaymentEndpoint<'_> {
        endpoints::TermsOfPaymentEndpoint::new(self)
    }

    /// Access the attachments endpoint.
    pub fn attachments(&self) -> endpoints::AttachmentsEndpoint<'_> {
        endpoints::AttachmentsEndpoint::new(self)
    }

    /// Access the documents endpoint.
    pub fn documents(&self) -> endpoints::DocumentsEndpoint<'_> {
        endpoints::DocumentsEndpoint::new(self)
    }

    /// Access the company settings endpoint.
    pub fn company_settings(&self) -> endpoints::CompanySettingsEndpoint<'_> {
        endpoints::CompanySettingsEndpoint::new(self)
    }

    /// Access the countries endpoint.
    pub fn countries(&self) -> endpoints::CountriesEndpoint<'_> {
        endpoints::CountriesEndpoint::new(self)
    }

    /// Access the currencies endpoint.
    pub fn currencies(&self) -> endpoints::CurrenciesEndpoint<'_> {
        endpoints::CurrenciesEndpoint::new(self)
    }

    /// Access the users endpoint.
    pub fn users(&self) -> endpoints::UsersEndpoint<'_> {
        endpoints::UsersEndpoint::new(self)
    }

    /// Access the banks endpoint.
    pub fn banks(&self) -> endpoints::BanksEndpoint<'_> {
        endpoints::BanksEndpoint::new(self)
    }

    /// Access the messages endpoint.
    pub fn messages(&self) -> endpoints::MessagesEndpoint<'_> {
        endpoints::MessagesEndpoint::new(self)
    }

    /// Access the approvals endpoint.
    pub fn approvals(&self) -> endpoints::ApprovalsEndpoint<'_> {
        endpoints::ApprovalsEndpoint::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let token = AccessToken::new("test_token".to_string(), 3600, None);
        let client = Client::new(token);
        assert!(!client.is_token_expired());
    }

    #[test]
    fn test_customer_default() {
        let customer = Customer::default();
        assert!(customer.id.is_none());
        assert!(customer.name.is_none());
    }

    #[test]
    fn test_invoice_default() {
        let invoice = Invoice::default();
        assert!(invoice.id.is_none());
        assert!(invoice.customer_id.is_none());
    }

    #[test]
    fn test_pagination_params() {
        let params = PaginationParams::new().page(2).pagesize(50);
        assert_eq!(params.page, Some(2));
        assert_eq!(params.pagesize, Some(50));
    }

    #[test]
    fn test_query_params() {
        let params = QueryParams::new()
            .filter("IsActive eq true")
            .select("Id,Name");
        assert_eq!(params.filter, Some("IsActive eq true".to_string()));
        assert_eq!(params.select, Some("Id,Name".to_string()));
    }

    #[test]
    fn test_client_config_builder() {
        let config = ClientConfig::new()
            .timeout_seconds(60)
            .enable_tracing(false);

        assert_eq!(config.timeout_seconds, 60);
        assert!(!config.enable_tracing);
    }

    #[test]
    fn test_retry_config() {
        let retry = RetryConfig::new().max_retries(5);
        assert_eq!(retry.max_retries, 5);
    }
}
