//! Customer invoices API endpoint.

use crate::client::Client;
use crate::error::Result;
use crate::types::{Invoice, InvoicePayment, PaginatedResponse, PaginationParams, QueryParams};

/// Invoices endpoint for managing customer invoices.
///
/// # Example
///
/// ```no_run
/// use spiris::{Client, AccessToken};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let token = AccessToken::new("your_token".to_string(), 3600, None);
///     let client = Client::new(token);
///     let invoices = client.invoices();
///
///     // List all invoices
///     let all_invoices = invoices.list(None).await?;
///     println!("Found {} invoices", all_invoices.data.len());
///
///     Ok(())
/// }
/// ```
pub struct InvoicesEndpoint<'a> {
    client: &'a Client,
}

impl<'a> InvoicesEndpoint<'a> {
    /// Create a new invoices endpoint.
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// List all customer invoices with optional pagination.
    ///
    /// # Arguments
    ///
    /// * `params` - Optional pagination parameters
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use spiris::{Client, PaginationParams};
    /// # async fn example(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    /// let params = PaginationParams::new().page(0).pagesize(50);
    /// let invoices = client.invoices().list(Some(params)).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list(
        &self,
        params: Option<PaginationParams>,
    ) -> Result<PaginatedResponse<Invoice>> {
        if let Some(params) = params {
            self.client
                .get_with_params("/customerinvoices", &params)
                .await
        } else {
            self.client.get("/customerinvoices").await
        }
    }

    /// Get a specific invoice by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The invoice ID (GUID)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use spiris::Client;
    /// # async fn example(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    /// let invoice = client.invoices().get("invoice-id-123").await?;
    /// println!("Invoice #{:?}", invoice.invoice_number);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get(&self, id: &str) -> Result<Invoice> {
        let path = format!("/customerinvoices/{}", id);
        self.client.get(&path).await
    }

    /// Create a new customer invoice.
    ///
    /// # Arguments
    ///
    /// * `invoice` - The invoice data to create
    ///
    /// # Example
    ///
    /// ```ignore
    /// # use spiris::{Client, Invoice, InvoiceRow, money};
    /// # use chrono::Utc;
    /// # async fn example(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    /// let new_invoice = Invoice {
    ///     customer_id: Some("customer-id-123".to_string()),
    ///     invoice_date: Some(Utc::now()),
    ///     rows: vec![
    ///         InvoiceRow {
    ///             text: Some("Consulting services".to_string()),
    ///             unit_price: Some(money!(1000.0)),
    ///             quantity: Some(money!(10.0)),
    ///             ..Default::default()
    ///         }
    ///     ],
    ///     ..Default::default()
    /// };
    /// let created = client.invoices().create(&new_invoice).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(&self, invoice: &Invoice) -> Result<Invoice> {
        self.client.post("/customerinvoices", invoice).await
    }

    /// Update an existing invoice.
    ///
    /// # Arguments
    ///
    /// * `id` - The invoice ID (GUID)
    /// * `invoice` - The updated invoice data
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use spiris::Client;
    /// # async fn example(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    /// let mut invoice = client.invoices().get("invoice-id-123").await?;
    /// invoice.remarks = Some("Updated remarks".to_string());
    /// let updated = client.invoices().update("invoice-id-123", &invoice).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update(&self, id: &str, invoice: &Invoice) -> Result<Invoice> {
        let path = format!("/customerinvoices/{}", id);
        self.client.put(&path, invoice).await
    }

    /// Delete an invoice.
    ///
    /// # Arguments
    ///
    /// * `id` - The invoice ID (GUID)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use spiris::Client;
    /// # async fn example(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    /// client.invoices().delete("invoice-id-123").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete(&self, id: &str) -> Result<()> {
        let path = format!("/customerinvoices/{}", id);
        self.client.delete(&path).await
    }

    /// Search invoices with custom query parameters.
    ///
    /// # Arguments
    ///
    /// * `query` - Query parameters for filtering and field selection
    /// * `pagination` - Optional pagination parameters
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use spiris::{Client, QueryParams};
    /// # async fn example(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    /// let query = QueryParams::new()
    ///     .filter("IsSent eq true");
    /// let invoices = client.invoices().search(query, None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn search(
        &self,
        query: QueryParams,
        pagination: Option<PaginationParams>,
    ) -> Result<PaginatedResponse<Invoice>> {
        #[derive(serde::Serialize)]
        struct CombinedParams {
            #[serde(flatten)]
            query: QueryParams,
            #[serde(flatten)]
            pagination: Option<PaginationParams>,
        }

        let params = CombinedParams { query, pagination };
        self.client
            .get_with_params("/customerinvoices", &params)
            .await
    }

    /// Register a payment for an invoice.
    ///
    /// # Arguments
    ///
    /// * `invoice_id` - The invoice ID
    /// * `payment` - The payment details
    pub async fn register_payment(&self, invoice_id: &str, payment: &InvoicePayment) -> Result<()> {
        let path = format!("/customerinvoices/{}/payments", invoice_id);
        self.client.post::<(), _>(&path, payment).await?;
        Ok(())
    }

    /// Get the PDF for an invoice.
    ///
    /// # Arguments
    ///
    /// * `invoice_id` - The invoice ID
    ///
    /// # Returns
    ///
    /// The PDF as raw bytes.
    pub async fn get_pdf(&self, invoice_id: &str) -> Result<Vec<u8>> {
        let path = format!("/customerinvoices/{}/pdf", invoice_id);
        self.client.get_bytes(&path).await
    }

    /// Send an invoice via e-invoice.
    ///
    /// # Arguments
    ///
    /// * `invoice_id` - The invoice ID to send electronically
    pub async fn send_einvoice(&self, invoice_id: &str) -> Result<()> {
        let path = format!("/customerinvoices/{}/einvoice", invoice_id);
        self.client.post::<(), _>(&path, &()).await?;
        Ok(())
    }

    /// Stream all invoices, automatically paginating through results.
    ///
    /// Requires the `stream` feature.
    #[cfg(feature = "stream")]
    pub fn list_stream(&self) -> impl futures::Stream<Item = Result<Invoice>> + '_ {
        self.list_stream_with_page_size(crate::pagination::DEFAULT_PAGE_SIZE)
    }

    /// Stream all invoices with a custom page size.
    ///
    /// Requires the `stream` feature.
    #[cfg(feature = "stream")]
    pub fn list_stream_with_page_size(
        &self,
        page_size: u32,
    ) -> impl futures::Stream<Item = Result<Invoice>> + '_ {
        crate::paginated_stream!(page_size, |params| self.list(Some(params)))
    }
}
