# Spiris Bokföring och Fakturering API Client for Rust

[![Crates.io](https://img.shields.io/crates/v/spiris.svg)](https://crates.io/crates/spiris)
[![Documentation](https://docs.rs/spiris/badge.svg)](https://docs.rs/spiris)
[![CI](https://github.com/jimmystridh/spiris-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/jimmystridh/spiris-rust/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE-MIT)
[![MSRV](https://img.shields.io/badge/MSRV-1.70-blue.svg)](https://blog.rust-lang.org/2023/06/01/Rust-1.70.0.html)

A comprehensive Rust client library for the [Spiris Bokföring och Fakturering API](https://developer.visma.com/api/eaccounting) (formerly Visma eAccounting).

## Features

- **OAuth2 Authentication**: Complete OAuth2 flow support with PKCE and token refresh
- **Type-safe API**: Strongly typed request/response models
- **Async/Await**: Built on tokio and reqwest for async operations
- **Automatic Retries**: Exponential backoff for transient failures
- **Rate Limiting**: Automatic handling of API rate limits (600 req/min)
- **Configurable**: Builder patterns for client and retry configuration
- **Observability**: Optional tracing support for request/response logging
- **Comprehensive Coverage**: Support for customers, invoices, articles, and more
- **Error Handling**: Rich error types with detailed information
- **Production Ready**: CI/CD, comprehensive tests, and battle-tested
- **Well-documented**: Extensive documentation and examples

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
spiris = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

## Quick Start

```rust
use spiris::{Client, AccessToken};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create an access token (usually obtained via OAuth2)
    let token = AccessToken::new("your_access_token".to_string(), 3600, None);

    // Create the API client
    let client = Client::new(token);

    // List customers
    let customers = client.customers().list(None).await?;
    println!("Found {} customers", customers.data.len());

    Ok(())
}
```

## Authentication

The Spiris API uses OAuth2 for authentication. Here's how to authenticate:

### 1. Register Your Application

First, register your application in the [Visma Developer Portal](https://developer.visma.com/) to obtain:
- Client ID
- Client Secret
- Redirect URI

### 2. Implement OAuth2 Flow

```rust
use spiris::auth::{OAuth2Config, OAuth2Handler};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = OAuth2Config::new(
        "your_client_id".to_string(),
        "your_client_secret".to_string(),
        "http://localhost:8080/callback".to_string(),
    );

    let handler = OAuth2Handler::new(config)?;

    // Get authorization URL
    let (auth_url, csrf_token, pkce_verifier) = handler.authorize_url();
    println!("Visit this URL to authorize: {}", auth_url);

    // After user authorizes and you receive the code...
    let token = handler.exchange_code(code, pkce_verifier).await?;

    // Use the token to create a client
    let client = Client::new(token);

    Ok(())
}
```

## Usage Examples

### List Customers with Pagination

```rust
use spiris::{Client, AccessToken, PaginationParams};

let token = AccessToken::new("your_token".to_string(), 3600, None);
let client = Client::new(token);

let params = PaginationParams::new().page(0).pagesize(100);
let customers = client.customers().list(Some(params)).await?;

for customer in customers.data {
    println!("{:?}: {:?}", customer.id, customer.name);
}
```

### Create a Customer

```rust
use spiris::{Client, Customer, Address};

let new_customer = Customer {
    name: Some("Acme Corporation".to_string()),
    email: Some("contact@acme.com".to_string()),
    phone: Some("+46123456789".to_string()),
    invoice_address: Some(Address {
        address1: Some("123 Main Street".to_string()),
        city: Some("Stockholm".to_string()),
        postal_code: Some("11122".to_string()),
        country_code: Some("SE".to_string()),
        ..Default::default()
    }),
    is_active: Some(true),
    payment_terms_in_days: Some(30),
    ..Default::default()
};

let created = client.customers().create(&new_customer).await?;
println!("Created customer with ID: {:?}", created.id);
```

### Create an Invoice

```rust
use spiris::{Client, Invoice, InvoiceRow};
use chrono::Utc;

let invoice = Invoice {
    customer_id: Some("customer-id-here".to_string()),
    invoice_date: Some(Utc::now()),
    currency_code: Some("SEK".to_string()),
    rows: vec![
        InvoiceRow {
            text: Some("Consulting services".to_string()),
            unit_price: Some(1000.0),
            quantity: Some(10.0),
            ..Default::default()
        }
    ],
    ..Default::default()
};

let created_invoice = client.invoices().create(&invoice).await?;
println!("Invoice #{:?} created", created_invoice.invoice_number);
```

### Search with Filters

```rust
use spiris::QueryParams;

let query = QueryParams::new()
    .filter("IsActive eq true")
    .select("Id,Name,Email");

let active_customers = client.customers().search(query, None).await?;
```

### Manage Articles/Products

```rust
use spiris::Article;

let article = Article {
    name: Some("Consulting Hour".to_string()),
    unit: Some("hours".to_string()),
    sales_price: Some(1200.0),
    is_active: Some(true),
    ..Default::default()
};

let created_article = client.articles().create(&article).await?;
```

## API Feature Matrix

### Endpoints Implemented

| Endpoint | API Path | List | Get | Create | Update | Delete | Search | Extra |
|----------|----------|:----:|:---:|:------:|:------:|:------:|:------:|-------|
| **Customers** | | | | | | | | |
| Customers | `/customers` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | |
| Customer Invoice Drafts | `/customerinvoicedrafts` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | convert |
| Customer Invoices | `/customerinvoices` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | payments, pdf, einvoice |
| Customer Ledger Items | `/customerledgeritems` | ✓ | ✓ | ✓ | | | ✓ | |
| Customer Labels | `/customerlabels` | ✓ | ✓ | ✓ | ✓ | ✓ | | |
| **Suppliers** | | | | | | | | |
| Suppliers | `/suppliers` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | |
| Supplier Invoices | `/supplierinvoices` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | payments |
| Supplier Invoice Drafts | `/supplierinvoicedrafts` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | convert |
| Supplier Ledger Items | `/supplierledgeritems` | ✓ | ✓ | ✓ | | | ✓ | |
| Supplier Labels | `/supplierlabels` | ✓ | ✓ | ✓ | ✓ | ✓ | | |
| **Articles** | | | | | | | | |
| Articles | `/articles` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | |
| Article Labels | `/articlelabels` | ✓ | ✓ | ✓ | ✓ | ✓ | | |
| Article Account Codings | `/articleaccountcodings` | ✓ | ✓ | ✓ | ✓ | ✓ | | |
| Units | `/units` | ✓ | ✓ | ✓ | ✓ | ✓ | | |
| **Accounting** | | | | | | | | |
| Accounts | `/accounts` | ✓ | ✓ | ✓ | ✓ | | | balances, types, standard |
| Fiscal Years | `/fiscalyears` | ✓ | ✓ | ✓ | | | | opening balances |
| VAT Codes | `/vatcodes` | ✓ | ✓ | | | | | |
| Vouchers | `/vouchers` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | |
| **Banking** | | | | | | | | |
| Bank Accounts | `/bankaccounts` | ✓ | ✓ | ✓ | ✓ | ✓ | | |
| Banks | `/banks` | ✓ | | | | | | foreign payment codes |
| **Projects & Cost Centers** | | | | | | | | |
| Projects | `/projects` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | |
| Cost Centers | `/costcenters` | ✓ | ✓ | ✓ | ✓ | ✓ | | items |
| Allocation Periods | `/allocationperiods` | ✓ | ✓ | | | | | |
| **Orders & Quotations** | | | | | | | | |
| Orders | `/orders` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | |
| Quotations | `/quotations` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | |
| **Delivery & Payment** | | | | | | | | |
| Delivery Methods | `/deliverymethods` | ✓ | ✓ | ✓ | ✓ | ✓ | | |
| Delivery Terms | `/deliveryterms` | ✓ | ✓ | ✓ | ✓ | ✓ | | |
| Terms of Payment | `/termsofpayment` | ✓ | ✓ | ✓ | ✓ | ✓ | | |
| **Documents** | | | | | | | | |
| Attachments | `/attachments` | ✓ | ✓ | ✓ | | ✓ | | upload binary |
| Documents | `/documents` | ✓ | ✓ | | | | | |
| **Settings & Reference** | | | | | | | | |
| Company Settings | `/companysettings` | | ✓ | | ✓ | | | |
| Countries | `/countries` | ✓ | | | | | | |
| Currencies | `/currencies` | ✓ | | | | | | |
| Users | `/users` | ✓ | ✓ | | | | | |
| **Messaging & Approvals** | | | | | | | | |
| Message Threads | `/messagethreads` | | ✓ | | ✓ | | | add message |
| Approvals | `/approval/*` | | | | | | | VAT report, supplier invoice |

**Total: 35+ endpoints with full CRUD operations where applicable**

### Query Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `page` | `u32` | Page number (0-indexed) |
| `pagesize` | `u32` | Items per page (default: 50, max: 500) |
| `filter` | `String` | OData filter expression (e.g., `"IsActive eq true"`) |
| `select` | `String` | Fields to return (e.g., `"Id,Name,Email"`) |

### Data Types

**Core Entities:**
| Type | Fields |
|------|--------|
| `Customer` | id, customer_number, corporate_identity_number, name, email, phone, mobile_phone, website, invoice_address, delivery_address, payment_terms_in_days, is_active, is_private_person, created_utc, modified_utc |
| `Supplier` | id, supplier_number, corporate_identity_number, name, email, phone, mobile_phone, website, address, bank_account_number, bank_giro_number, plus_giro_number, is_active, created_utc, modified_utc |
| `Article` | id, article_number, name, unit, sales_price, purchase_price, is_active, vat_rate_id, created_utc, modified_utc |

**Invoices:**
| Type | Fields |
|------|--------|
| `Invoice` | id, invoice_number, customer_id, invoice_date, due_date, delivery_date, currency_code, rows, total_amount, total_vat_amount, total_amount_including_vat, is_sent, remarks, created_utc, modified_utc |
| `CustomerInvoiceDraft` | id, customer_id, invoice_date, due_date, delivery_date, currency_code, rows, total_amount, total_vat_amount, total_amount_including_vat, remarks, your_reference, our_reference, created_utc, modified_utc |
| `SupplierInvoice` | id, supplier_id, invoice_number, invoice_date, due_date, currency_code, currency_rate, rows, total_amount, total_vat_amount, total_amount_including_vat, is_paid, payment_date, ocr_number, created_utc, modified_utc |
| `InvoiceRow` | id, article_id, text, unit_price, quantity, discount_percentage, vat_rate_id, total_amount |

**Accounting:**
| Type | Fields |
|------|--------|
| `Account` | account_number, name, account_type, vat_code_id, fiscal_year_id, is_active, opening_balance |
| `Voucher` | id, voucher_number, voucher_date, voucher_type, voucher_text, rows, created_utc, modified_utc |
| `VoucherRow` | account_number, debit_amount, credit_amount, transaction_text, cost_center_item_id, project_id |
| `FiscalYear` | id, start_date, end_date, is_locked, bookkeeping_method |
| `VatCode` | id, code, description, vat_rate |

**Banking & Payments:**
| Type | Fields |
|------|--------|
| `BankAccount` | id, name, account_number, iban, bic, ledger_account_number, currency_code, is_default, is_active |
| `InvoicePayment` | amount, payment_date, bank_account_id, payment_reference_number, currency_rate |
| `CustomerLedgerItem` | id, customer_id, customer_invoice_id, currency_amount, currency_code, amount, payment_date, payment_reference_number, voucher_id, voucher_number, created_utc |

**Projects & Cost Centers:**
| Type | Fields |
|------|--------|
| `Project` | id, number, name, start_date, end_date, customer_id, notes, is_completed, is_active, created_utc, modified_utc |
| `CostCenter` | id, number, name, is_active |
| `CostCenterItem` | id, cost_center_id, name, short_name, is_active |
| `AllocationPeriod` | id, start_date, end_date, name, is_locked |

**Orders & Quotations:**
| Type | Fields |
|------|--------|
| `Order` | id, order_number, customer_id, order_date, delivery_date, currency_code, rows, total_amount, is_invoiced, created_utc, modified_utc |
| `Quotation` | id, quotation_number, customer_id, quotation_date, valid_until_date, currency_code, rows, total_amount, is_accepted, created_utc, modified_utc |

**Delivery & Payment:**
| Type | Fields |
|------|--------|
| `DeliveryMethod` | id, name, code, is_active |
| `DeliveryTerm` | id, name, code, is_active |
| `TermsOfPayment` | id, name, code, number_of_days, is_active |

**Documents:**
| Type | Fields |
|------|--------|
| `Attachment` | id, name, content_type, size, temporary_url, created_utc |
| `AttachmentLink` | entity_type, entity_id, attachment_id |
| `Document` | id, document_number, document_type, document_date, voucher_id, attachment_id |

**Settings & Reference:**
| Type | Fields |
|------|--------|
| `CompanySettings` | name, corporate_identity_number, address, phone, email, website, currency_code, country_code, fiscal_year_start_month |
| `Country` | code, name |
| `Currency` | code, name |
| `User` | id, email, first_name, last_name, is_active, role |
| `Bank` | id, name, bic, country_code |
| `ForeignPaymentCode` | id, code, name |

**Messaging & Approvals:**
| Type | Fields |
|------|--------|
| `MessageThread` | id, subject, entity_type, entity_id, is_read, messages, created_utc, modified_utc |
| `Message` | id, body, is_from_user, created_utc |
| `ApprovalAction` | is_approved, comment |

**Other:**
| Type | Fields |
|------|--------|
| `Address` | address1, address2, postal_code, city, country_code |
| `CustomerLabel` | id, name, description |
| `SupplierLabel` | id, name, description |
| `ArticleLabel` | id, name, description |
| `ArticleAccountCoding` | id, article_id, account_coding_type, sales_account_number, purchase_account_number |
| `Unit` | id, code, name, is_active |

### Authentication Features

| Feature | Supported |
|---------|:---------:|
| OAuth2 Authorization Code + PKCE | ✓ |
| Token refresh | ✓ |
| Token expiration check (5-min buffer) | ✓ |
| Scopes: `ea:api`, `ea:sales`, `offline_access` | ✓ |

### Client Features

| Feature | Supported | Configuration |
|---------|:---------:|---------------|
| Automatic retry with exponential backoff | ✓ | `RetryConfig` |
| Rate limit handling (429) | ✓ | Auto-retry |
| Server error retry (5xx) | ✓ | Auto-retry |
| Configurable timeout | ✓ | `ClientConfig.timeout_seconds` |
| Custom base URL | ✓ | `ClientConfig.base_url` |
| Tracing/logging | ✓ | `ClientConfig.enable_tracing` |
| Thread-safe token updates | ✓ | `Arc<RwLock<AccessToken>>` |

### Error Types

| Error | Description |
|-------|-------------|
| `TokenExpired` | Access token expired (not retried) |
| `RateLimitExceeded` | 429 response (retried) |
| `NotFound` | 404 response |
| `InvalidRequest` | 400 response |
| `AuthError` | 401/403 response |
| `ApiError` | Other HTTP errors |
| `OAuth2Error` | OAuth2 flow failures |
| `Http` | Network/connection errors (retried) |

### RetryConfig Options

| Option | Default | Description |
|--------|---------|-------------|
| `max_retries` | 3 | Maximum retry attempts |
| `initial_interval` | 500ms | Initial backoff duration |
| `max_interval` | 30s | Maximum backoff duration |
| `multiplier` | 2.0 | Exponential backoff multiplier |
| `max_elapsed_time` | 120s | Total time before giving up |

## Error Handling

The library provides comprehensive error handling:

```rust
use spiris::Error;

match client.customers().get("invalid-id").await {
    Ok(customer) => println!("Found customer: {:?}", customer.name),
    Err(Error::NotFound(msg)) => println!("Customer not found: {}", msg),
    Err(Error::TokenExpired) => println!("Token expired, please refresh"),
    Err(Error::RateLimitExceeded(msg)) => println!("Rate limit hit: {}", msg),
    Err(e) => println!("Error: {}", e),
}
```

## Rate Limiting

The Spiris API has a rate limit of **600 requests per minute** per client per endpoint. The library automatically handles rate limit errors and returns appropriate error types.

## Token Expiration and Refresh

Access tokens expire after 1 hour. The library checks token expiration before making requests and provides built-in token refresh:

```rust
use spiris::auth::{OAuth2Config, OAuth2Handler};

// Check if token is expired
if client.is_token_expired() {
    let current_token = client.get_access_token();

    if let Some(refresh_token) = current_token.refresh_token {
        // Use the OAuth2 handler to refresh the token
        let config = OAuth2Config::new(/* ... */);
        let handler = OAuth2Handler::new(config)?;
        let new_token = handler.refresh_token(refresh_token).await?;

        // Update the client with the new token
        client.set_access_token(new_token);
    }
}
```

## Advanced Configuration

The client supports extensive configuration for production use:

```rust
use spiris::{Client, AccessToken, ClientConfig, RetryConfig};
use std::time::Duration;

let token = AccessToken::new("token".to_string(), 3600, None);

// Configure retry behavior
let retry_config = RetryConfig::new()
    .max_retries(5)
    .initial_interval(Duration::from_secs(1))
    .max_interval(Duration::from_secs(30));

// Create client with custom configuration
let config = ClientConfig::new()
    .base_url("https://eaccountingapi.vismaonline.com/v2/")
    .timeout_seconds(60)
    .retry_config(retry_config)
    .enable_tracing(true);

let client = Client::with_config(token, config);
```

## Retry Logic

The client automatically retries failed requests with exponential backoff:

- **Network errors**: Automatically retried
- **Rate limits (429)**: Automatically retried with backoff
- **Server errors (5xx)**: Automatically retried
- **Client errors (4xx)**: Not retried (permanent errors)

Configure retry behavior:

```rust
let retry_config = RetryConfig::new()
    .max_retries(3)                                  // Max retry attempts
    .initial_interval(Duration::from_millis(500))    // Initial backoff
    .max_interval(Duration::from_secs(30))           // Max backoff
    .multiplier(2.0);                                // Backoff multiplier
```

## Examples

The `examples/` directory contains complete working examples:

- `oauth_flow.rs`: OAuth2 authentication flow
- `list_customers.rs`: List customers with pagination
- `create_customer.rs`: Create a new customer
- `create_invoice.rs`: Create an invoice

Run an example with:

```bash
export SPIRIS_ACCESS_TOKEN="your_token_here"
cargo run --example list_customers
```

## Testing

Run the test suite:

```bash
cargo test
```

Run specific tests:

```bash
# Run only library tests
cargo test --lib

# Run only integration tests
cargo test --test '*'

# Run with output
cargo test -- --nocapture
```

## Performance Tips

### Connection Pooling

The client uses reqwest's built-in connection pooling. Reuse the same `Client` instance for multiple requests:

```rust
// Good: Reuse client
let client = Client::new(token);
for customer_id in customer_ids {
    let customer = client.customers().get(customer_id).await?;
}

// Bad: Creating new client for each request
for customer_id in customer_ids {
    let client = Client::new(token.clone());
    let customer = client.customers().get(customer_id).await?;
}
```

### Batch Operations

When possible, use pagination to fetch multiple records in one request:

```rust
// Fetch 100 customers at once instead of 100 individual requests
let params = PaginationParams::new().pagesize(100);
let customers = client.customers().list(Some(params)).await?;
```

### Timeout Configuration

Adjust timeouts based on your network conditions:

```rust
let config = ClientConfig::new()
    .timeout_seconds(60)  // Increase for slower networks
    .retry_config(
        RetryConfig::new()
            .max_retries(5)
            .initial_interval(Duration::from_secs(2))
    );
```

## Security Best Practices

### Never Hardcode Credentials

Always use environment variables or secure configuration management:

```rust
// Good
let token = std::env::var("SPIRIS_ACCESS_TOKEN")?;

// Bad - Never do this!
// let token = "hardcoded_token_12345";
```

### Token Storage

Store refresh tokens securely:

```rust
use std::fs;
use std::os::unix::fs::PermissionsExt;

// Write token to file with restricted permissions
let token_json = serde_json::to_string(&token)?;
fs::write(".spiris_token", token_json)?;
fs::set_permissions(".spiris_token", fs::Permissions::from_mode(0o600))?;
```

### HTTPS Only

The client uses HTTPS by default. Never modify the base URL to use HTTP:

```rust
// The default is already HTTPS - don't change it
const DEFAULT_BASE_URL: &str = "https://eaccountingapi.vismaonline.com/v2/";
```

## Troubleshooting

### Token Expired Errors

If you're getting `TokenExpired` errors:

```rust
// Check token expiration before making requests
if client.is_token_expired() {
    // Refresh the token
    let current_token = client.get_access_token();
    if let Some(refresh_token) = current_token.refresh_token {
        let handler = OAuth2Handler::new(oauth_config)?;
        let new_token = handler.refresh_token(refresh_token).await?;
        client.set_access_token(new_token);
    }
}
```

### Rate Limiting

If you're hitting rate limits (600 requests/minute):

```rust
// Configure more aggressive retry backoff
let retry_config = RetryConfig::new()
    .max_retries(10)
    .initial_interval(Duration::from_secs(5))
    .max_interval(Duration::from_secs(60));

let config = ClientConfig::new().retry_config(retry_config);
let client = Client::with_config(token, config);
```

### Network Timeouts

For unreliable networks:

```rust
let config = ClientConfig::new()
    .timeout_seconds(120)  // 2 minutes
    .retry_config(
        RetryConfig::new()
            .max_retries(5)
            .initial_interval(Duration::from_secs(2))
    );
```

### Debugging API Requests

Enable tracing to see detailed request/response information:

```rust
// Add to Cargo.toml
// tracing-subscriber = "0.3"

use tracing_subscriber;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = ClientConfig::new().enable_tracing(true);
    let client = Client::with_config(token, config);

    // Now you'll see detailed logs
}
```

## FAQ

### Q: Do I need to manually refresh tokens?

A: The client checks token expiration before each request and returns a `TokenExpired` error if the token is expired. You can either:
1. Manually refresh using `OAuth2Handler::refresh_token()`
2. Implement automatic refresh logic in your application

### Q: What's the rate limit?

A: The API has a rate limit of **600 requests per minute** per client per endpoint. The client automatically retries rate-limited requests with exponential backoff.

### Q: Can I use this with multiple accounts?

A: Yes! Create separate `Client` instances for each account with different access tokens:

```rust
let client1 = Client::new(token1);
let client2 = Client::new(token2);
```

### Q: How do I handle pagination for large datasets?

A: Use a loop to fetch all pages:

```rust
let mut all_customers = Vec::new();
let mut page = 0;
let pagesize = 100;

loop {
    let params = PaginationParams::new().page(page).pagesize(pagesize);
    let response = client.customers().list(Some(params)).await?;

    all_customers.extend(response.data);

    if !response.meta.has_next_page {
        break;
    }
    page += 1;
}
```

### Q: What happens if my API call fails?

A: The client automatically retries transient failures (network errors, rate limits, 5xx errors) with exponential backoff. Permanent errors (4xx) are returned immediately.

### Q: Can I customize the retry behavior?

A: Yes! See the "Retry Logic" section for configuration options.

### Q: Is this thread-safe?

A: Yes! The `Client` can be safely cloned and shared across threads:

```rust
let client = Client::new(token);

// Clone for use in different threads
let client1 = client.clone();
let client2 = client.clone();

tokio::spawn(async move {
    client1.customers().list(None).await
});

tokio::spawn(async move {
    client2.invoices().list(None).await
});
```

### Q: What's the difference between Spiris and Visma eAccounting?

A: Spiris Bokföring och Fakturering is the new name for Visma eAccounting. All API endpoints and functionality remain exactly the same - only the branding has changed.

## Migration Guide

### From visma_eaccounting to spiris

If you were using an earlier version with the `visma_eaccounting` package name:

1. Update `Cargo.toml`:
```toml
[dependencies]
# Old
# visma_eaccounting = "0.1.0"

# New
spiris = "0.1.0"
```

2. Update imports:
```rust
// Old
use visma_eaccounting::{Client, AccessToken};

// New
use spiris::{Client, AccessToken};
```

3. Update environment variables:
```bash
# Old
export VISMA_ACCESS_TOKEN="..."
export VISMA_CLIENT_ID="..."

# New
export SPIRIS_ACCESS_TOKEN="..."
export SPIRIS_CLIENT_ID="..."
```

All API functionality remains identical - no code changes needed beyond the import statements.

## Documentation

Generate and view the documentation:

```bash
cargo doc --open
```

Browse available modules:
- `spiris::auth` - OAuth2 authentication
- `spiris::client` - HTTP client
- `spiris::endpoints` - API endpoints
- `spiris::error` - Error types
- `spiris::types` - Data models
- `spiris::retry` - Retry configuration

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/jimmystridh/spiris-rust
cd spiris-rust

# Run tests
cargo test

# Run examples (requires API credentials)
export SPIRIS_ACCESS_TOKEN="your_token"
cargo run --example list_customers

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy -- -D warnings

# Build documentation
cargo doc --no-deps
```

### Reporting Issues

When reporting issues, please include:
- Rust version (`rustc --version`)
- Package version
- Minimal code example
- Error messages
- Expected vs actual behavior

## License

This project is licensed under the MIT license. See [LICENSE-MIT](LICENSE-MIT) for details.

## Resources

- [Spiris Bokföring och Fakturering API Documentation](https://developer.visma.com/api/eaccounting)
- [Visma Developer Portal](https://developer.visma.com/)
- [API Authentication Guide](https://developer.vismaonline.com/docs/authentication)
- [Visma Community Forum](https://community.visma.com/t5/Visma-eAccounting-API/ct-p/IN_MA_eAccountingAPI)

## Note

Spiris Bokföring och Fakturering was formerly known as Visma eAccounting. All API endpoints and technical details remain the same.

## Disclaimer

This is an unofficial client library and is not affiliated with or endorsed by Visma or Spiris. Use at your own risk.
