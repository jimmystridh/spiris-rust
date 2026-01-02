# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust workspace containing two crates:
- **spiris_bokforing** (root): API client library for Spiris Bokföring och Fakturering (formerly Visma eAccounting)
- **spiris-tui**: Terminal UI application built on the API client

## Build & Test Commands

```bash
# Build everything
cargo build

# Build release
cargo build --release

# Run all tests
cargo test

# Run library tests only
cargo test --lib

# Run a specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Format code
cargo fmt

# Lint
cargo clippy -- -D warnings

# Generate docs
cargo doc --no-deps

# Run TUI
cargo run -p spiris-tui --release

# Run examples (requires SPIRIS_ACCESS_TOKEN env var)
cargo run --example list_customers
```

## Architecture

### API Client Library (src/)

```
src/
├── lib.rs        # Public API: re-exports + Client endpoint accessors
├── client.rs     # HTTP client with auth, retry, rate limit handling
├── auth.rs       # OAuth2 with PKCE: OAuth2Config, OAuth2Handler, AccessToken
├── error.rs      # Error enum (TokenExpired, RateLimitExceeded, NotFound, etc.)
├── retry.rs      # Exponential backoff configuration
├── types.rs      # All API types: Customer, Invoice, Article, Address, pagination
└── endpoints/    # Endpoint modules with CRUD operations
    ├── customers.rs
    ├── invoices.rs
    └── articles.rs
```

**Key patterns:**
- `Client` holds token in `Arc<RwLock<AccessToken>>` for thread-safe updates
- Endpoint access via `client.customers()`, `client.invoices()`, `client.articles()`
- All API types use `#[serde(rename_all = "PascalCase")]` for JSON serialization
- Pagination via `PaginationParams`, filtering via `QueryParams` with OData syntax

### TUI Application (spiris-tui/src/)

```
spiris-tui/src/
├── main.rs       # Terminal setup, event loop
├── app.rs        # Application state machine, all business logic
├── ui.rs         # Ratatui rendering
├── auth.rs       # OAuth callback server (tiny_http)
├── config.rs     # Configuration handling
├── help.rs       # Help screen content
└── screens/      # Screen-specific modules
```

**TUI stack:** ratatui + crossterm + tokio

## API Details

- Base URL: `https://eaccountingapi.vismaonline.com/v2/`
- Rate limit: 600 requests/minute per endpoint
- OAuth2 with PKCE required; tokens expire in 1 hour
- Environment variables: `SPIRIS_CLIENT_ID`, `SPIRIS_CLIENT_SECRET`, `SPIRIS_ACCESS_TOKEN`

## Adding New Endpoints

1. Add types in `src/types.rs` with `#[serde(rename_all = "PascalCase")]`
2. Create `src/endpoints/new_resource.rs` with endpoint struct
3. Export in `src/endpoints/mod.rs`
4. Add accessor method on `Client` in `src/lib.rs`
