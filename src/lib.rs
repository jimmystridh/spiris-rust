//! # Visma eAccounting API Client for Rust
//!
//! This crate provides a Rust client for the [Visma eAccounting API](https://developer.visma.com/api/eaccounting).
//!
//! ## Features
//!
//! - **OAuth2 Authentication**: Complete OAuth2 flow support with token refresh
//! - **Type-safe API**: Strongly typed request/response models
//! - **Async/Await**: Built on tokio and reqwest for async operations
//! - **Rate Limiting**: Automatic handling of API rate limits
//! - **Comprehensive Coverage**: Support for customers, invoices, articles, and more
//!
//! ## Quick Start
//!
//! ```no_run
//! use visma_eaccounting::{Client, AccessToken};
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
//! use visma_eaccounting::auth::{OAuth2Config, OAuth2Handler};
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
//! use visma_eaccounting::{Client, AccessToken, Customer, PaginationParams};
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
//! ```no_run
//! use visma_eaccounting::{Client, AccessToken, Invoice, InvoiceRow};
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
//!             unit_price: Some(1000.0),
//!             quantity: Some(10.0),
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

pub mod auth;
pub mod client;
pub mod endpoints;
pub mod error;
pub mod types;

// Re-export commonly used types
pub use auth::{AccessToken, OAuth2Config, OAuth2Handler};
pub use client::{Client, ClientConfig};
pub use error::{Error, Result};
pub use types::{
    Address, Article, Customer, Invoice, InvoiceRow, PaginatedResponse, PaginationParams,
    QueryParams, ResponseMetadata,
};

// Add endpoint accessors to the Client
impl Client {
    /// Access the customers endpoint.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use visma_eaccounting::{Client, AccessToken};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let token = AccessToken::new("token".to_string(), 3600, None);
    /// let client = Client::new(token);
    /// let customers = client.customers().list(None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn customers(&self) -> endpoints::CustomersEndpoint {
        endpoints::CustomersEndpoint::new(self)
    }

    /// Access the invoices endpoint.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use visma_eaccounting::{Client, AccessToken};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let token = AccessToken::new("token".to_string(), 3600, None);
    /// let client = Client::new(token);
    /// let invoices = client.invoices().list(None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn invoices(&self) -> endpoints::InvoicesEndpoint {
        endpoints::InvoicesEndpoint::new(self)
    }

    /// Access the articles endpoint.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use visma_eaccounting::{Client, AccessToken};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let token = AccessToken::new("token".to_string(), 3600, None);
    /// let client = Client::new(token);
    /// let articles = client.articles().list(None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn articles(&self) -> endpoints::ArticlesEndpoint {
        endpoints::ArticlesEndpoint::new(self)
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
}
