//! Common types and data models for the Visma eAccounting API.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Pagination parameters for list requests.
#[derive(Debug, Clone, Default, Serialize)]
pub struct PaginationParams {
    /// Page number (default: 0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,

    /// Page size (default: 50, max: 500).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagesize: Option<u32>,
}

impl PaginationParams {
    /// Create new pagination parameters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the page number.
    pub fn page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    /// Set the page size.
    pub fn pagesize(mut self, pagesize: u32) -> Self {
        self.pagesize = Some(pagesize);
        self
    }
}

/// Response wrapper for paginated list requests.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PaginatedResponse<T> {
    /// The data items.
    pub data: Vec<T>,

    /// Metadata about the response.
    pub meta: ResponseMetadata,
}

/// Metadata included in API responses.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ResponseMetadata {
    /// Current page number.
    pub current_page: u32,

    /// Page size.
    pub page_size: u32,

    /// Total number of pages.
    pub total_pages: u32,

    /// Total number of items.
    pub total_count: u32,

    /// Whether there are more pages.
    pub has_next_page: bool,

    /// Whether there are previous pages.
    pub has_previous_page: bool,
}

/// Customer in the eAccounting system.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Customer {
    /// Unique identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Customer number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_number: Option<String>,

    /// Corporate identity number (organization number).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub corporate_identity_number: Option<String>,

    /// Customer name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Email address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    /// Phone number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,

    /// Mobile phone number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mobile_phone: Option<String>,

    /// Website URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,

    /// Invoice address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invoice_address: Option<Address>,

    /// Delivery address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivery_address: Option<Address>,

    /// Payment terms in days.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_terms_in_days: Option<u32>,

    /// Whether the customer is active.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,

    /// Whether the customer is private (person).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_private_person: Option<bool>,

    /// When the customer was created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_utc: Option<DateTime<Utc>>,

    /// When the customer was last modified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_utc: Option<DateTime<Utc>>,
}

/// Address information.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Address {
    /// Street address line 1.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address1: Option<String>,

    /// Street address line 2.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address2: Option<String>,

    /// Postal code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub postal_code: Option<String>,

    /// City.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,

    /// Country code (ISO 3166-1 alpha-2).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_code: Option<String>,
}

/// Invoice/Customer invoice.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Invoice {
    /// Unique identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Invoice number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invoice_number: Option<String>,

    /// Customer ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_id: Option<String>,

    /// Invoice date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invoice_date: Option<DateTime<Utc>>,

    /// Due date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<DateTime<Utc>>,

    /// Delivery date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivery_date: Option<DateTime<Utc>>,

    /// Currency code (ISO 4217).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<String>,

    /// Invoice rows/line items.
    pub rows: Vec<InvoiceRow>,

    /// Total amount excluding VAT.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_amount: Option<f64>,

    /// Total VAT amount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_vat_amount: Option<f64>,

    /// Total amount including VAT.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_amount_including_vat: Option<f64>,

    /// Whether the invoice is sent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_sent: Option<bool>,

    /// Remarks/notes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remarks: Option<String>,

    /// When the invoice was created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_utc: Option<DateTime<Utc>>,

    /// When the invoice was last modified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_utc: Option<DateTime<Utc>>,
}

/// Invoice row/line item.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct InvoiceRow {
    /// Unique identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Article/product ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub article_id: Option<String>,

    /// Description/text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,

    /// Unit price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit_price: Option<f64>,

    /// Quantity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<f64>,

    /// Discount percentage (0-100).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discount_percentage: Option<f64>,

    /// VAT rate ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vat_rate_id: Option<String>,

    /// Total amount for this row.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_amount: Option<f64>,
}

/// Article/Product.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Article {
    /// Unique identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Article number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub article_number: Option<String>,

    /// Article name/description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Unit label (e.g., "pcs", "hours").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,

    /// Sales price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sales_price: Option<f64>,

    /// Purchase price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purchase_price: Option<f64>,

    /// Whether the article is active.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,

    /// VAT rate ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vat_rate_id: Option<String>,

    /// When the article was created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_utc: Option<DateTime<Utc>>,

    /// When the article was last modified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_utc: Option<DateTime<Utc>>,
}

/// Generic query parameters for filtering and selecting fields.
#[derive(Debug, Clone, Default, Serialize)]
pub struct QueryParams {
    /// Filter expression.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,

    /// Fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub select: Option<String>,

    /// Additional custom parameters.
    #[serde(flatten)]
    pub extra: HashMap<String, String>,
}

impl QueryParams {
    /// Create new query parameters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a filter expression.
    pub fn filter(mut self, filter: impl Into<String>) -> Self {
        self.filter = Some(filter.into());
        self
    }

    /// Set fields to select.
    pub fn select(mut self, select: impl Into<String>) -> Self {
        self.select = Some(select.into());
        self
    }

    /// Add a custom parameter.
    pub fn param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.extra.insert(key.into(), value.into());
        self
    }
}
