//! Customers API endpoint.

use crate::client::Client;
use crate::error::Result;
use crate::types::{Customer, PaginatedResponse, PaginationParams, QueryParams};

/// Customers endpoint for managing customer records.
///
/// # Example
///
/// ```no_run
/// use visma_eaccounting::{Client, AccessToken};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let token = AccessToken::new("your_token".to_string(), 3600, None);
///     let client = Client::new(token);
///     let customers = client.customers();
///
///     // List all customers
///     let all_customers = customers.list(None).await?;
///     println!("Found {} customers", all_customers.data.len());
///
///     Ok(())
/// }
/// ```
pub struct CustomersEndpoint<'a> {
    client: &'a Client,
}

impl<'a> CustomersEndpoint<'a> {
    /// Create a new customers endpoint.
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// List all customers with optional pagination.
    ///
    /// # Arguments
    ///
    /// * `params` - Optional pagination parameters
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use visma_eaccounting::{Client, AccessToken, PaginationParams};
    /// # async fn example(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    /// let params = PaginationParams::new().page(0).pagesize(100);
    /// let customers = client.customers().list(Some(params)).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list(
        &self,
        params: Option<PaginationParams>,
    ) -> Result<PaginatedResponse<Customer>> {
        if let Some(params) = params {
            self.client.get_with_params("/customers", &params).await
        } else {
            self.client.get("/customers").await
        }
    }

    /// Get a specific customer by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The customer ID (GUID)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use visma_eaccounting::Client;
    /// # async fn example(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    /// let customer = client.customers().get("customer-id-123").await?;
    /// println!("Customer: {}", customer.name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get(&self, id: &str) -> Result<Customer> {
        let path = format!("/customers/{}", id);
        self.client.get(&path).await
    }

    /// Create a new customer.
    ///
    /// # Arguments
    ///
    /// * `customer` - The customer data to create
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use visma_eaccounting::{Client, Customer};
    /// # async fn example(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    /// let new_customer = Customer {
    ///     name: "Acme Corporation".to_string(),
    ///     email: Some("contact@acme.com".to_string()),
    ///     ..Default::default()
    /// };
    /// let created = client.customers().create(&new_customer).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(&self, customer: &Customer) -> Result<Customer> {
        self.client.post("/customers", customer).await
    }

    /// Update an existing customer.
    ///
    /// # Arguments
    ///
    /// * `id` - The customer ID (GUID)
    /// * `customer` - The updated customer data
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use visma_eaccounting::Client;
    /// # async fn example(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    /// let mut customer = client.customers().get("customer-id-123").await?;
    /// customer.email = Some("newemail@acme.com".to_string());
    /// let updated = client.customers().update("customer-id-123", &customer).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update(&self, id: &str, customer: &Customer) -> Result<Customer> {
        let path = format!("/customers/{}", id);
        self.client.put(&path, customer).await
    }

    /// Delete a customer.
    ///
    /// # Arguments
    ///
    /// * `id` - The customer ID (GUID)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use visma_eaccounting::Client;
    /// # async fn example(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    /// client.customers().delete("customer-id-123").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete(&self, id: &str) -> Result<()> {
        let path = format!("/customers/{}", id);
        self.client.delete(&path).await
    }

    /// Search customers with custom query parameters.
    ///
    /// # Arguments
    ///
    /// * `query` - Query parameters for filtering and field selection
    /// * `pagination` - Optional pagination parameters
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use visma_eaccounting::{Client, QueryParams};
    /// # async fn example(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    /// let query = QueryParams::new()
    ///     .filter("IsActive eq true")
    ///     .select("Id,Name,Email");
    /// let customers = client.customers().search(query, None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn search(
        &self,
        query: QueryParams,
        pagination: Option<PaginationParams>,
    ) -> Result<PaginatedResponse<Customer>> {
        #[derive(serde::Serialize)]
        struct CombinedParams {
            #[serde(flatten)]
            query: QueryParams,
            #[serde(flatten)]
            pagination: Option<PaginationParams>,
        }

        let params = CombinedParams { query, pagination };
        self.client.get_with_params("/customers", &params).await
    }
}
