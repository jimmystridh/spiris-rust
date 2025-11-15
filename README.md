# Visma eAccounting API Client for Rust

[![Crates.io](https://img.shields.io/crates/v/visma_eaccounting.svg)](https://crates.io/crates/visma_eaccounting)
[![Documentation](https://docs.rs/visma_eaccounting/badge.svg)](https://docs.rs/visma_eaccounting)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

A comprehensive Rust client library for the [Visma eAccounting API](https://developer.visma.com/api/eaccounting).

## Features

- **OAuth2 Authentication**: Complete OAuth2 flow support with PKCE
- **Type-safe API**: Strongly typed request/response models
- **Async/Await**: Built on tokio and reqwest for async operations
- **Rate Limiting**: Automatic handling of API rate limits (600 req/min)
- **Comprehensive Coverage**: Support for customers, invoices, articles, and more
- **Error Handling**: Rich error types with detailed information
- **Well-documented**: Extensive documentation and examples

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
visma_eaccounting = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

## Quick Start

```rust
use visma_eaccounting::{Client, AccessToken};

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

The Visma eAccounting API uses OAuth2 for authentication. Here's how to authenticate:

### 1. Register Your Application

First, register your application in the [Visma Developer Portal](https://developer.visma.com/) to obtain:
- Client ID
- Client Secret
- Redirect URI

### 2. Implement OAuth2 Flow

```rust
use visma_eaccounting::auth::{OAuth2Config, OAuth2Handler};

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
use visma_eaccounting::{Client, AccessToken, PaginationParams};

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
use visma_eaccounting::{Client, Customer, Address};

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
use visma_eaccounting::{Client, Invoice, InvoiceRow};
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
use visma_eaccounting::QueryParams;

let query = QueryParams::new()
    .filter("IsActive eq true")
    .select("Id,Name,Email");

let active_customers = client.customers().search(query, None).await?;
```

### Manage Articles/Products

```rust
use visma_eaccounting::Article;

let article = Article {
    name: Some("Consulting Hour".to_string()),
    unit: Some("hours".to_string()),
    sales_price: Some(1200.0),
    is_active: Some(true),
    ..Default::default()
};

let created_article = client.articles().create(&article).await?;
```

## API Coverage

The library currently supports the following endpoints:

- **Customers**: List, Get, Create, Update, Delete, Search
- **Invoices**: List, Get, Create, Update, Delete, Search
- **Articles**: List, Get, Create, Update, Delete, Search

More endpoints will be added in future releases.

## Error Handling

The library provides comprehensive error handling:

```rust
use visma_eaccounting::Error;

match client.customers().get("invalid-id").await {
    Ok(customer) => println!("Found customer: {:?}", customer.name),
    Err(Error::NotFound(msg)) => println!("Customer not found: {}", msg),
    Err(Error::TokenExpired) => println!("Token expired, please refresh"),
    Err(Error::RateLimitExceeded(msg)) => println!("Rate limit hit: {}", msg),
    Err(e) => println!("Error: {}", e),
}
```

## Rate Limiting

The Visma eAccounting API has a rate limit of **600 requests per minute** per client per endpoint. The library automatically handles rate limit errors and returns appropriate error types.

## Token Expiration

Access tokens expire after 1 hour. The library checks token expiration before making requests:

```rust
if client.is_token_expired() {
    // Refresh your token here
    let new_token = refresh_token().await?;
    client.set_access_token(new_token);
}
```

## Examples

The `examples/` directory contains complete working examples:

- `oauth_flow.rs`: OAuth2 authentication flow
- `list_customers.rs`: List customers with pagination
- `create_customer.rs`: Create a new customer
- `create_invoice.rs`: Create an invoice

Run an example with:

```bash
export VISMA_ACCESS_TOKEN="your_token_here"
cargo run --example list_customers
```

## Testing

Run the test suite:

```bash
cargo test
```

## Documentation

Generate and view the documentation:

```bash
cargo doc --open
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Resources

- [Visma eAccounting API Documentation](https://developer.visma.com/api/eaccounting)
- [Visma Developer Portal](https://developer.visma.com/)
- [API Authentication Guide](https://developer.vismaonline.com/docs/authentication)
- [Visma Community Forum](https://community.visma.com/t5/Visma-eAccounting-API/ct-p/IN_MA_eAccountingAPI)

## Disclaimer

This is an unofficial client library and is not affiliated with or endorsed by Visma. Use at your own risk.
