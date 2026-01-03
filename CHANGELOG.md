# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-11-16

### Added
- Initial release of Spiris Bokföring och Fakturering API client
- OAuth2 authentication with PKCE support
- Token refresh functionality
- Automatic retry logic with exponential backoff
- Support for Customers endpoint (List, Get, Create, Update, Delete, Search)
- Support for Invoices endpoint (List, Get, Create, Update, Delete, Search)
- Support for Articles endpoint (List, Get, Create, Update, Delete, Search)
- Type-safe request/response models
- Comprehensive error handling
- Rate limiting support (600 req/min)
- Configurable client with builder pattern
- Optional tracing support for observability
- Pagination support
- Query filtering and field selection
- GitHub Actions CI/CD pipeline
- Comprehensive test suite (14 tests)
- Four example applications
- Detailed documentation

### Changed
- Rebranded from Visma eAccounting to Spiris Bokföring och Fakturering
- Package name: `visma_eaccounting` → `spiris`
- Environment variables: `VISMA_*` → `SPIRIS_*`
- User agent: `spiris-bokforing-rust/0.1.0`

### Technical Details
- Built with Rust 2021 edition
- Async/await support via tokio
- HTTP client via reqwest with rustls
- Serde for JSON serialization
- OAuth2 via oauth2 crate
- Comprehensive error types via thiserror

### Notes
- API endpoints remain unchanged from Visma eAccounting
- Full backward compatibility maintained
- Production-ready with automated testing
