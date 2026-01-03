# Spiris Bokföring API Client - Improvement Plan

This document outlines proposed improvements to the Rust API client library, organized by priority and implementation phase.

---

## Executive Summary

The library provides comprehensive API coverage with 37 endpoints and clean async patterns. However, several issues need attention:

- ✅ ~~**2 Critical bugs** (PKCE broken, retry unused) requiring immediate fixes~~ **FIXED on 2026-01-03**
- **Core improvements** for production-readiness (token refresh, rate limiting, pagination)
- **DX improvements** for better ergonomics (typed filters, reduced boilerplate)
- **Advanced features** for enterprise use (middleware, observability)

Estimated total effort: 4-6 weeks for full implementation.

### Status Update (2026-01-03)
- ✅ **Phase 0 Complete**: Both critical bugs fixed (PKCE in auth.rs, retry in client.rs)
- 📊 **553 tests passing** + 12 ignored real API tests + performance benchmarks

---

## Phase 0: Critical Fixes (Immediate) ✅ COMPLETE

### 0.1 PKCE Verifier Not Used in Token Exchange ✅ FIXED

**Severity**: 🔴 Security vulnerability
**File**: `src/auth.rs:219-228`
**Status**: ✅ Fixed on 2026-01-03

#### Current State
```rust
pub async fn exchange_code(&self, code: String, _pkce_verifier: String) -> Result<AccessToken> {
    let token_result = self
        .client
        .exchange_code(AuthorizationCode::new(code))
        // PKCE verifier is NEVER sent!
        .request_async(oauth2::reqwest::async_http_client)
        .await
```

The PKCE verifier is generated, returned to the caller, but **discarded** during token exchange. This completely defeats PKCE protection against authorization code interception attacks.

#### Proposed Fix
```rust
pub async fn exchange_code(&self, code: String, pkce_verifier: String) -> Result<AccessToken> {
    use oauth2::PkceCodeVerifier;

    let token_result = self
        .client
        .exchange_code(AuthorizationCode::new(code))
        .set_pkce_verifier(PkceCodeVerifier::new(pkce_verifier))
        .request_async(oauth2::reqwest::async_http_client)
        .await
        .map_err(|e| Error::OAuth2Error(format!("Token exchange failed: {}", e)))?;
    // ...
}
```

#### Acceptance Criteria
- [x] PKCE verifier included in token request
- [x] Integration test with mock OAuth server validates PKCE flow
- [x] Documentation updated with security note

---

### 0.2 Retry Logic Exists But Never Invoked ✅ FIXED

**Severity**: 🔴 Correctness
**Files**: `src/client.rs:172-205`
**Status**: ✅ Fixed on 2026-01-03

#### Current State
`retry.rs` implements `retry_request()` with exponential backoff, but `client.rs` calls `request.send().await` directly without any retry wrapper.

#### Proposed Fix

Option A: Wrap at the execute level (minimal change):
```rust
// src/client.rs
async fn execute_request(&self, request: RequestBuilder) -> Result<Response> {
    if self.config.retry_config.max_retries > 0 {
        crate::retry::retry_request(&self.config.retry_config, || async {
            let req = request.try_clone().ok_or_else(|| {
                Error::InvalidRequest("Request body not cloneable".into())
            })?;
            let response = req.send().await?;
            self.handle_response(response).await
        }).await
    } else {
        let response = request.send().await?;
        self.handle_response(response).await
    }
}
```

Option B (preferred): Use `reqwest-retry` middleware for cleaner separation.

#### Acceptance Criteria
- [x] Transient errors (5xx, network) trigger retry with backoff
- [x] 4xx errors fail immediately (no retry)
- [x] Rate limit (429) is retryable but respects config
- [x] Max retries configurable and honored
- [x] MockApi disables retries by default for predictable testing
- [x] Unit tests verify retry behavior (tests/retry_test.rs)

---

## Phase 1: Core Production Readiness

### 1.1 Automatic Token Refresh ✅ COMPLETE

**Priority**: 🟠 High
**Files**: `src/client.rs` (lines 163-209)
**Status**: ✅ Implemented on 2026-01-03

#### Current State
When token expires, client returns `Error::TokenExpired`. User must manually refresh and update token.

#### Proposed Solution

```rust
// New client configuration
pub struct ClientConfig {
    // ... existing fields ...

    /// OAuth2 config for automatic token refresh (optional)
    pub oauth_config: Option<OAuth2Config>,
}

impl Client {
    /// Attempt to refresh token if expired
    async fn ensure_valid_token(&self) -> Result<()> {
        if !self.is_token_expired() {
            return Ok(());
        }

        let oauth_config = self.config.oauth_config.as_ref()
            .ok_or(Error::TokenExpired)?;

        let current_token = self.get_access_token();
        let refresh_token = current_token.refresh_token
            .ok_or(Error::TokenExpired)?;

        let handler = OAuth2Handler::new(oauth_config.clone())?;
        let new_token = handler.refresh_token(refresh_token).await?;
        self.set_access_token(new_token);

        Ok(())
    }

    fn build_request(&self, method: Method, url: Url) -> Result<RequestBuilder> {
        // Call ensure_valid_token() before building request
        // Note: Need to make this async or handle differently
    }
}
```

Alternative: Use a `tower` layer for token refresh middleware.

#### Acceptance Criteria
- [x] Expired tokens automatically refresh when refresh_token available
- [x] Refresh failure surfaces as clear error
- [x] Concurrent requests don't trigger multiple refreshes (tokio::sync::Mutex)
- [x] Works without oauth_config (manual mode, returns TokenExpired)
- [x] Tests added in tests/token_refresh_test.rs (5 new tests)

---

### 1.2 Pagination Stream Support ✅ COMPLETE

**Priority**: 🟠 High
**Files**: `src/pagination.rs`, `src/endpoints/*.rs`
**Status**: ✅ Implemented on 2026-01-03
**Dependencies**: `futures`, `async-stream` (optional, via `stream` feature)

#### Current State
```rust
// User must manually paginate
let mut page = 0;
loop {
    let response = client.customers().list(Some(PaginationParams::new().page(page))).await?;
    for customer in response.data {
        process(customer);
    }
    if !response.meta.has_next_page {
        break;
    }
    page += 1;
}
```

#### Proposed Solution

```rust
// src/pagination.rs
use futures::Stream;
use std::pin::Pin;

pub struct PaginatedStream<'a, T, F> {
    fetch: F,
    current_page: u32,
    page_size: u32,
    buffer: Vec<T>,
    exhausted: bool,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a, T, F, Fut> Stream for PaginatedStream<'a, T, F>
where
    F: FnMut(u32, u32) -> Fut,
    Fut: std::future::Future<Output = Result<PaginatedResponse<T>>>,
{
    type Item = Result<T>;
    // ... implementation
}

// Usage in endpoints
impl<'a> CustomersEndpoint<'a> {
    pub fn list_stream(&self) -> impl Stream<Item = Result<Customer>> + 'a {
        self.list_stream_with_page_size(50)
    }

    pub fn list_stream_with_page_size(&self, page_size: u32) -> impl Stream<Item = Result<Customer>> + 'a {
        PaginatedStream::new(page_size, move |page, size| {
            let params = PaginationParams::new().page(page).pagesize(size);
            self.list(Some(params))
        })
    }
}
```

#### Usage
```rust
use futures::StreamExt;

let mut customers = client.customers().list_stream();
while let Some(customer) = customers.next().await {
    let customer = customer?;
    println!("{:?}", customer.name);
}

// Or collect all
let all: Vec<Customer> = client.customers()
    .list_stream()
    .try_collect()
    .await?;
```

#### Acceptance Criteria
- [x] Stream yields items one at a time
- [x] Fetches next page automatically when buffer depleted
- [x] Errors propagate correctly
- [x] Works with customers, articles, invoices endpoints
- [x] Optional feature flag `stream`
- [x] Macro-based implementation to handle lifetimes correctly
- [x] Unit tests in src/pagination.rs (4 new tests)

---

### 1.3 Rate Limiting ✅ COMPLETE

**Priority**: 🟠 High
**Files**: `src/client.rs`, `src/rate_limit.rs`
**Status**: ✅ Implemented on 2026-01-03
**Dependencies**: `governor` crate (optional, via `rate-limit` feature)

#### Implementation

Using `governor` crate with configurable quota and burst:
```rust
// src/rate_limit.rs
use governor::{Quota, RateLimiter};
use std::num::NonZeroU32;

pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub burst_size: u32,
}

pub(crate) struct ApiRateLimiter {
    limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
}

impl ApiRateLimiter {
    pub fn new(config: &RateLimitConfig) -> Self {
        let quota = Quota::per_minute(NonZeroU32::new(config.requests_per_minute).unwrap())
            .allow_burst(NonZeroU32::new(config.burst_size).unwrap());
        Self {
            limiter: Arc::new(RateLimiter::direct(quota)),
        }
    }

    pub async fn acquire(&self) {
        self.limiter.until_ready().await;
    }
}

// In Client::execute_request
#[cfg(feature = "rate-limit")]
if let Some(ref limiter) = self.rate_limiter {
    limiter.acquire().await;
}
```

#### Configuration
```rust
use spiris::{ClientConfig, RateLimitConfig};

let config = ClientConfig::new()
    .rate_limit_config(RateLimitConfig::new(600).burst_size(10));
```

#### Acceptance Criteria
- [x] Requests throttled to configured rate
- [x] Burst handling (allow small bursts via burst_size)
- [x] Optional feature flag `rate-limit` (disabled by default)
- [x] Unit tests in src/rate_limit.rs (5 tests)
- [ ] Per-endpoint rate limiting option (deferred - API uses single global limit)
- [ ] Respects 429 Retry-After header (handled by retry logic instead)

---

### 1.4 Decimal Type for Money ✅ COMPLETE

**Priority**: 🟠 High
**File**: `src/types.rs`
**Status**: ✅ Implemented on 2026-01-03
**Dependencies**: `rust_decimal` crate (optional, via `decimal` feature)

#### Implementation

Introduced a `Money` type alias that switches between `f64` (default) and
`rust_decimal::Decimal` based on the `decimal` feature flag.

```rust
// src/types.rs
#[cfg(feature = "decimal")]
pub type Money = rust_decimal::Decimal;

#[cfg(not(feature = "decimal"))]
pub type Money = f64;

// All monetary fields now use Money:
pub struct InvoiceRow {
    pub unit_price: Option<Money>,
    pub quantity: Option<Money>,
    pub discount_percentage: Option<Money>,
    pub total_amount: Option<Money>,
    // ...
}

// money! macro for portable code:
use spiris::{money, Money};
let price: Money = money!(100.50);
```

#### Configuration

```toml
# Enable decimal support for precise financial calculations
[dependencies]
spiris = { version = "0.1", features = ["decimal"] }
```

#### Acceptance Criteria
- [x] All money fields use Money type alias (46 fields updated)
- [x] Serialization works with both f64 and Decimal (serde-with-float feature)
- [x] Calculations preserve precision when decimal enabled
- [x] Feature flag `decimal` allows opt-in
- [x] `money!` macro for portable test code
- [x] Dedicated decimal tests in tests/decimal_test.rs

---

## Phase 2: Developer Experience

### 2.1 Typed Query/Filter Builder ✅ COMPLETE

**Priority**: 🟡 Medium
**Files**: `src/query.rs`, `src/types.rs`
**Status**: ✅ Implemented on 2026-01-03

#### Implementation

Created a type-safe OData filter builder in `src/query.rs`:

```rust
use spiris::{QueryParams, query::Filter};

// Simple filter
let filter = Filter::field("IsActive").eq(true);

// Combined filters
let filter = Filter::field("IsActive").eq(true)
    .and(Filter::field("Country").eq("SE"));

// String escaping automatic
let filter = Filter::field("Name").eq("O'Brien & Co");
// Produces: Name eq 'O''Brien & Co'

// Numeric comparisons
let filter = Filter::field("TotalAmount").gt(1000);

// String functions
let filter = Filter::field("Name").contains("Corp");
let filter = Filter::field("Code").starts_with("ABC");

// Null checks
let filter = Filter::field("Email").is_null();

// Use with QueryParams
let params = QueryParams::new()
    .filter_by(filter)
    .select("Id,Name");

let customers = client.customers().search(params, None).await?;
```

#### Supported Operations
- **Comparison**: `eq`, `ne`, `gt`, `ge`, `lt`, `le`
- **String**: `contains`, `starts_with`, `ends_with`
- **Null**: `is_null`, `is_not_null`
- **Logic**: `and`, `or`, `not`
- **Raw**: `Filter::raw()` for complex expressions

#### Acceptance Criteria
- [x] Type-safe filter construction
- [x] String escaping handled automatically (single quotes doubled)
- [x] Composable with and/or/not
- [x] Supports common OData operators
- [x] Falls back to raw string via `Filter::raw()`
- [x] Integrated with QueryParams via `filter_by()` method
- [x] 21 unit tests + 7 integration tests

---

### 2.2 Endpoint Macro to Reduce Boilerplate ✅ COMPLETE

**Priority**: 🟡 Medium
**Files**: `src/macros.rs`, `src/endpoints/*.rs`
**Status**: ✅ Implemented on 2026-01-03

#### Implementation

Created a `define_endpoint!` macro with capability flags for flexible endpoint generation:

```rust
use crate::types::Currency;

crate::define_endpoint! {
    /// Currencies endpoint for accessing available currencies.
    CurrenciesEndpoint, "/currencies", Currency,
    caps: [list]
}
```

#### Supported Capabilities

- `list` - List all items with pagination
- `get` - Get a single item by ID
- `create` - Create a new item
- `update` - Update an existing item
- `delete` - Delete an item
- `search` - Search with query parameters
- `stream` - Paginated streaming (requires `stream` feature)

#### Example with Extra Methods

```rust
crate::define_endpoint! {
    BanksEndpoint, "/banks", Bank,
    caps: [list],
    extra: {
        pub async fn list_foreign_payment_codes(
            &self,
        ) -> crate::error::Result<PaginatedResponse<ForeignPaymentCode>> {
            self.client.get("/foreignpaymentcodes").await
        }
    }
}
```

#### Refactored Endpoints

The following 17 endpoints were refactored to use the macro:
- currencies, countries, vat_codes (read-only)
- units, terms_of_payment, bank_accounts (full CRUD)
- delivery_methods, delivery_terms, users (list + get)
- projects (full CRUD + search)
- customer_labels, article_labels, supplier_labels (full CRUD)
- allocation_periods, article_account_codings, documents
- banks, cost_centers (with extra methods)

Complex endpoints with non-standard APIs (accounts, attachments, fiscal_years,
invoices, customers, articles, etc.) were left unchanged as the macro wouldn't
provide significant benefits.

#### Acceptance Criteria
- [x] Macro covers common CRUD operations
- [x] Custom methods easily added via `extra:` block
- [x] Doc comments preserved via outer attributes
- [x] Reduces endpoint code by >50% for refactored files (from ~40 lines to ~9 lines)

---

### 2.3 Separate Create/Update Types ✅ COMPLETE

**Priority**: 🟡 Medium
**File**: `src/types.rs`
**Status**: ✅ Implemented on 2026-01-03

#### Implementation

Added type-safe Create and Update types for the main entities:

```rust
/// CustomerCreate - required fields enforced at compile time
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CustomerCreate {
    pub name: String,  // Required!
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    // ... other optional fields
}

impl CustomerCreate {
    pub fn new(name: impl Into<String>) -> Self { ... }
    pub fn email(mut self, value: impl Into<String>) -> Self { ... }
    // ... builder methods
}

/// CustomerUpdate - all fields optional for partial updates
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CustomerUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    // ...
}
```

#### New Types Added

- `CustomerCreate` / `CustomerUpdate` - Customer creation and updates
- `ArticleCreate` / `ArticleUpdate` - Article/product management
- `InvoiceCreate` / `InvoiceUpdate` - Invoice operations
- `InvoiceRowCreate` - Invoice line items

All types use the builder pattern for ergonomic construction:

```rust
let customer = CustomerCreate::new("Acme Corp")
    .email("contact@acme.com")
    .phone("+46701234567")
    .is_active(true);
```

#### Migration
- Existing types (`Customer`, `Article`, `Invoice`) remain unchanged for backward compatibility
- New types available for stricter type checking
- Endpoints can accept either the existing types or the new typed variants

#### Acceptance Criteria
- [x] Create types have required fields non-optional
- [x] Update types have all fields optional
- [x] Builder pattern for ergonomic construction
- [x] Compile-time enforcement of required fields
- [x] Exported from lib.rs

---

### 2.4 Structured API Error Responses ✅ COMPLETE

**Priority**: 🟡 Medium
**File**: `src/error.rs`, `src/client.rs`
**Status**: ✅ Implemented on 2026-01-03

#### Implementation

Added structured API error handling with the following types:

```rust
/// Structured API error response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ApiErrorResponse {
    pub error_code: Option<String>,
    pub message: String,
    pub validation_errors: Vec<ValidationError>,
}

/// Field-level validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

/// Updated Error enum
#[derive(Error, Debug)]
pub enum Error {
    #[error("API error ({status_code}): {response}")]
    ApiError {
        status_code: u16,
        response: ApiErrorResponse,
        raw_body: String,
    },
    // ...
}
```

#### Helper Methods

Added convenience methods to `Error`:
- `from_api_response(status_code, raw_body)` - Parse error from raw body
- `is_retryable()` - Check if error is retryable
- `status_code()` - Get HTTP status code if applicable
- `validation_errors()` - Get validation errors if present

And to `ApiErrorResponse`:
- `from_raw(message)` - Create from plain text
- `has_validation_errors()` - Check for validation errors
- `validation_error_for(field)` - Get error for specific field

#### Example Usage

```rust
if let Error::ApiError { status_code, response, .. } = err {
    println!("Error {}: {}", status_code, response.message);
    for err in &response.validation_errors {
        println!("  {}: {}", err.field, err.message);
    }
}
```

#### Acceptance Criteria
- [x] API errors parsed into structured type
- [x] Validation errors accessible programmatically
- [x] Raw body preserved for debugging
- [x] Graceful fallback if parsing fails (uses `from_raw`)

---

## Phase 3: Advanced Features

### 3.1 Middleware/Interceptor Pattern ✅ COMPLETE

**Priority**: 🟢 Lower
**Files**: `src/middleware.rs`, `src/client.rs`
**Status**: ✅ Implemented on 2026-01-03

#### Implementation

Created a lightweight middleware pattern without external dependencies (no `tower`):

```rust
use spiris::middleware::{Middleware, RequestContext, ResponseContext};
use spiris::error::Result;

struct LoggingMiddleware;

impl Middleware for LoggingMiddleware {
    fn on_request(&self, ctx: &mut RequestContext) -> Result<()> {
        println!("→ {} {}", ctx.method, ctx.url);
        Ok(())
    }

    fn on_response(&self, ctx: &ResponseContext) {
        println!("← {} {} ({}ms)", ctx.method, ctx.status, ctx.duration.as_millis());
    }
}

// Usage
let config = ClientConfig::new()
    .middleware(LoggingMiddleware);
```

#### Components

- **`Middleware` trait** - Simple sync trait with `on_request` and `on_response` hooks
- **`RequestContext`** - Method, URL, headers (modifiable), extensions for passing data
- **`ResponseContext`** - Method, URL, status, duration, success flag, error info
- **`MiddlewareStack`** - Manages multiple middlewares in order

#### Built-in Middlewares

- **`LoggingMiddleware`** - Logs requests and responses with timing
- **`HeadersMiddleware`** - Adds custom headers to all requests
- **`MetricsMiddleware`** - Collects request metrics (counts, durations, by method/status)

#### Use Cases
- Request/response logging
- Metrics collection (Prometheus, DataDog)
- Custom header injection
- Request signing
- Caching layer

#### Acceptance Criteria
- [x] Simple trait-based middleware pattern
- [x] Request interception with header modification
- [x] Response interception with timing info
- [x] Built-in logging middleware
- [x] Built-in headers middleware
- [x] Built-in metrics middleware
- [x] 15 unit tests

---

### 3.2 Full Tracing Implementation ✅ COMPLETE

**Priority**: 🟢 Lower
**Files**: `src/client.rs`, `src/auth.rs`
**Status**: ✅ Implemented on 2026-01-03
**Dependencies**: `tracing` (optional feature)

#### Implementation

Added comprehensive tracing instrumentation throughout the client using conditional compilation:

```rust
// API Request Tracing (src/client.rs)
#[cfg(feature = "tracing")]
let span = tracing::info_span!("api_request", %method, %url);

#[cfg(feature = "tracing")]
match &result {
    Ok(response) => info!(
        status = response.status().as_u16(),
        duration_ms = elapsed.as_millis() as u64,
        "API request completed"
    ),
    Err(err) => error!(
        error = %err,
        duration_ms = elapsed.as_millis() as u64,
        "API request failed"
    ),
}
```

#### Traced Operations

**Client Operations:**
- API request execution (method, URL, status, duration)
- Token expiration detection
- Token refresh operations
- Retry attempts
- Rate limiter waits

**OAuth2 Operations:**
- Authorization URL generation
- Token exchange (with expiration info)
- Token refresh (with expiration info)
- Error logging for failed operations

#### Usage

```toml
[dependencies]
spiris = { version = "0.1", features = ["tracing"] }
```

```rust
// In your application
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

tracing_subscriber::registry()
    .with(tracing_subscriber::fmt::layer())
    .init();

// Now all API operations will be traced
```

#### Acceptance Criteria
- [x] Tracing disabled by default (feature flag)
- [x] API requests traced with method, URL, status, duration
- [x] Token operations traced (expiration, refresh)
- [x] OAuth2 flows traced (authorize, exchange, refresh)
- [x] Error conditions logged at appropriate levels
- [x] No performance impact when disabled

---

### 3.3 Webhook Support ✅ COMPLETE

**Priority**: 🟢 Lower
**Files**: `src/webhooks.rs`
**Status**: ✅ Implemented on 2026-01-03
**Dependencies**: `hmac`, `sha2`, `hex` (optional via `webhooks` feature)

#### Note on API Support

The Visma eAccounting API currently uses a **polling-based model** and does not natively support webhooks. However, this module provides infrastructure for:
- Future API webhook support
- Third-party integration webhooks
- Custom notification systems

#### Implementation

Created a comprehensive webhook handling module with:

```rust
use spiris::webhooks::{WebhookHandler, WebhookConfig, WebhookEvent};

// Configure webhook handler with signing secret
let config = WebhookConfig::new("your_webhook_secret")
    .signature_header("X-Custom-Signature")
    .timestamp_tolerance_secs(600);

let handler = WebhookHandler::new(config);

// Verify and parse incoming webhook
let event = handler.verify_and_parse(&payload, &signature)?;

match event.event_type.as_str() {
    "invoice.created" => {
        let data: InvoiceData = event.data_as()?;
        // Handle invoice created
    }
    _ => { /* unknown event */ }
}
```

#### Components

- **`WebhookConfig`** - Configuration for signing secret, header name, timestamp tolerance
- **`WebhookEvent`** - Parsed event with id, type, timestamp, resource info, and data payload
- **`WebhookHandler`** - Verifies signatures (HMAC-SHA256) and parses payloads
- **`WebhookEventBuilder`** - Builder pattern for creating test events
- **`WebhookTestHelper`** - Utility for creating signed test payloads
- **`event_types` module** - Constants for common event types

#### Features

- HMAC-SHA256 signature verification
- Event type parsing with category/action helpers
- Typed data payload deserialization
- Test utilities for creating valid signed payloads
- 13 unit tests

#### Usage

```toml
[dependencies]
spiris = { version = "0.1", features = ["webhooks"] }
```

#### Acceptance Criteria
- [x] Webhook payload types defined
- [x] HMAC signature verification
- [x] Event type parsing
- [x] Test utilities for webhook development
- [x] Optional feature flag (disabled by default)

---

## Implementation Order

```
Week 1:
├── 0.1 Fix PKCE (30 min)
├── 0.2 Enable retry logic (1 hr)
└── 1.1 Auto token refresh (4 hr)

Week 2:
├── 1.3 Rate limiting (3 hr)
├── 2.4 Structured errors (2 hr)
└── 1.2 Pagination streams (3 hr)

Week 3:
├── 2.2 Endpoint macro (4 hr)
└── 2.1 Typed query builder (6 hr)

Week 4:
├── 1.4 Decimal for money (4 hr)
└── 2.3 Separate create/update types (6 hr)

Week 5-6:
├── 3.1 Middleware pattern (8 hr)
└── 3.2 Tracing implementation (3 hr)
```

---

## Breaking Changes Summary

For next major version (2.0):

1. `Customer` → `Customer` (response), `CustomerCreate`, `CustomerUpdate`
2. Money fields: `f64` → `Decimal`
3. `Error::ApiError` includes structured response
4. Pagination methods return `Stream` by default
5. `#[non_exhaustive]` on `Error` enum

---

## Feature Flags

```toml
[features]
default = ["rustls-tls"]

# TLS backends
rustls-tls = ["reqwest/rustls-tls"]
native-tls = ["reqwest/native-tls"]

# Optional features
stream = ["futures", "async-stream"]  # Pagination streams
decimal = ["rust_decimal"]             # Decimal money types
tracing = ["dep:tracing"]              # Observability
middleware = ["tower"]                 # Middleware support
```

---

## Success Metrics

After implementation:

- [ ] Zero security vulnerabilities (PKCE fixed)
- [ ] Transient failures auto-recover (retry working)
- [ ] Token refresh transparent to users
- [ ] Rate limits never exceeded
- [ ] 90%+ reduction in endpoint boilerplate
- [ ] Compile-time validation of required fields
- [ ] Full tracing coverage when enabled

---

## References

- [Spiris API Documentation](https://developer.visma.com/api/eaccounting)
- [OAuth2 PKCE RFC 7636](https://tools.ietf.org/html/rfc7636)
- [OData Query Options](https://www.odata.org/documentation/)
- [Tower Middleware](https://docs.rs/tower)
- [rust_decimal](https://docs.rs/rust_decimal)
