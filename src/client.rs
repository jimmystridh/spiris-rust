//! Core HTTP client for the Spiris Bokföring och Fakturering API.

use crate::auth::{AccessToken, OAuth2Config, OAuth2Handler};
use crate::error::{Error, Result};
use crate::middleware::{MiddlewareStack, RequestContext, RequestTimer, ResponseContext};
use crate::retry::RetryConfig;
use reqwest::{header, Client as HttpClient, Method, RequestBuilder, Response, StatusCode};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::{Arc, RwLock};
use tokio::sync::Mutex;
use url::Url;

#[cfg(feature = "tracing")]
use tracing::{debug, error, info, warn};

/// Default base URL for the Spiris Bokföring och Fakturering API (v2).
/// Note: The API endpoint remains the same as the former Visma eAccounting.
pub const DEFAULT_BASE_URL: &str = "https://eaccountingapi.vismaonline.com/v2/";

/// Rate limit: 600 requests per minute per client per endpoint.
pub const RATE_LIMIT_PER_MINUTE: u32 = 600;

/// Configuration for the API client.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Base URL for the API.
    pub base_url: String,

    /// User agent string.
    pub user_agent: String,

    /// Request timeout in seconds.
    pub timeout_seconds: u64,

    /// Retry configuration.
    pub retry_config: RetryConfig,

    /// Enable request/response logging.
    pub enable_tracing: bool,

    /// OAuth2 configuration for automatic token refresh.
    /// When set, the client will automatically refresh expired tokens.
    pub oauth_config: Option<OAuth2Config>,

    /// Rate limiting configuration (requires `rate-limit` feature).
    #[cfg(feature = "rate-limit")]
    pub rate_limit_config: Option<crate::rate_limit::RateLimitConfig>,

    /// Middleware stack for request/response interception.
    pub middleware: MiddlewareStack,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            base_url: DEFAULT_BASE_URL.to_string(),
            user_agent: format!("spiris-bokforing-rust/{}", env!("CARGO_PKG_VERSION")),
            timeout_seconds: 30,
            retry_config: RetryConfig::default(),
            enable_tracing: true,
            oauth_config: None,
            #[cfg(feature = "rate-limit")]
            rate_limit_config: None,
            middleware: MiddlewareStack::new(),
        }
    }
}

impl ClientConfig {
    /// Create a new client configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the base URL.
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Set the request timeout.
    pub fn timeout_seconds(mut self, seconds: u64) -> Self {
        self.timeout_seconds = seconds;
        self
    }

    /// Set the retry configuration.
    pub fn retry_config(mut self, retry_config: RetryConfig) -> Self {
        self.retry_config = retry_config;
        self
    }

    /// Enable or disable tracing.
    pub fn enable_tracing(mut self, enable: bool) -> Self {
        self.enable_tracing = enable;
        self
    }

    /// Set OAuth2 configuration for automatic token refresh.
    ///
    /// When configured, the client will automatically refresh expired tokens
    /// using the refresh token, if available.
    pub fn oauth_config(mut self, oauth_config: OAuth2Config) -> Self {
        self.oauth_config = Some(oauth_config);
        self
    }

    /// Set rate limiting configuration.
    ///
    /// When configured, the client will limit request rates to avoid
    /// exceeding API quotas.
    #[cfg(feature = "rate-limit")]
    pub fn rate_limit_config(mut self, config: crate::rate_limit::RateLimitConfig) -> Self {
        self.rate_limit_config = Some(config);
        self
    }

    /// Add a middleware to the stack.
    ///
    /// Middleware is executed in the order it is added for requests,
    /// and in reverse order for responses.
    ///
    /// # Example
    ///
    /// ```
    /// use spiris::{ClientConfig, middleware::LoggingMiddleware};
    ///
    /// let config = ClientConfig::new()
    ///     .middleware(LoggingMiddleware::new());
    /// ```
    pub fn middleware<M: crate::middleware::Middleware + 'static>(mut self, middleware: M) -> Self {
        self.middleware.push(middleware);
        self
    }

    /// Set the middleware stack.
    ///
    /// This replaces any existing middleware.
    pub fn middleware_stack(mut self, stack: MiddlewareStack) -> Self {
        self.middleware = stack;
        self
    }
}

/// Main API client for Spiris Bokföring och Fakturering.
///
/// The client handles authentication, rate limiting, and HTTP communication
/// with the Spiris API.
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
///
///     // Use the client to make API calls
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct Client {
    http_client: HttpClient,
    config: ClientConfig,
    access_token: Arc<RwLock<AccessToken>>,
    /// Mutex to prevent concurrent token refresh operations.
    refresh_lock: Arc<Mutex<()>>,
    /// Rate limiter for API requests (requires `rate-limit` feature).
    #[cfg(feature = "rate-limit")]
    rate_limiter: Option<crate::rate_limit::ApiRateLimiter>,
    /// Middleware stack for request/response interception.
    middleware: MiddlewareStack,
}

impl Client {
    /// Create a new API client with an access token.
    pub fn new(access_token: AccessToken) -> Self {
        Self::with_config(access_token, ClientConfig::default())
    }

    /// Create a new API client with custom configuration.
    pub fn with_config(access_token: AccessToken, config: ClientConfig) -> Self {
        let http_client = HttpClient::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .build()
            .expect("Failed to build HTTP client");

        #[cfg(feature = "rate-limit")]
        let rate_limiter = config
            .rate_limit_config
            .as_ref()
            .map(crate::rate_limit::ApiRateLimiter::new);

        let middleware = config.middleware.clone();

        Self {
            http_client,
            config,
            access_token: Arc::new(RwLock::new(access_token)),
            refresh_lock: Arc::new(Mutex::new(())),
            #[cfg(feature = "rate-limit")]
            rate_limiter,
            middleware,
        }
    }

    /// Update the access token.
    ///
    /// This is useful when refreshing expired tokens.
    pub fn set_access_token(&self, token: AccessToken) {
        let mut access_token = self.access_token.write().unwrap();
        *access_token = token;
    }

    /// Get the current access token.
    pub fn get_access_token(&self) -> AccessToken {
        self.access_token.read().unwrap().clone()
    }

    /// Check if the current access token is expired.
    pub fn is_token_expired(&self) -> bool {
        self.access_token.read().unwrap().is_expired()
    }

    /// Ensure the access token is valid, refreshing if necessary.
    ///
    /// This method handles automatic token refresh when:
    /// 1. The token is expired or about to expire
    /// 2. OAuth2 configuration is available
    /// 3. A refresh token is available
    ///
    /// If any of these conditions are not met and the token is expired,
    /// returns `Error::TokenExpired`.
    ///
    /// Uses a mutex to prevent multiple concurrent refresh operations.
    async fn ensure_valid_token(&self) -> Result<()> {
        // Quick check without lock - if token is valid, we're done
        if !self.is_token_expired() {
            return Ok(());
        }

        #[cfg(feature = "tracing")]
        debug!("Token expired, attempting refresh");

        // Token is expired, acquire refresh lock to prevent concurrent refreshes
        let _guard = self.refresh_lock.lock().await;

        // Double-check after acquiring lock (another request may have refreshed)
        if !self.is_token_expired() {
            #[cfg(feature = "tracing")]
            debug!("Token was refreshed by another request");
            return Ok(());
        }

        // Check if we can refresh
        let oauth_config = match &self.config.oauth_config {
            Some(config) => config.clone(),
            None => {
                #[cfg(feature = "tracing")]
                warn!("Token expired but no OAuth config available for refresh");
                return Err(Error::TokenExpired);
            }
        };

        let refresh_token = {
            let token = self.access_token.read().unwrap();
            match &token.refresh_token {
                Some(rt) => rt.clone(),
                None => {
                    #[cfg(feature = "tracing")]
                    warn!("Token expired but no refresh token available");
                    return Err(Error::TokenExpired);
                }
            }
        };

        // Perform the refresh
        #[cfg(feature = "tracing")]
        info!("Refreshing access token");

        let handler = OAuth2Handler::new(oauth_config)?;
        let new_token = handler.refresh_token(refresh_token).await?;

        #[cfg(feature = "tracing")]
        info!("Token refreshed successfully");

        // Update the token
        self.set_access_token(new_token);

        Ok(())
    }

    /// Build a URL for an API endpoint.
    fn build_url(&self, path: &str) -> Result<Url> {
        let base = Url::parse(&self.config.base_url)?;
        // Strip leading "/" to ensure proper joining with base URL
        let path = path.strip_prefix('/').unwrap_or(path);
        let url = base.join(path)?;
        Ok(url)
    }

    /// Build a request with authentication headers.
    fn build_request(&self, method: Method, url: Url) -> Result<RequestBuilder> {
        let token = self.access_token.read().unwrap();

        if token.is_expired() {
            return Err(Error::TokenExpired);
        }

        let request = self
            .http_client
            .request(method, url)
            .header(header::AUTHORIZATION, token.authorization_header())
            .header(header::USER_AGENT, &self.config.user_agent)
            .header(header::ACCEPT, "application/json");

        Ok(request)
    }

    /// Execute a request and handle the response with automatic retry on transient errors.
    async fn execute_request(
        &self,
        method: &str,
        url: &str,
        request: RequestBuilder,
    ) -> Result<Response> {
        #[cfg(feature = "tracing")]
        let span = tracing::info_span!("api_request", %method, %url);
        #[cfg(feature = "tracing")]
        let _guard = span.enter();

        #[cfg(feature = "tracing")]
        debug!("Sending API request");

        // Create middleware context
        let mut ctx = RequestContext::new(method, url);
        let timer = RequestTimer::start();

        // Process middleware on_request handlers
        if !self.middleware.is_empty() {
            self.middleware.process_request(&mut ctx)?;
        }

        // Apply any headers added by middleware
        let request = ctx
            .headers
            .iter()
            .fold(request, |req, (k, v)| req.header(k.as_str(), v.as_str()));

        // Apply rate limiting if configured
        #[cfg(feature = "rate-limit")]
        if let Some(ref limiter) = self.rate_limiter {
            #[cfg(feature = "tracing")]
            debug!("Waiting for rate limiter");
            limiter.acquire().await;
        }

        // Execute the request
        let result = self.execute_request_inner(request).await;
        let elapsed = timer.elapsed();

        // Log the result
        #[cfg(feature = "tracing")]
        match &result {
            Ok(response) => {
                info!(
                    status = response.status().as_u16(),
                    duration_ms = elapsed.as_millis() as u64,
                    "API request completed"
                );
            }
            Err(err) => {
                error!(
                    error = %err,
                    duration_ms = elapsed.as_millis() as u64,
                    "API request failed"
                );
            }
        }

        // Process middleware on_response handlers
        if !self.middleware.is_empty() {
            let response_ctx = match &result {
                Ok(response) => ResponseContext::new(
                    method.to_string(),
                    url.to_string(),
                    response.status().as_u16(),
                    elapsed,
                    ctx.extensions,
                ),
                Err(err) => ResponseContext::with_error(
                    method.to_string(),
                    url.to_string(),
                    elapsed,
                    err.to_string(),
                    ctx.extensions,
                ),
            };
            self.middleware.process_response(&response_ctx);
        }

        result
    }

    /// Inner request execution with retry logic.
    async fn execute_request_inner(&self, request: RequestBuilder) -> Result<Response> {
        // If retries are disabled, just send directly
        if self.config.retry_config.max_retries == 0 {
            let response = request.send().await?;
            return self.handle_response(response).await;
        }

        // Clone the request for potential retries
        // Note: try_clone() returns None if the body is a stream that can't be cloned
        let request_clone = request.try_clone().ok_or_else(|| {
            Error::InvalidRequest("Request body cannot be cloned for retry".into())
        })?;

        // Try the first request
        let response = request.send().await?;
        let first_result = self.handle_response(response).await;

        match first_result {
            Ok(response) => Ok(response),
            Err(err) if crate::retry::is_retryable_error(&err) => {
                #[cfg(feature = "tracing")]
                warn!(error = %err, "Request failed, will retry");

                // Use retry logic for retryable errors
                crate::retry::retry_request(&self.config.retry_config, || async {
                    // We need to rebuild the request each time
                    let url = request_clone
                        .try_clone()
                        .ok_or_else(|| Error::InvalidRequest("Request cannot be cloned".into()))?;
                    let response = url.send().await?;
                    self.handle_response(response).await
                })
                .await
            }
            Err(err) => Err(err),
        }
    }

    /// Handle API response, checking for errors.
    ///
    /// This method parses error responses into structured `ApiErrorResponse` objects
    /// when possible, providing access to error codes and field-level validation errors.
    async fn handle_response(&self, response: Response) -> Result<Response> {
        let status = response.status();

        match status {
            StatusCode::OK | StatusCode::CREATED | StatusCode::NO_CONTENT => Ok(response),
            StatusCode::UNAUTHORIZED => Err(Error::AuthError("Unauthorized".to_string())),
            StatusCode::FORBIDDEN => Err(Error::AuthError("Forbidden".to_string())),
            StatusCode::NOT_FOUND => {
                let raw_body = response.text().await.unwrap_or_default();
                Err(Error::NotFound(raw_body))
            }
            StatusCode::TOO_MANY_REQUESTS => {
                let raw_body = response.text().await.unwrap_or_default();
                Err(Error::RateLimitExceeded(raw_body))
            }
            StatusCode::BAD_REQUEST => {
                let raw_body = response.text().await.unwrap_or_default();
                Err(Error::InvalidRequest(raw_body))
            }
            _ => {
                let raw_body = response.text().await.unwrap_or_default();
                Err(Error::from_api_response(status.as_u16(), raw_body))
            }
        }
    }

    /// Make a GET request to an API endpoint.
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        self.ensure_valid_token().await?;
        let url = self.build_url(path)?;
        let url_str = url.to_string();
        let request = self.build_request(Method::GET, url)?;
        let response = self.execute_request("GET", &url_str, request).await?;
        let data = response.json().await?;
        Ok(data)
    }

    /// Make a GET request with query parameters.
    pub async fn get_with_params<T: DeserializeOwned, P: Serialize>(
        &self,
        path: &str,
        params: &P,
    ) -> Result<T> {
        self.ensure_valid_token().await?;
        let url = self.build_url(path)?;
        let url_str = url.to_string();
        let request = self.build_request(Method::GET, url)?.query(params);
        let response = self.execute_request("GET", &url_str, request).await?;
        let data = response.json().await?;
        Ok(data)
    }

    /// Make a POST request to create a resource.
    pub async fn post<T: DeserializeOwned, B: Serialize>(&self, path: &str, body: &B) -> Result<T> {
        self.ensure_valid_token().await?;
        let url = self.build_url(path)?;
        let url_str = url.to_string();
        let request = self
            .build_request(Method::POST, url)?
            .header(header::CONTENT_TYPE, "application/json")
            .json(body);
        let response = self.execute_request("POST", &url_str, request).await?;
        let data = response.json().await?;
        Ok(data)
    }

    /// Make a PUT request to update a resource.
    pub async fn put<T: DeserializeOwned, B: Serialize>(&self, path: &str, body: &B) -> Result<T> {
        self.ensure_valid_token().await?;
        let url = self.build_url(path)?;
        let url_str = url.to_string();
        let request = self
            .build_request(Method::PUT, url)?
            .header(header::CONTENT_TYPE, "application/json")
            .json(body);
        let response = self.execute_request("PUT", &url_str, request).await?;
        let data = response.json().await?;
        Ok(data)
    }

    /// Make a DELETE request to remove a resource.
    pub async fn delete(&self, path: &str) -> Result<()> {
        self.ensure_valid_token().await?;
        let url = self.build_url(path)?;
        let url_str = url.to_string();
        let request = self.build_request(Method::DELETE, url)?;
        self.execute_request("DELETE", &url_str, request).await?;
        Ok(())
    }

    /// Make a GET request that returns raw bytes (for binary data like PDFs).
    pub async fn get_bytes(&self, path: &str) -> Result<Vec<u8>> {
        self.ensure_valid_token().await?;
        let url = self.build_url(path)?;
        let url_str = url.to_string();
        let request = self.build_request(Method::GET, url)?;
        let response = self.execute_request("GET", &url_str, request).await?;
        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_url() {
        let token = AccessToken::new("test".to_string(), 3600, None);
        let client = Client::new(token);

        let url = client.build_url("/customers").unwrap();
        assert_eq!(
            url.as_str(),
            "https://eaccountingapi.vismaonline.com/v2/customers"
        );
    }

    #[test]
    fn test_token_expiration_check() {
        let expired_token = AccessToken::new("test".to_string(), 0, None);
        let client = Client::new(expired_token);

        std::thread::sleep(std::time::Duration::from_millis(100));
        assert!(client.is_token_expired());
    }
}
