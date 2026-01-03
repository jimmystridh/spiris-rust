//! Common types and data models for the Visma eAccounting API.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type alias for monetary values.
///
/// When the `decimal` feature is enabled, this is `rust_decimal::Decimal` for precise
/// financial calculations. Otherwise, it defaults to `f64` for simplicity.
///
/// # Example with decimal feature
///
/// ```toml
/// [dependencies]
/// spiris = { version = "0.1", features = ["decimal"] }
/// ```
///
/// ```ignore
/// use spiris::Money;
/// use rust_decimal_macros::dec;
///
/// let price: Money = dec!(100.50);
/// let quantity: Money = dec!(3);
/// let total = price * quantity;  // Precise calculation
/// assert_eq!(total, dec!(301.50));
/// ```
#[cfg(feature = "decimal")]
pub type Money = rust_decimal::Decimal;

/// Type alias for monetary values.
///
/// This is the default `f64` representation. For precise financial calculations,
/// enable the `decimal` feature to use `rust_decimal::Decimal` instead.
#[cfg(not(feature = "decimal"))]
pub type Money = f64;

/// Create a Money value from a float literal.
///
/// This macro works with both the `decimal` feature enabled (Decimal) and
/// disabled (f64), making test code portable.
///
/// # Example
///
/// ```
/// use spiris::{money, Money};
///
/// let price: Money = money!(100.50);
/// ```
#[macro_export]
macro_rules! money {
    ($val:expr) => {{
        #[cfg(feature = "decimal")]
        {
            use std::str::FromStr;
            rust_decimal::Decimal::from_str(stringify!($val)).expect("Invalid decimal literal")
        }
        #[cfg(not(feature = "decimal"))]
        {
            $val as f64
        }
    }};
}

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
    pub total_amount: Option<Money>,

    /// Total VAT amount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_vat_amount: Option<Money>,

    /// Total amount including VAT.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_amount_including_vat: Option<Money>,

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
    pub unit_price: Option<Money>,

    /// Quantity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<Money>,

    /// Discount percentage (0-100).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discount_percentage: Option<Money>,

    /// VAT rate ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vat_rate_id: Option<String>,

    /// Total amount for this row.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_amount: Option<Money>,
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
    pub sales_price: Option<Money>,

    /// Purchase price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purchase_price: Option<Money>,

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

// =============================================================================
// Customer Invoice Draft Types
// =============================================================================

/// Customer invoice draft (unpublished invoice).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CustomerInvoiceDraft {
    /// Unique identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

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
    #[serde(default)]
    pub rows: Vec<CustomerInvoiceDraftRow>,

    /// Total amount excluding VAT.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_amount: Option<Money>,

    /// Total VAT amount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_vat_amount: Option<Money>,

    /// Total amount including VAT.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_amount_including_vat: Option<Money>,

    /// Remarks/notes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remarks: Option<String>,

    /// Your reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub your_reference: Option<String>,

    /// Our reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub our_reference: Option<String>,

    /// When the draft was created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_utc: Option<DateTime<Utc>>,

    /// When the draft was last modified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_utc: Option<DateTime<Utc>>,
}

/// Customer invoice draft row/line item.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CustomerInvoiceDraftRow {
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
    pub unit_price: Option<Money>,

    /// Quantity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<Money>,

    /// Discount percentage (0-100).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discount_percentage: Option<Money>,

    /// VAT rate ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vat_rate_id: Option<String>,

    /// Total amount for this row.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_amount: Option<Money>,
}

/// Options for converting a draft to an invoice.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ConvertDraftOptions {
    /// How to send the invoice (0 = None, 1 = Email, 2 = Print, 3 = EInvoice).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_type: Option<i32>,
}

// =============================================================================
// Customer Ledger Item Types
// =============================================================================

/// Customer ledger item (payment/transaction record).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CustomerLedgerItem {
    /// Unique identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Customer ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_id: Option<String>,

    /// Customer invoice ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_invoice_id: Option<String>,

    /// Amount in currency.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_amount: Option<Money>,

    /// Currency code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<String>,

    /// Amount in domestic currency.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<Money>,

    /// Payment date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_date: Option<DateTime<Utc>>,

    /// Payment reference number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_reference_number: Option<String>,

    /// Voucher ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voucher_id: Option<String>,

    /// Voucher number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voucher_number: Option<String>,

    /// When the item was created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_utc: Option<DateTime<Utc>>,
}

// =============================================================================
// Customer Label Types
// =============================================================================

/// Customer label for categorization.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CustomerLabel {
    /// Unique identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Label name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Label description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

// =============================================================================
// Invoice Payment Types
// =============================================================================

/// Invoice payment record.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct InvoicePayment {
    /// Payment amount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<Money>,

    /// Payment date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_date: Option<DateTime<Utc>>,

    /// Bank account ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bank_account_id: Option<String>,

    /// Payment reference number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_reference_number: Option<String>,

    /// Currency rate (exchange rate).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_rate: Option<Money>,
}

// =============================================================================
// Supplier Types
// =============================================================================

/// Supplier in the eAccounting system.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Supplier {
    /// Unique identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Supplier number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supplier_number: Option<String>,

    /// Corporate identity number (organization number).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub corporate_identity_number: Option<String>,

    /// Supplier name.
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

    /// Address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<Address>,

    /// Bank account number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bank_account_number: Option<String>,

    /// Bank giro number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bank_giro_number: Option<String>,

    /// Plus giro number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plus_giro_number: Option<String>,

    /// Whether the supplier is active.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,

    /// When the supplier was created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_utc: Option<DateTime<Utc>>,

    /// When the supplier was last modified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_utc: Option<DateTime<Utc>>,
}

/// Supplier invoice.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SupplierInvoice {
    /// Unique identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Supplier ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supplier_id: Option<String>,

    /// Invoice number from supplier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invoice_number: Option<String>,

    /// Invoice date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invoice_date: Option<DateTime<Utc>>,

    /// Due date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<DateTime<Utc>>,

    /// Currency code (ISO 4217).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<String>,

    /// Currency rate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_rate: Option<Money>,

    /// Invoice rows/line items.
    #[serde(default)]
    pub rows: Vec<SupplierInvoiceRow>,

    /// Total amount excluding VAT.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_amount: Option<Money>,

    /// Total VAT amount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_vat_amount: Option<Money>,

    /// Total amount including VAT.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_amount_including_vat: Option<Money>,

    /// Whether the invoice is paid.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_paid: Option<bool>,

    /// Payment date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_date: Option<DateTime<Utc>>,

    /// OCR number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ocr_number: Option<String>,

    /// When the invoice was created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_utc: Option<DateTime<Utc>>,

    /// When the invoice was last modified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_utc: Option<DateTime<Utc>>,
}

/// Supplier invoice row/line item.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SupplierInvoiceRow {
    /// Unique identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Account number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_number: Option<String>,

    /// Description/text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,

    /// Amount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<Money>,

    /// VAT amount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vat_amount: Option<Money>,

    /// VAT rate ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vat_rate_id: Option<String>,

    /// Cost center item ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_center_item_id: Option<String>,

    /// Project ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
}

// =============================================================================
// Accounting Types
// =============================================================================

/// Account in the chart of accounts.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Account {
    /// Account number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_number: Option<String>,

    /// Account name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Account type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_type: Option<i32>,

    /// VAT code ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vat_code_id: Option<String>,

    /// Fiscal year ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fiscal_year_id: Option<String>,

    /// Whether the account is active.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,

    /// Opening balance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opening_balance: Option<Money>,
}

/// Account balance at a specific date.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AccountBalance {
    /// Account number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_number: Option<String>,

    /// Account name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Balance amount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance: Option<Money>,
}

/// Account type definition.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AccountType {
    /// Account type ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,

    /// Account type name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Fiscal year.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FiscalYear {
    /// Unique identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Start date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<DateTime<Utc>>,

    /// End date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<DateTime<Utc>>,

    /// Whether this is the current fiscal year.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_locked: Option<bool>,

    /// Bookkeeping method (1 = Invoice, 2 = Cash).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bookkeeping_method: Option<i32>,
}

/// VAT code.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct VatCode {
    /// Unique identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// VAT code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    /// Description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// VAT rate percentage.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vat_rate: Option<Money>,
}

/// Voucher (journal entry).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Voucher {
    /// Unique identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Voucher number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voucher_number: Option<String>,

    /// Voucher date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voucher_date: Option<DateTime<Utc>>,

    /// Voucher type (0 = Manual, etc.).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voucher_type: Option<i32>,

    /// Voucher text/description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voucher_text: Option<String>,

    /// Voucher rows.
    #[serde(default)]
    pub rows: Vec<VoucherRow>,

    /// When the voucher was created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_utc: Option<DateTime<Utc>>,

    /// When the voucher was last modified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_utc: Option<DateTime<Utc>>,
}

/// Voucher row (journal entry line).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct VoucherRow {
    /// Account number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_number: Option<String>,

    /// Debit amount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debit_amount: Option<Money>,

    /// Credit amount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credit_amount: Option<Money>,

    /// Transaction text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_text: Option<String>,

    /// Cost center item ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_center_item_id: Option<String>,

    /// Project ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
}

// =============================================================================
// Banking Types
// =============================================================================

/// Bank account.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BankAccount {
    /// Unique identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Bank account name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Bank account number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_number: Option<String>,

    /// IBAN.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iban: Option<String>,

    /// BIC/SWIFT.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bic: Option<String>,

    /// Ledger account number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ledger_account_number: Option<String>,

    /// Currency code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<String>,

    /// Whether this is the default bank account.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_default: Option<bool>,

    /// Whether the bank account is active.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
}

// =============================================================================
// Projects & Cost Centers
// =============================================================================

/// Project for tracking work/costs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Project {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_completed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_utc: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_utc: Option<DateTime<Utc>>,
}

/// Cost center for allocating expenses.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CostCenter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
}

/// Cost center item (specific allocation).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CostCenterItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_center_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
}

/// Allocation period for cost distribution.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AllocationPeriod {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<DateTime<Utc>>,
}

// =============================================================================
// Orders & Quotations
// =============================================================================

/// Sales order.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Order {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_date: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivery_date: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<String>,
    #[serde(default)]
    pub rows: Vec<OrderRow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_amount: Option<Money>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_vat_amount: Option<Money>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub your_reference: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub our_reference: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_utc: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_utc: Option<DateTime<Utc>>,
}

/// Order row/line item.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct OrderRow {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub article_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit_price: Option<Money>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<Money>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discount_percentage: Option<Money>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivered_quantity: Option<Money>,
}

/// Sales quotation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Quotation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quotation_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quotation_date: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_until_date: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<String>,
    #[serde(default)]
    pub rows: Vec<QuotationRow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_amount: Option<Money>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_vat_amount: Option<Money>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_utc: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_utc: Option<DateTime<Utc>>,
}

/// Quotation row/line item.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct QuotationRow {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub article_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit_price: Option<Money>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<Money>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discount_percentage: Option<Money>,
}

// =============================================================================
// Supplier Extensions
// =============================================================================

/// Supplier invoice draft.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SupplierInvoiceDraft {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supplier_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invoice_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invoice_date: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<String>,
    #[serde(default)]
    pub rows: Vec<SupplierInvoiceRow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_amount: Option<Money>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_utc: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_utc: Option<DateTime<Utc>>,
}

/// Supplier ledger item.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SupplierLedgerItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supplier_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supplier_invoice_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_amount: Option<Money>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<Money>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_date: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voucher_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_utc: Option<DateTime<Utc>>,
}

/// Supplier label for categorization.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SupplierLabel {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

// =============================================================================
// Article Extensions
// =============================================================================

/// Article label for categorization.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ArticleLabel {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Article account coding (GL mapping).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ArticleAccountCoding {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sales_account_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purchase_account_number: Option<String>,
}

/// Unit of measurement.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Unit {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

// =============================================================================
// Delivery & Payment Terms
// =============================================================================

/// Delivery method.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeliveryMethod {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Delivery terms.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeliveryTerm {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Terms of payment.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TermsOfPayment {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_english: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_of_days: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terms_of_payment_type: Option<i32>,
}

// =============================================================================
// Attachments & Documents
// =============================================================================

/// File attachment.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Attachment {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temporary_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_utc: Option<DateTime<Utc>>,
}

/// Link between attachment and document.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AttachmentLink {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachment_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_type: Option<i32>,
}

/// Document reference.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Document {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_type: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_number: Option<String>,
}

// =============================================================================
// Settings & Reference Data
// =============================================================================

/// Company settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CompanySettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub corporate_identity_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<Address>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_code: Option<String>,
}

/// Country.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Country {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub english_name: Option<String>,
}

/// Currency.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Currency {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// User.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct User {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
}

/// Bank reference data.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Bank {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bic: Option<String>,
}

/// Foreign payment code.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ForeignPaymentCode {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

// =============================================================================
// Messaging
// =============================================================================

/// Message thread.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MessageThread {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_type: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_id: Option<String>,
    #[serde(default)]
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_utc: Option<DateTime<Utc>>,
}

/// Message in a thread.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Message {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_utc: Option<DateTime<Utc>>,
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

    /// Set a filter expression from a string.
    ///
    /// For type-safe filter building, use `filter_by()` instead.
    pub fn filter(mut self, filter: impl Into<String>) -> Self {
        self.filter = Some(filter.into());
        self
    }

    /// Set a type-safe filter expression.
    ///
    /// # Example
    ///
    /// ```
    /// use spiris::{QueryParams, query::Filter};
    ///
    /// let params = QueryParams::new()
    ///     .filter_by(Filter::field("IsActive").eq(true));
    /// ```
    pub fn filter_by(mut self, filter: crate::query::Filter) -> Self {
        self.filter = Some(filter.to_string());
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

// =============================================================================
// Typed Request Types (Create/Update)
// =============================================================================
//
// These types provide compile-time enforcement of required fields for create
// and update operations. They separate concerns between:
// - Response types (what the API returns)
// - Create types (required fields for creating new entities)
// - Update types (all optional for partial updates)

/// Data required to create a new customer.
///
/// This type enforces that required fields (like `name`) are provided
/// at compile time, preventing runtime validation errors.
///
/// # Example
///
/// ```
/// use spiris::CustomerCreate;
///
/// let new_customer = CustomerCreate::new("Acme Corporation".to_string())
///     .email("contact@acme.com".to_string())
///     .phone("+46701234567".to_string());
/// ```
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CustomerCreate {
    /// Customer name (required).
    pub name: String,

    /// Customer number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_number: Option<String>,

    /// Corporate identity number (organization number).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub corporate_identity_number: Option<String>,

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
}

impl CustomerCreate {
    /// Create a new customer with the required name.
    pub fn new(name: String) -> Self {
        Self {
            name,
            customer_number: None,
            corporate_identity_number: None,
            email: None,
            phone: None,
            mobile_phone: None,
            website: None,
            invoice_address: None,
            delivery_address: None,
            payment_terms_in_days: None,
            is_active: None,
            is_private_person: None,
        }
    }

    /// Set the customer number.
    pub fn customer_number(mut self, value: String) -> Self {
        self.customer_number = Some(value);
        self
    }

    /// Set the corporate identity number.
    pub fn corporate_identity_number(mut self, value: String) -> Self {
        self.corporate_identity_number = Some(value);
        self
    }

    /// Set the email address.
    pub fn email(mut self, value: String) -> Self {
        self.email = Some(value);
        self
    }

    /// Set the phone number.
    pub fn phone(mut self, value: String) -> Self {
        self.phone = Some(value);
        self
    }

    /// Set the mobile phone number.
    pub fn mobile_phone(mut self, value: String) -> Self {
        self.mobile_phone = Some(value);
        self
    }

    /// Set the website URL.
    pub fn website(mut self, value: String) -> Self {
        self.website = Some(value);
        self
    }

    /// Set the invoice address.
    pub fn invoice_address(mut self, value: Address) -> Self {
        self.invoice_address = Some(value);
        self
    }

    /// Set the delivery address.
    pub fn delivery_address(mut self, value: Address) -> Self {
        self.delivery_address = Some(value);
        self
    }

    /// Set the payment terms in days.
    pub fn payment_terms_in_days(mut self, value: u32) -> Self {
        self.payment_terms_in_days = Some(value);
        self
    }

    /// Set whether the customer is active.
    pub fn is_active(mut self, value: bool) -> Self {
        self.is_active = Some(value);
        self
    }

    /// Set whether the customer is a private person.
    pub fn is_private_person(mut self, value: bool) -> Self {
        self.is_private_person = Some(value);
        self
    }
}

/// Data for updating an existing customer.
///
/// All fields are optional, allowing partial updates.
/// Only fields that are set will be included in the request.
///
/// # Example
///
/// ```
/// use spiris::CustomerUpdate;
///
/// // Update only the email
/// let update = CustomerUpdate::new()
///     .email("newemail@example.com".to_string());
/// ```
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CustomerUpdate {
    /// Customer name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Customer number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_number: Option<String>,

    /// Corporate identity number (organization number).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub corporate_identity_number: Option<String>,

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
}

impl CustomerUpdate {
    /// Create an empty update (use builder methods to set fields).
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the customer name.
    pub fn name(mut self, value: String) -> Self {
        self.name = Some(value);
        self
    }

    /// Set the customer number.
    pub fn customer_number(mut self, value: String) -> Self {
        self.customer_number = Some(value);
        self
    }

    /// Set the corporate identity number.
    pub fn corporate_identity_number(mut self, value: String) -> Self {
        self.corporate_identity_number = Some(value);
        self
    }

    /// Set the email address.
    pub fn email(mut self, value: String) -> Self {
        self.email = Some(value);
        self
    }

    /// Set the phone number.
    pub fn phone(mut self, value: String) -> Self {
        self.phone = Some(value);
        self
    }

    /// Set the mobile phone number.
    pub fn mobile_phone(mut self, value: String) -> Self {
        self.mobile_phone = Some(value);
        self
    }

    /// Set the website URL.
    pub fn website(mut self, value: String) -> Self {
        self.website = Some(value);
        self
    }

    /// Set the invoice address.
    pub fn invoice_address(mut self, value: Address) -> Self {
        self.invoice_address = Some(value);
        self
    }

    /// Set the delivery address.
    pub fn delivery_address(mut self, value: Address) -> Self {
        self.delivery_address = Some(value);
        self
    }

    /// Set the payment terms in days.
    pub fn payment_terms_in_days(mut self, value: u32) -> Self {
        self.payment_terms_in_days = Some(value);
        self
    }

    /// Set whether the customer is active.
    pub fn is_active(mut self, value: bool) -> Self {
        self.is_active = Some(value);
        self
    }

    /// Set whether the customer is a private person.
    pub fn is_private_person(mut self, value: bool) -> Self {
        self.is_private_person = Some(value);
        self
    }
}

/// Data required to create a new article.
///
/// # Example
///
/// ```
/// use spiris::{ArticleCreate, money};
///
/// let article = ArticleCreate::new("Consulting Services".to_string())
///     .article_number("SVC-001".to_string())
///     .sales_price(money!(1500.0));
/// ```
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ArticleCreate {
    /// Article name (required).
    pub name: String,

    /// Article number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub article_number: Option<String>,

    /// Unit label (e.g., "pcs", "hours").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,

    /// Sales price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sales_price: Option<Money>,

    /// Purchase price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purchase_price: Option<Money>,

    /// Whether the article is active.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,

    /// VAT rate ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vat_rate_id: Option<String>,
}

impl ArticleCreate {
    /// Create a new article with the required name.
    pub fn new(name: String) -> Self {
        Self {
            name,
            article_number: None,
            unit: None,
            sales_price: None,
            purchase_price: None,
            is_active: None,
            vat_rate_id: None,
        }
    }

    /// Set the article number.
    pub fn article_number(mut self, value: String) -> Self {
        self.article_number = Some(value);
        self
    }

    /// Set the unit label.
    pub fn unit(mut self, value: String) -> Self {
        self.unit = Some(value);
        self
    }

    /// Set the sales price.
    pub fn sales_price(mut self, value: impl Into<Money>) -> Self {
        self.sales_price = Some(value.into());
        self
    }

    /// Set the purchase price.
    pub fn purchase_price(mut self, value: impl Into<Money>) -> Self {
        self.purchase_price = Some(value.into());
        self
    }

    /// Set whether the article is active.
    pub fn is_active(mut self, value: bool) -> Self {
        self.is_active = Some(value);
        self
    }

    /// Set the VAT rate ID.
    pub fn vat_rate_id(mut self, value: String) -> Self {
        self.vat_rate_id = Some(value);
        self
    }
}

/// Data for updating an existing article.
///
/// All fields are optional, allowing partial updates.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ArticleUpdate {
    /// Article name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Article number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub article_number: Option<String>,

    /// Unit label.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,

    /// Sales price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sales_price: Option<Money>,

    /// Purchase price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purchase_price: Option<Money>,

    /// Whether the article is active.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,

    /// VAT rate ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vat_rate_id: Option<String>,
}

impl ArticleUpdate {
    /// Create an empty update.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the article name.
    pub fn name(mut self, value: String) -> Self {
        self.name = Some(value);
        self
    }

    /// Set the article number.
    pub fn article_number(mut self, value: String) -> Self {
        self.article_number = Some(value);
        self
    }

    /// Set the unit label.
    pub fn unit(mut self, value: String) -> Self {
        self.unit = Some(value);
        self
    }

    /// Set the sales price.
    pub fn sales_price(mut self, value: impl Into<Money>) -> Self {
        self.sales_price = Some(value.into());
        self
    }

    /// Set the purchase price.
    pub fn purchase_price(mut self, value: impl Into<Money>) -> Self {
        self.purchase_price = Some(value.into());
        self
    }

    /// Set whether the article is active.
    pub fn is_active(mut self, value: bool) -> Self {
        self.is_active = Some(value);
        self
    }

    /// Set the VAT rate ID.
    pub fn vat_rate_id(mut self, value: String) -> Self {
        self.vat_rate_id = Some(value);
        self
    }
}

/// A row/line item for creating an invoice.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct InvoiceRowCreate {
    /// Article/product ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub article_id: Option<String>,

    /// Description/text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,

    /// Unit price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit_price: Option<Money>,

    /// Quantity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<Money>,

    /// Discount percentage (0-100).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discount_percentage: Option<Money>,

    /// VAT rate ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vat_rate_id: Option<String>,
}

impl InvoiceRowCreate {
    /// Create a new invoice row.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an invoice row with an article reference.
    pub fn with_article(article_id: String, quantity: Money) -> Self {
        Self {
            article_id: Some(article_id),
            quantity: Some(quantity),
            ..Default::default()
        }
    }

    /// Create an invoice row with text description.
    pub fn with_text(text: String, unit_price: Money, quantity: Money) -> Self {
        Self {
            text: Some(text),
            unit_price: Some(unit_price),
            quantity: Some(quantity),
            ..Default::default()
        }
    }

    /// Set the article ID.
    pub fn article_id(mut self, value: String) -> Self {
        self.article_id = Some(value);
        self
    }

    /// Set the text description.
    pub fn text(mut self, value: String) -> Self {
        self.text = Some(value);
        self
    }

    /// Set the unit price.
    pub fn unit_price(mut self, value: impl Into<Money>) -> Self {
        self.unit_price = Some(value.into());
        self
    }

    /// Set the quantity.
    pub fn quantity(mut self, value: impl Into<Money>) -> Self {
        self.quantity = Some(value.into());
        self
    }

    /// Set the discount percentage.
    pub fn discount_percentage(mut self, value: impl Into<Money>) -> Self {
        self.discount_percentage = Some(value.into());
        self
    }

    /// Set the VAT rate ID.
    pub fn vat_rate_id(mut self, value: String) -> Self {
        self.vat_rate_id = Some(value);
        self
    }
}

/// Data required to create a new invoice.
///
/// # Example
///
/// ```
/// use spiris::{InvoiceCreate, InvoiceRowCreate, money};
///
/// let invoice = InvoiceCreate::new("customer-id-123".to_string())
///     .row(InvoiceRowCreate::with_text(
///         "Consulting".to_string(),
///         money!(1500.0),
///         money!(10.0),
///     ))
///     .remarks("Thank you for your business!".to_string());
/// ```
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct InvoiceCreate {
    /// Customer ID (required).
    pub customer_id: String,

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
    pub rows: Vec<InvoiceRowCreate>,

    /// Remarks/notes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remarks: Option<String>,
}

impl InvoiceCreate {
    /// Create a new invoice for a customer.
    pub fn new(customer_id: String) -> Self {
        Self {
            customer_id,
            invoice_date: None,
            due_date: None,
            delivery_date: None,
            currency_code: None,
            rows: Vec::new(),
            remarks: None,
        }
    }

    /// Set the invoice date.
    pub fn invoice_date(mut self, value: DateTime<Utc>) -> Self {
        self.invoice_date = Some(value);
        self
    }

    /// Set the due date.
    pub fn due_date(mut self, value: DateTime<Utc>) -> Self {
        self.due_date = Some(value);
        self
    }

    /// Set the delivery date.
    pub fn delivery_date(mut self, value: DateTime<Utc>) -> Self {
        self.delivery_date = Some(value);
        self
    }

    /// Set the currency code.
    pub fn currency_code(mut self, value: String) -> Self {
        self.currency_code = Some(value);
        self
    }

    /// Add a row to the invoice.
    pub fn row(mut self, row: InvoiceRowCreate) -> Self {
        self.rows.push(row);
        self
    }

    /// Set all rows at once.
    pub fn rows(mut self, rows: Vec<InvoiceRowCreate>) -> Self {
        self.rows = rows;
        self
    }

    /// Set the remarks/notes.
    pub fn remarks(mut self, value: String) -> Self {
        self.remarks = Some(value);
        self
    }
}

/// Data for updating an existing invoice.
///
/// All fields are optional, allowing partial updates.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct InvoiceUpdate {
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

    /// Currency code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<String>,

    /// Invoice rows (replaces existing rows if set).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rows: Option<Vec<InvoiceRowCreate>>,

    /// Remarks/notes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remarks: Option<String>,
}

impl InvoiceUpdate {
    /// Create an empty update.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the customer ID.
    pub fn customer_id(mut self, value: String) -> Self {
        self.customer_id = Some(value);
        self
    }

    /// Set the invoice date.
    pub fn invoice_date(mut self, value: DateTime<Utc>) -> Self {
        self.invoice_date = Some(value);
        self
    }

    /// Set the due date.
    pub fn due_date(mut self, value: DateTime<Utc>) -> Self {
        self.due_date = Some(value);
        self
    }

    /// Set the delivery date.
    pub fn delivery_date(mut self, value: DateTime<Utc>) -> Self {
        self.delivery_date = Some(value);
        self
    }

    /// Set the currency code.
    pub fn currency_code(mut self, value: String) -> Self {
        self.currency_code = Some(value);
        self
    }

    /// Set the invoice rows (replaces all existing rows).
    pub fn rows(mut self, rows: Vec<InvoiceRowCreate>) -> Self {
        self.rows = Some(rows);
        self
    }

    /// Set the remarks/notes.
    pub fn remarks(mut self, value: String) -> Self {
        self.remarks = Some(value);
        self
    }
}
