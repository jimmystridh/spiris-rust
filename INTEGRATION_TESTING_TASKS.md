# Integration Testing Tasks

Detailed task breakdown for comprehensive integration testing of the Spiris Bokföring API client library improvements.

**Last Updated**: 2026-01-03

---

## Progress Summary

| Section | Status | Tests Added |
|---------|--------|-------------|
| 1. Test Infrastructure | ✅ Complete | 13 tests |
| 2. Phase 0: Critical Fix Tests | ✅ Complete | 44 tests |
| 3. Phase 1: Core Production Tests | ✅ Complete | 146 tests |
| 4. Phase 2: DX Tests | ✅ Complete | 148 tests |
| 5. Phase 3: Advanced Tests | 🔶 Partial | - |
| 6. E2E Tests | ✅ Complete | 32 tests |
| 7. Performance Tests | ✅ Complete | Benchmarks |
| 8. CI/CD | ✅ Complete | Already configured |

**Total Tests: 553** (all passing) + 12 ignored real API tests

### Test Files Summary
- `tests/mock_server/` - Mock server infrastructure
- `tests/auth_pkce_test.rs` - OAuth2 PKCE flow tests
- `tests/retry_test.rs` - Retry logic and exponential backoff tests
- `tests/pagination_stream_test.rs` - Pagination iterator tests
- `tests/rate_limiting_test.rs` - Rate limit handling tests
- `tests/error_handling_test.rs` - Error handling tests
- `tests/query_builder_test.rs` - Query builder pattern tests
- `tests/token_refresh_test.rs` - Token management tests
- `tests/monetary_types_test.rs` - Monetary value (f64) tests
- `tests/endpoint_crud_test.rs` - CRUD operation tests
- `tests/type_validation_test.rs` - Type serialization/traits tests
- `tests/client_config_test.rs` - Client configuration tests
- `tests/contract_test.rs` - API response contract validation
- `tests/real_api_test.rs` - Real API tests (12 tests, ignored by default)
- `benches/client_bench.rs` - Performance benchmarks

### Notes
- **Phase 3 (Advanced)**: Middleware and tracing features are not yet implemented in the library
- **CI/CD**: Already configured with multi-platform testing, MSRV, clippy, coverage, and security audit
- **Real API Tests**: Run with `cargo test --test real_api_test -- --ignored` (requires SPIRIS_ACCESS_TOKEN)
- **Benchmarks**: Run with `cargo bench`

### Critical Fixes Applied (from IMPROVEMENT_PLAN.md)
- ✅ **PKCE Security Fix**: `src/auth.rs:225` - PKCE verifier is now sent during token exchange
- ✅ **Retry Logic Fix**: `src/client.rs:172-205` - Retry logic is now invoked for transient errors
- ✅ **Mock Server Update**: `tests/mock_server.rs:208-211` - MockApi disables retries by default for predictable testing

---

## Table of Contents

1. [Test Infrastructure](#1-test-infrastructure)
2. [Phase 0: Critical Fix Tests](#2-phase-0-critical-fix-tests)
3. [Phase 1: Core Production Readiness Tests](#3-phase-1-core-production-readiness-tests)
4. [Phase 2: Developer Experience Tests](#4-phase-2-developer-experience-tests)
5. [Phase 3: Advanced Feature Tests](#5-phase-3-advanced-feature-tests)
6. [End-to-End Integration Tests](#6-end-to-end-integration-tests)
7. [Performance & Load Tests](#7-performance--load-tests)
8. [CI/CD Integration](#8-cicd-integration)

---

## 1. Test Infrastructure

### 1.1 Enhanced Mock Server ✅ COMPLETE

**File**: `tests/mock_server.rs`

#### 1.1.1 Extend MockApi Capabilities

- [x] **Add OAuth2 mock endpoints** ✅
  - [x] Mock `/connect/authorize` endpoint
  - [x] Mock `/connect/token` endpoint with PKCE validation
  - [x] Mock token refresh endpoint
  - [x] Support for returning expired tokens
  - [x] Support for returning invalid tokens

- [x] **Add rate limiting simulation** ✅
  - [x] Return 429 status after N requests
  - [x] Include `Retry-After` header in responses
  - [x] Reset counter between tests
  - [x] Configurable rate limit threshold

- [x] **Add network failure simulation** ✅
  - [x] Connection timeout simulation
  - [x] Connection refused simulation (simulated via 502)
  - [x] Slow response simulation (configurable delay)

- [x] **Add response sequence support** ✅
  ```rust
  // Example API for sequential responses
  api.mock_sequence("/customers", vec![
      MockResponse::error(500, "Server Error"),
      MockResponse::error(500, "Server Error"),
      MockResponse::ok(customer_json),
  ]);
  ```

- [x] **Add request validation** ✅
  - [x] Validate Authorization header format
  - [x] Validate Content-Type header
  - [x] Validate request body JSON schema
  - [x] Capture and expose request history (RequestRecorder)

#### 1.1.2 Create Test Fixtures ✅ COMPLETE

**Directory**: `tests/fixtures/`

- [x] **Customer fixtures** ✅
  - [x] `customer_simple.json` - Minimal customer
  - [x] `customer_full.json` - All fields populated
  - [x] `customer_list_page1.json` - First page of paginated results
  - [x] `customer_list_page2.json` - Second page
  - [x] `customer_list_empty.json` - Empty results

- [x] **Invoice fixtures** ✅
  - [x] `invoice_simple.json`
  - [x] `invoice_with_rows.json`

- [x] **Article fixtures** ✅
  - [x] `article_simple.json`

- [x] **Error response fixtures** ✅
  - [x] `error_400_validation.json`
  - [x] `error_401_unauthorized.json`
  - [x] `error_404_not_found.json`
  - [x] `error_429_rate_limit.json`
  - [x] `error_500_server.json`

- [x] **OAuth fixtures** ✅
  - [x] `token_response_valid.json`
  - [x] `token_response_with_refresh.json`
  - [x] `token_error_invalid_grant.json`

#### 1.1.3 Test Utilities Module ✅ COMPLETE

**File**: `tests/test_utils.rs`

- [x] **Assertion helpers** ✅
  ```rust
  fn assert_customer_eq(actual: &Customer, expected: &Customer);
  fn assert_pagination_meta(meta: &ResponseMetadata, page: u32, total: u32);
  fn assert_error_status(error: &Error, expected_status: u16);
  ```

- [x] **Test data generators** ✅
  ```rust
  fn random_customer() -> Customer;
  fn random_invoice(customer_id: &str) -> Invoice;
  fn expired_token() -> AccessToken;
  fn valid_token() -> AccessToken;
  ```

- [x] **Async test timeout wrapper** ✅
  ```rust
  async fn with_timeout<F, T>(duration: Duration, f: F) -> T
  where F: Future<Output = T>;
  ```

---

## 2. Phase 0: Critical Fix Tests

### 2.1 PKCE Verification Tests

**File**: `tests/auth_pkce_test.rs` ✅ COMPLETE (20 tests)

#### 2.1.1 PKCE Flow Validation

- [x] **Test PKCE challenge generation** ✅
  - [x] Verify verifier meets length requirements (43-128 chars) - `test_pkce_verifier_generation`
  - [x] Verify each call generates unique challenge/verifier pair - `test_pkce_verifier_uniqueness`

- [x] **Test authorize URL contains PKCE** ✅
  - [x] URL includes `code_challenge` parameter - `test_authorize_url_contains_pkce_challenge`
  - [x] URL includes `code_challenge_method=S256` - `test_authorize_url_contains_pkce_challenge`
  - [x] URL includes required OAuth2 params - `test_authorize_url_contains_required_params`
  - [x] URL includes scopes (ea:api, offline_access) - `test_authorize_url_contains_scopes`

- [x] **Test token exchange includes verifier** ✅ (PKCE fix implemented in auth.rs)
  - [x] PKCE verifier is passed to token exchange (fixed in src/auth.rs:225)
  - [x] Mock server can validate verifier presence

#### 2.1.2 CSRF Token Tests ✅

- [x] **Test CSRF token uniqueness** ✅
  - [x] CSRF token generation - `test_csrf_token_generation`
  - [x] Each authorize_url call generates unique CSRF - `test_csrf_token_uniqueness`
  - [x] CSRF token in URL - `test_csrf_token_in_url`

#### 2.1.3 OAuth2 Config Tests ✅

- [x] **Test OAuth2Config** ✅
  - [x] Default URLs - `test_oauth2_config_default_urls`
  - [x] Custom config - `test_oauth2_config_new`
  - [x] Handler creation - `test_oauth2_handler_creation`
  - [x] Invalid auth URL rejection - `test_oauth2_handler_invalid_auth_url`
  - [x] Invalid redirect URI rejection - `test_oauth2_handler_invalid_redirect_uri`

#### 2.1.4 Access Token Tests ✅

- [x] **Access Token module** ✅ (8 tests)
  - [x] Token creation - `test_access_token_creation`
  - [x] Token with refresh - `test_access_token_with_refresh`
  - [x] Token not expired - `test_access_token_not_expired`
  - [x] Token expired - `test_access_token_expired`
  - [x] Token expires with 5 min buffer - `test_access_token_expires_with_buffer`
  - [x] Authorization header format - `test_access_token_authorization_header`
  - [x] Serialization roundtrip - `test_access_token_serialization`

### 2.2 Retry Logic Tests ✅ COMPLETE

**File**: `tests/retry_test.rs` ✅ COMPLETE (24 tests)

#### 2.2.1 Retry Configuration ✅

- [x] **RetryConfig tests** ✅
  - [x] Default values - `test_retry_config_defaults`
  - [x] Builder pattern - `test_retry_config_builder`

#### 2.2.2 is_retryable_error Tests ✅

- [x] **Error classification** ✅ (7 tests)
  - [x] Rate limit is retryable - `test_rate_limit_is_retryable`
  - [x] 500 is retryable - `test_server_error_500_is_retryable`
  - [x] 502 is retryable - `test_server_error_502_is_retryable`
  - [x] 503 is retryable - `test_server_error_503_is_retryable`
  - [x] 400 NOT retryable - `test_client_error_400_not_retryable`
  - [x] 401 NOT retryable - `test_client_error_401_not_retryable`
  - [x] 404 NOT retryable - `test_client_error_404_not_retryable`
  - [x] TokenExpired NOT retryable - `test_token_expired_not_retryable`

#### 2.2.3 retry_request Function Tests ✅

- [x] **Retry function behavior** ✅ (5 tests)
  - [x] Succeeds on first try - `test_retry_function_succeeds_first_try`
  - [x] Succeeds after retries - `test_retry_function_succeeds_after_retries`
  - [x] Fails after max retries - `test_retry_function_fails_after_max_retries`
  - [x] No retry on 4xx - `test_retry_function_no_retry_on_4xx`
  - [x] No retry on TokenExpired - `test_retry_function_no_retry_on_token_expired`

#### 2.2.4 Mock Server Integration ✅

- [x] **Mock server retry testing** ✅ (3 tests)
  - [x] Sequence setup works - `test_mock_server_sequence_setup`
  - [x] Rate limit response - `test_mock_server_rate_limit_response`
  - [x] Connection reset simulation - `test_mock_server_connection_reset`

#### 2.2.5 Exponential Backoff ✅

- [x] **Backoff calculation** ✅ (2 tests)
  - [x] Backoff multiplier - `test_backoff_calculation`
  - [x] Max interval cap - `test_backoff_capped_at_max`

#### 2.2.6 Client Integration ✅

- [x] **Test retry in client** ✅ (Retry logic integrated in client.rs)
  - [x] Client retries on 500/502/503 errors (fixed in src/client.rs:172-205)
  - [x] MockApi disables retries by default for predictable tests
  - [x] MockApi::with_retry_config() enables custom retry testing

### 2.3 Critical Fixes ✅ COMPLETE

Both critical fixes from IMPROVEMENT_PLAN.md have been implemented:

1. ✅ **PKCE Fix**: PKCE verifier is now passed during token exchange (auth.rs:225)
2. ✅ **Retry Logic**: Retry logic is now invoked from client.rs execute_request (client.rs:172-205)

---

## 3. Phase 1: Core Production Readiness Tests

### 3.1 Automatic Token Refresh Tests ✅ PARTIAL

**File**: `tests/token_refresh_test.rs` ✅ (27 tests)

#### 3.1.0 Token Tests Implemented ✅

- [x] **AccessToken Creation and Properties** ✅
  - test_access_token_creation
  - test_access_token_with_refresh_token
  - test_access_token_authorization_header

- [x] **Token Expiration** ✅
  - test_token_not_expired_with_long_expiry
  - test_token_expired_with_zero_expiry
  - test_token_expired_with_negative_expiry
  - test_token_expires_within_buffer
  - test_token_not_expired_outside_buffer
  - test_token_exactly_at_buffer_boundary

- [x] **Token Serialization** ✅
  - test_token_serialization
  - test_token_deserialization
  - test_token_roundtrip_preserves_data

- [x] **Client Token Handling** ✅
  - test_client_rejects_expired_token
  - test_client_accepts_valid_token
  - test_set_access_token_updates_client
  - test_concurrent_requests_with_valid_token

- [x] **OAuth2 Handler** ✅
  - test_oauth2_handler_creation_succeeds
  - test_oauth2_handler_authorize_url_generated

#### 3.1.1 Automatic Refresh Behavior

- [ ] **Test refresh on expired token**
  ```rust
  #[tokio::test]
  async fn test_auto_refresh_expired_token() {
      let mut api = MockApi::new().await;
      let mut oauth = MockOAuthServer::new().await;

      // Start with expired token
      let expired_token = AccessToken::new("old_token".into(), -100, Some("refresh_token".into()));

      // Expect refresh call, return new token
      oauth.expect_refresh("refresh_token", "new_token", 3600);

      // Expect API call with new token
      api.mock_get_with_auth("/customers", "Bearer new_token", customer_list());

      let client = Client::with_auto_refresh(expired_token, oauth.config());
      let result = client.customers().list(None).await;

      assert!(result.is_ok());
      oauth.assert_refresh_called();
  }
  ```

- [ ] **Test refresh failure handling**
  - [ ] Invalid refresh token returns clear error
  - [ ] Revoked refresh token returns clear error
  - [ ] Network error during refresh is retried
  - [ ] Error includes original cause

- [ ] **Test no refresh if token valid**
  - [ ] Valid token doesn't trigger refresh
  - [ ] Token with >5 min remaining doesn't refresh

- [ ] **Test refresh token rotation**
  - [ ] New refresh token from server is stored
  - [ ] Subsequent refresh uses new token

#### 3.1.2 Concurrent Request Handling

- [ ] **Test single refresh for concurrent requests**
  ```rust
  #[tokio::test]
  async fn test_concurrent_requests_single_refresh() {
      // When 10 requests hit with expired token
      // Only 1 refresh should occur
      // All 10 should succeed with new token
  }
  ```

- [ ] **Test requests wait for refresh**
  - [ ] Requests queue while refresh in progress
  - [ ] All queued requests use refreshed token

#### 3.1.3 Manual Token Management

- [ ] **Test manual mode (no oauth config)**
  - [ ] Expired token returns TokenExpired error
  - [ ] User can call set_access_token manually
  - [ ] Subsequent requests use new token

### 3.2 Pagination Stream Tests ✅ COMPLETE

**File**: `tests/pagination_stream_test.rs` ✅ (19 tests)

#### 3.2.1 Basic Stream Functionality

- [x] **Test stream yields all items** ✅
  ```rust
  #[tokio::test]
  async fn test_stream_all_pages() {
      let mut api = MockApi::new().await;

      // 3 pages of 2 items each
      api.mock_paginated_responses("/customers", vec![
          (vec![customer(1), customer(2)], true),  // has_next
          (vec![customer(3), customer(4)], true),
          (vec![customer(5), customer(6)], false), // last page
      ]);

      let stream = api.client.customers().list_stream();
      let customers: Vec<Customer> = stream.try_collect().await.unwrap();

      assert_eq!(customers.len(), 6);
      api.assert_request_count("/customers", 3);
  }
  ```

- [x] **Test stream with empty results** ✅
  - test_list_customers_empty_result

- [x] **Test stream with single page** ✅
  - test_pagination_single_item
  - All items returned

#### 3.2.2 Stream Error Handling

- [x] **Test error propagation** ✅
  ```rust
  #[tokio::test]
  async fn test_stream_error_mid_pagination() {
      let mut api = MockApi::new().await;

      api.mock_sequence("/customers", vec![
          MockResponse::ok(page1_json),
          MockResponse::error(500, "Server Error"),
      ]);

      let stream = api.client.customers().list_stream();
      let results: Vec<Result<Customer, _>> = stream.collect().await;

      // First 2 items succeed, then error
      assert!(results[0].is_ok());
      assert!(results[1].is_ok());
      assert!(matches!(results[2], Err(Error::ApiError { .. })));
  }
  ```

- [ ] **Test stream recovery not attempted**
  - [ ] After error, stream terminates
  - [ ] No retry within stream (user's responsibility)

#### 3.2.3 Stream Configuration

- [ ] **Test custom page size**
  ```rust
  #[tokio::test]
  async fn test_stream_custom_page_size() {
      let mut api = MockApi::new().await;

      // Expect pagesize=100 in requests
      api.expect_query_param("/customers", "pagesize", "100");

      let stream = api.client.customers().list_stream_with_page_size(100);
      // ...
  }
  ```

- [ ] **Test stream with filters**
  - [ ] Filter applied to all page requests
  - [ ] Sort order preserved across pages

#### 3.2.4 Stream Memory Efficiency

- [ ] **Test items not buffered unnecessarily**
  - [ ] Only current page in memory
  - [ ] Previous page items dropped after yield

- [ ] **Test lazy fetching**
  - [ ] Next page only fetched when needed
  - [ ] Dropping stream doesn't fetch remaining pages

### 3.3 Rate Limiting Tests ✅ COMPLETE

**File**: `tests/rate_limiting_test.rs` ✅ (22 tests)

#### 3.3.1 Rate Limiter Behavior

- [x] **Test rate limit detection** ✅
  - test_rate_limit_returns_429_error
  - test_rate_limit_on_customer_create
  - test_rate_limit_on_invoice_list
  - test_rate_limit_on_article_get

- [ ] **Test requests throttled at limit** (Pending automatic rate limit handling)
  ```rust
  #[tokio::test]
  async fn test_rate_limit_enforced() {
      let client = Client::new(token)
          .with_config(ClientConfig::new().rate_limit(10)); // 10/min for testing

      let start = Instant::now();

      // Fire 15 requests
      let futures: Vec<_> = (0..15).map(|_| client.customers().list(None)).collect();
      join_all(futures).await;

      // Should take at least 30 seconds (10 immediate, 5 throttled)
      assert!(start.elapsed() >= Duration::from_secs(30));
  }
  ```

- [ ] **Test burst allowance**
  - [ ] Initial burst allowed
  - [ ] Sustained rate enforced after burst

- [ ] **Test rate limit disabled**
  - [ ] `rate_limit(0)` disables limiting
  - [ ] All requests fire immediately

#### 3.3.2 429 Response Handling

- [x] **Test Retry-After header present** ✅
  - test_rate_limit_with_retry_after_header
  - test_rate_limit_short_retry_after
  - test_rate_limit_long_retry_after

- [ ] **Test Retry-After header respected** (Pending automatic handling)
  ```rust
  #[tokio::test]
  async fn test_retry_after_header() {
      let mut api = MockApi::new().await;

      api.mock_rate_limit("/customers", 5, Duration::from_secs(2));
      // Returns 429 with Retry-After: 2 after 5 requests

      let start = Instant::now();
      for _ in 0..10 {
          let _ = api.client.customers().list(None).await;
      }

      // Should have waited ~2 seconds for retry
      assert!(start.elapsed() >= Duration::from_secs(2));
  }
  ```

- [ ] **Test without Retry-After header**
  - [ ] Uses exponential backoff
  - [ ] Eventually succeeds

#### 3.3.3 Per-Endpoint Rate Limiting

- [x] **Test independent endpoint limits** ✅
  - test_rate_limit_independent_per_endpoint
  - test_multiple_requests_during_rate_limit

- [x] **Error recovery tests** ✅
  - test_recovery_after_rate_limit_sequence
  - test_multiple_rate_limits_then_success

- [x] **Edge cases** ✅
  - test_rate_limit_on_empty_response
  - test_rate_limit_with_malformed_json
  - test_rate_limit_zero_retry_after

### 3.4 Decimal Money Type Tests ✅ COMPLETE

**File**: `tests/monetary_types_test.rs` ✅ (28 tests)

#### 3.4.0 Current f64 Behavior Tests ✅

- [x] **Basic f64 tests** ✅
  - test_invoice_total_amount_f64
  - test_invoice_row_unit_price_f64
  - test_article_sales_price_f64

- [x] **f64 Precision Documentation** ✅
  - test_f64_precision_issue_demonstration
  - test_f64_multiplication_precision
  - test_f64_large_amount
  - test_f64_small_amount
  - test_f64_zero_amount
  - test_f64_negative_amount

- [x] **JSON Serialization** ✅
  - test_invoice_amount_serialization
  - test_invoice_amount_deserialization
  - test_amount_deserialization_from_integer
  - test_amount_deserialization_null
  - test_amount_deserialization_missing

- [x] **Integration Tests** ✅
  - test_invoice_with_amount_from_api
  - test_article_with_prices_from_api
  - test_invoice_list_with_amounts

- [x] **Calculation Tests** ✅
  - test_calculate_line_total
  - test_calculate_with_discount
  - test_sum_invoice_rows

- [x] **Edge Cases** ✅
  - test_amount_with_many_decimal_places
  - test_amount_scientific_notation
  - test_very_large_amount
  - test_very_small_positive_amount

#### 3.4.1 Serialization/Deserialization (Future Decimal)

- [ ] **Test decimal serialization** (Pending rust_decimal implementation)
  ```rust
  #[test]
  fn test_invoice_row_decimal_serialization() {
      let row = InvoiceRow {
          unit_price: Some(Decimal::new(9999, 2)), // 99.99
          quantity: Some(Decimal::new(25, 1)),     // 2.5
          ..Default::default()
      };

      let json = serde_json::to_string(&row).unwrap();

      // Verify format matches API expectation
      assert!(json.contains("\"UnitPrice\":\"99.99\"") ||
              json.contains("\"UnitPrice\":99.99"));
  }
  ```

- [ ] **Test decimal deserialization**
  - [ ] Parse from string: `"99.99"`
  - [ ] Parse from number: `99.99`
  - [ ] Parse from integer: `100`

- [ ] **Test precision preservation**
  ```rust
  #[test]
  fn test_decimal_precision() {
      let price = Decimal::from_str("19.99").unwrap();
      let qty = Decimal::from_str("3").unwrap();
      let total = price * qty;

      assert_eq!(total, Decimal::from_str("59.97").unwrap());
      // NOT 59.970000000000006 like f64!
  }
  ```

#### 3.4.2 Edge Cases

- [ ] **Test very large amounts**
  - [ ] Millions of currency units
  - [ ] No overflow

- [ ] **Test very small amounts**
  - [ ] Fractions of cents
  - [ ] Precision maintained

- [ ] **Test zero and negative**
  - [ ] Zero serializes correctly
  - [ ] Negative amounts (credits) work

---

## 4. Phase 2: Developer Experience Tests

### 4.1 Typed Query Builder Tests ✅ COMPLETE

**File**: `tests/query_builder_test.rs` ✅ (55 tests)

#### 4.1.1 Filter Expression Building

- [x] **Test equality filters** ✅
  - test_filter_equality
  - test_filter_not_equal
  ```rust
  #[test]
  fn test_eq_filter() {
      let filter = Filter::field("IsActive").eq(true);
      assert_eq!(filter.to_string(), "IsActive eq true");

      let filter = Filter::field("Name").eq("Acme");
      assert_eq!(filter.to_string(), "Name eq 'Acme'");
  }
  ```

- [x] **Test comparison filters** ✅
  - test_filter_greater_than
  - test_filter_less_than
  - test_filter_greater_or_equal
  - test_filter_less_or_equal

- [x] **Test string functions** ✅
  - test_filter_contains
  - test_filter_startswith
  - test_filter_endswith

- [x] **Test null checks** ✅
  - test_filter_null

#### 4.1.2 Composite Filters

- [x] **Test AND composition** ✅
  - test_filter_and
  ```rust
  #[test]
  fn test_and_filter() {
      let filter = Filter::field("IsActive").eq(true)
          .and(Filter::field("Name").contains("Corp"));

      assert_eq!(filter.to_string(), "(IsActive eq true) and (contains(Name,'Corp'))");
  }
  ```

- [x] **Test OR composition** ✅
  - test_filter_or

- [x] **Test complex nested filters** ✅
  - test_filter_complex
  - test_filter_boolean
  - test_filter_date
  ```rust
  #[test]
  fn test_complex_filter() {
      let filter = Filter::field("IsActive").eq(true)
          .and(
              Filter::field("Type").eq("A")
                  .or(Filter::field("Type").eq("B"))
          );

      // Proper parenthesization
      assert_eq!(
          filter.to_string(),
          "(IsActive eq true) and ((Type eq 'A') or (Type eq 'B'))"
      );
  }
  ```

#### 4.1.3 String Escaping

- [x] **Test quote escaping** ✅
  - test_filter_with_special_characters

- [x] **Test special characters** ✅
  - test_filter_with_unicode
  - test_empty_filter_string

#### 4.1.4 Integration with API

- [x] **Test filter in actual request** ✅
  - test_customer_list_with_filter
  - test_customer_list_with_pagination_params
  - test_article_list_pagination
  - test_invoice_list_pagination

#### 4.1.5 QueryParams & PaginationParams Builder Tests ✅

- [x] **QueryParams tests** ✅
  - test_query_params_default
  - test_query_params_filter
  - test_query_params_select
  - test_query_params_custom_param
  - test_query_params_chaining
  - test_query_params_filter_override
  - test_query_params_multiple_custom_params
  - test_query_params_clone
  - test_query_params_debug
  - test_query_params_serialize_filter
  - test_query_params_serialize_custom

- [x] **PaginationParams tests** ✅
  - test_pagination_params_default
  - test_pagination_params_page
  - test_pagination_params_pagesize
  - test_pagination_params_both
  - test_pagination_params_chaining_override
  - test_pagination_zero_page
  - test_pagination_large_pagesize
  - test_pagination_pagesize_one
  - test_pagination_params_clone
  - test_pagination_params_debug
  - test_pagination_params_serialize

- [ ] **Test filter in actual request** (Future typed builder)
  ```rust
  #[tokio::test]
  async fn test_typed_filter_in_request() {
      let mut api = MockApi::new().await;

      api.expect_query_param(
          "/customers",
          "filter",
          "IsActive eq true"
      );
      api.mock_get("/customers", customer_list());

      let filter = Filter::field("IsActive").eq(true);
      api.client.customers()
          .search(QueryParams::new().filter_expr(filter), None)
          .await
          .unwrap();

      api.assert_expectations();
  }
  ```

### 4.2 Endpoint Macro Tests

**File**: `tests/endpoint_macro_test.rs`

#### 4.2.1 Generated CRUD Operations

- [ ] **Test list operation generated**
  - [ ] Correct URL called
  - [ ] Pagination params passed
  - [ ] Response deserialized

- [ ] **Test get operation generated**
  - [ ] ID interpolated in URL
  - [ ] Single item returned

- [ ] **Test create operation generated**
  - [ ] POST method used
  - [ ] Body serialized correctly
  - [ ] Response contains ID

- [ ] **Test update operation generated**
  - [ ] PUT method used
  - [ ] ID in URL
  - [ ] Body serialized

- [ ] **Test delete operation generated**
  - [ ] DELETE method used
  - [ ] Returns Ok(()) on success

#### 4.2.2 Custom Method Extension

- [ ] **Test custom methods work alongside generated**
  ```rust
  #[tokio::test]
  async fn test_invoice_custom_methods() {
      let mut api = MockApi::new().await;

      // Standard CRUD still works
      api.mock_get("/customerinvoices", invoice_list());
      api.client.invoices().list(None).await.unwrap();

      // Custom method also works
      api.mock_get_bytes("/customerinvoices/123/pdf", pdf_bytes());
      let pdf = api.client.invoices().get_pdf("123").await.unwrap();
      assert!(!pdf.is_empty());
  }
  ```

### 4.3 Separate Type Tests

**File**: `tests/separate_types_test.rs`

#### 4.3.1 Create Type Validation

- [ ] **Test required fields enforced at compile time**
  ```rust
  #[test]
  fn test_customer_create_required_fields() {
      // This should compile
      let valid = CustomerCreate {
          name: "Acme Corp".into(),
          ..Default::default()
      };

      // This should NOT compile (uncomment to verify)
      // let invalid = CustomerCreate {
      //     email: Some("test@test.com".into()),
      //     // name missing!
      // };
  }
  ```

- [ ] **Test optional fields remain optional**
  ```rust
  #[test]
  fn test_create_optional_fields() {
      let minimal = CustomerCreate {
          name: "Acme".into(),
          ..Default::default()
      };

      let full = CustomerCreate {
          name: "Acme".into(),
          email: Some("test@acme.com".into()),
          phone: Some("+1234567890".into()),
          ..Default::default()
      };

      // Both valid
  }
  ```

#### 4.3.2 Update Type Flexibility

- [ ] **Test partial updates**
  ```rust
  #[tokio::test]
  async fn test_partial_update() {
      let mut api = MockApi::new().await;

      // Only email in request body
      api.expect_request_body("/customers/123", |body: &str| {
          let json: serde_json::Value = serde_json::from_str(body).unwrap();
          json.get("Name").is_none() && json.get("Email").is_some()
      });

      let update = CustomerUpdate {
          email: Some("new@email.com".into()),
          ..Default::default()
      };

      api.client.customers().update("123", &update).await.unwrap();
  }
  ```

#### 4.3.3 Response Type Guarantees

- [ ] **Test non-optional fields in response**
  ```rust
  #[tokio::test]
  async fn test_response_has_id() {
      let mut api = MockApi::new().await;
      api.mock_get("/customers/123", r#"{"Id": "123", "Name": "Test"}"#);

      let customer: Customer = api.client.customers().get("123").await.unwrap();

      // id is String, not Option<String>
      let id: String = customer.id;
      assert_eq!(id, "123");
  }
  ```

### 4.4 Structured Error Tests ✅ COMPLETE

**File**: `tests/error_handling_test.rs` ✅ (38 tests)

#### 4.4.0 Error Handling Tests Added ✅

- [x] **HTTP Status Code Tests** ✅
  - test_400_bad_request (test_bad_request_error)
  - test_401_unauthorized (test_unauthorized_error)
  - test_403_forbidden (test_forbidden_error)
  - test_404_not_found (test_not_found_error)
  - test_409_conflict (test_update_conflict_error)
  - test_422_unprocessable_entity
  - test_500_internal_server_error (test_server_error)
  - test_502_bad_gateway
  - test_503_service_unavailable (test_service_unavailable_error)
  - test_504_gateway_timeout

- [x] **Error Retryability Tests** ✅
  - test_rate_limit_error_is_retryable
  - test_5xx_errors_are_retryable
  - test_4xx_errors_not_retryable
  - test_token_expired_not_retryable
  - test_not_found_not_retryable

- [x] **Error Display Tests** ✅
  - test_api_error_display
  - test_not_found_display
  - test_rate_limit_display
  - test_token_expired_display

- [x] **Error Trait Tests** ✅
  - test_error_implements_std_error
  - test_error_is_send_sync

- [x] **Error Response Format Tests** ✅
  - test_empty_response_body_error
  - test_error_with_plain_text_body
  - test_error_with_html_body
  - test_error_with_malformed_json
  - test_validation_error_details

- [x] **Endpoint-Specific Error Tests** ✅
  - test_invoice_not_found
  - test_article_not_found
  - test_delete_not_found_error

- [x] **Error Recovery Tests** ✅
  - test_success_after_error
  - test_different_errors_in_sequence
  - test_concurrent_errors

#### 4.4.1 Error Parsing

- [x] **Test validation error parsing** ✅
  ```rust
  #[tokio::test]
  async fn test_validation_error_parsed() {
      let mut api = MockApi::new().await;

      api.mock_error("/customers", 400, r#"{
          "ErrorCode": "VALIDATION_ERROR",
          "Message": "Validation failed",
          "ValidationErrors": [
              {"Field": "Name", "Message": "Name is required"},
              {"Field": "Email", "Message": "Invalid email format"}
          ]
      }"#);

      let result = api.client.customers().create(&customer).await;

      match result {
          Err(Error::ApiError { response, .. }) => {
              assert_eq!(response.validation_errors.len(), 2);
              assert_eq!(response.validation_errors[0].field, "Name");
          }
          _ => panic!("Expected ApiError"),
      }
  }
  ```

- [ ] **Test fallback for non-JSON errors**
  ```rust
  #[tokio::test]
  async fn test_non_json_error() {
      let mut api = MockApi::new().await;
      api.mock_error("/customers", 500, "Internal Server Error");

      let result = api.client.customers().list(None).await;

      match result {
          Err(Error::ApiError { response, raw_body, .. }) => {
              assert_eq!(response.message, "Internal Server Error");
              assert_eq!(raw_body, "Internal Server Error");
          }
          _ => panic!("Expected ApiError"),
      }
  }
  ```

#### 4.4.2 Error Display

- [ ] **Test error formatting**
  ```rust
  #[test]
  fn test_error_display() {
      let error = Error::ApiError {
          status_code: 400,
          response: ApiErrorResponse {
              error_code: Some("VALIDATION_ERROR".into()),
              message: "Name is required".into(),
              validation_errors: vec![],
          },
          raw_body: "...".into(),
      };

      let display = format!("{}", error);
      assert!(display.contains("400"));
      assert!(display.contains("Name is required"));
  }
  ```

---

## 5. Phase 3: Advanced Feature Tests

### 5.1 Middleware Tests

**File**: `tests/middleware_test.rs`

#### 5.1.1 Logging Middleware

- [ ] **Test request logging**
  ```rust
  #[tokio::test]
  async fn test_logging_middleware() {
      let log_capture = LogCapture::new();

      let client = Client::new(token)
          .with_middleware(LoggingMiddleware::new());

      client.customers().list(None).await.unwrap();

      let logs = log_capture.entries();
      assert!(logs.iter().any(|l| l.contains("GET /customers")));
      assert!(logs.iter().any(|l| l.contains("200 OK")));
  }
  ```

#### 5.1.2 Custom Header Middleware

- [ ] **Test header injection**
  ```rust
  #[tokio::test]
  async fn test_custom_header_middleware() {
      let mut api = MockApi::new().await;

      api.expect_header("X-Request-Id", "test-123");

      let client = api.client_with_middleware(
          HeaderMiddleware::new("X-Request-Id", "test-123")
      );

      client.customers().list(None).await.unwrap();
      api.assert_expectations();
  }
  ```

#### 5.1.3 Middleware Chain

- [ ] **Test multiple middleware**
  - [ ] Middlewares execute in order
  - [ ] Each can modify request/response
  - [ ] Error in one stops chain

### 5.2 Tracing Tests

**File**: `tests/tracing_test.rs`

#### 5.2.1 Span Creation

- [ ] **Test spans created for requests**
  ```rust
  #[tokio::test]
  async fn test_request_spans() {
      let subscriber = TestSubscriber::new();
      let _guard = tracing::subscriber::set_default(subscriber.clone());

      let client = Client::new(token)
          .with_config(ClientConfig::new().enable_tracing(true));

      client.customers().get("123").await.ok();

      let spans = subscriber.spans();
      assert!(spans.iter().any(|s| s.name == "http_request"));
      assert!(spans.iter().any(|s| s.fields.contains_key("url")));
  }
  ```

#### 5.2.2 Error Recording

- [ ] **Test errors recorded in spans**
  - [ ] Error events attached to span
  - [ ] Error details included

---

## 6. End-to-End Integration Tests

### 6.1 Real API Tests (Optional)

**File**: `tests/real_api_test.rs`
**Requires**: `SPIRIS_ACCESS_TOKEN` environment variable

#### 6.1.1 Smoke Tests

- [ ] **Test authentication**
  ```rust
  #[tokio::test]
  #[ignore] // Run with: cargo test -- --ignored
  async fn test_real_api_auth() {
      let token = std::env::var("SPIRIS_ACCESS_TOKEN").unwrap();
      let client = Client::new(AccessToken::new(token, 3600, None));

      // Simple call to verify auth works
      let result = client.company_settings().get().await;
      assert!(result.is_ok());
  }
  ```

- [ ] **Test customer CRUD cycle**
  ```rust
  #[tokio::test]
  #[ignore]
  async fn test_real_customer_lifecycle() {
      let client = real_client();

      // Create
      let created = client.customers().create(&test_customer()).await.unwrap();
      let id = created.id.clone();

      // Read
      let fetched = client.customers().get(&id).await.unwrap();
      assert_eq!(fetched.name, created.name);

      // Update
      let mut updated = fetched;
      updated.email = Some("updated@test.com".into());
      client.customers().update(&id, &updated).await.unwrap();

      // Delete
      client.customers().delete(&id).await.unwrap();

      // Verify deleted
      let result = client.customers().get(&id).await;
      assert!(matches!(result, Err(Error::NotFound(_))));
  }
  ```

#### 6.1.2 Rate Limit Verification

- [ ] **Test real rate limit behavior**
  ```rust
  #[tokio::test]
  #[ignore]
  async fn test_real_rate_limits() {
      // Make rapid requests, verify client handles 429
  }
  ```

### 6.2 Contract Tests

**File**: `tests/contract_test.rs`

#### 6.2.1 Response Schema Validation

- [ ] **Test customer response matches schema**
  - [ ] All expected fields present
  - [ ] Field types correct
  - [ ] Enum values valid

- [ ] **Test pagination metadata**
  - [ ] Meta object structure
  - [ ] Page numbers make sense

---

## 7. Performance & Load Tests

### 7.1 Benchmark Tests

**File**: `benches/client_bench.rs`

#### 7.1.1 Request Throughput

- [ ] **Benchmark single request latency**
  ```rust
  fn bench_single_request(c: &mut Criterion) {
      let rt = Runtime::new().unwrap();
      let api = rt.block_on(MockApi::new());

      c.bench_function("customer_get", |b| {
          b.to_async(&rt).iter(|| {
              api.client.customers().get("123")
          });
      });
  }
  ```

- [ ] **Benchmark concurrent requests**
  - [ ] 10 concurrent
  - [ ] 100 concurrent
  - [ ] Measure p50, p95, p99

#### 7.1.2 Serialization Performance

- [ ] **Benchmark JSON parsing**
  - [ ] Small response (1 item)
  - [ ] Large response (500 items)
  - [ ] Complex nested structures

### 7.2 Memory Tests

- [ ] **Test no memory leaks in pagination**
  - [ ] Stream 10,000 items
  - [ ] Memory usage stays bounded

- [ ] **Test connection pool cleanup**
  - [ ] Create/drop many clients
  - [ ] No file descriptor leaks

---

## 8. CI/CD Integration

### 8.1 GitHub Actions Workflow

**File**: `.github/workflows/test.yml`

#### 8.1.1 Test Matrix

- [ ] **Multi-platform testing**
  ```yaml
  strategy:
    matrix:
      os: [ubuntu-latest, macos-latest, windows-latest]
      rust: [stable, beta, 1.83.0]  # MSRV
  ```

- [ ] **Feature flag combinations**
  ```yaml
  - name: Test default features
    run: cargo test

  - name: Test all features
    run: cargo test --all-features

  - name: Test no default features
    run: cargo test --no-default-features

  - name: Test with stream feature
    run: cargo test --features stream

  - name: Test with decimal feature
    run: cargo test --features decimal
  ```

#### 8.1.2 Coverage Reporting

- [ ] **Setup codecov/coveralls**
  ```yaml
  - name: Generate coverage
    run: cargo tarpaulin --out xml

  - name: Upload coverage
    uses: codecov/codecov-action@v3
  ```

#### 8.1.3 Real API Tests in CI

- [ ] **Scheduled real API tests**
  ```yaml
  on:
    schedule:
      - cron: '0 0 * * *'  # Daily

  jobs:
    real-api:
      if: github.repository == 'owner/repo'  # Only main repo
      env:
        SPIRIS_ACCESS_TOKEN: ${{ secrets.SPIRIS_ACCESS_TOKEN }}
      steps:
        - run: cargo test -- --ignored
  ```

### 8.2 Pre-commit Hooks

- [ ] **Add test requirements to hooks**
  ```yaml
  # .pre-commit-config.yaml
  - repo: local
    hooks:
      - id: cargo-test
        name: cargo test
        entry: cargo test --lib
        language: system
        pass_filenames: false
  ```

---

## Test Coverage Goals

| Module | Current | Target |
|--------|---------|--------|
| `auth.rs` | ~60% | 95% |
| `client.rs` | ~70% | 90% |
| `retry.rs` | ~80% | 95% |
| `types.rs` | ~40% | 80% |
| `endpoints/*` | ~50% | 85% |
| **Overall** | ~55% | **85%** |

---

## Test Naming Conventions

```
test_<module>_<scenario>_<expected_outcome>

Examples:
- test_retry_on_500_succeeds_after_3_attempts
- test_pkce_verifier_missing_returns_error
- test_stream_empty_result_yields_nothing
- test_filter_with_quotes_escapes_correctly
```

---

## Running Tests

```bash
# All tests
cargo test

# Specific test file
cargo test --test retry_test

# Specific test
cargo test test_retry_on_500

# With output
cargo test -- --nocapture

# Real API tests (requires token)
SPIRIS_ACCESS_TOKEN=xxx cargo test -- --ignored

# With coverage
cargo tarpaulin --out html

# Benchmarks
cargo bench
```
