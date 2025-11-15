//! Core HTTP client for the Visma eAccounting API.

use crate::auth::AccessToken;
use crate::error::{Error, Result};
use reqwest::{header, Client as HttpClient, Method, RequestBuilder, Response, StatusCode};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::{Arc, RwLock};
use url::Url;

/// Default base URL for the Visma eAccounting API (v2).
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
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            base_url: DEFAULT_BASE_URL.to_string(),
            user_agent: format!("visma-eaccounting-rust/{}", env!("CARGO_PKG_VERSION")),
            timeout_seconds: 30,
        }
    }
}

/// Main API client for Visma eAccounting.
///
/// The client handles authentication, rate limiting, and HTTP communication
/// with the Visma eAccounting API.
///
/// # Example
///
/// ```no_run
/// use visma_eaccounting::{Client, AccessToken};
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

        Self {
            http_client,
            config,
            access_token: Arc::new(RwLock::new(access_token)),
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

    /// Execute a request and handle the response.
    async fn execute_request(&self, request: RequestBuilder) -> Result<Response> {
        let response = request.send().await?;
        self.handle_response(response).await
    }

    /// Handle API response, checking for errors.
    async fn handle_response(&self, response: Response) -> Result<Response> {
        let status = response.status();

        match status {
            StatusCode::OK | StatusCode::CREATED | StatusCode::NO_CONTENT => Ok(response),
            StatusCode::UNAUTHORIZED => Err(Error::AuthError("Unauthorized".to_string())),
            StatusCode::FORBIDDEN => Err(Error::AuthError("Forbidden".to_string())),
            StatusCode::NOT_FOUND => {
                let error_msg = response.text().await.unwrap_or_default();
                Err(Error::NotFound(error_msg))
            }
            StatusCode::TOO_MANY_REQUESTS => {
                let error_msg = response.text().await.unwrap_or_default();
                Err(Error::RateLimitExceeded(error_msg))
            }
            StatusCode::BAD_REQUEST => {
                let error_msg = response.text().await.unwrap_or_default();
                Err(Error::InvalidRequest(error_msg))
            }
            _ => {
                let error_msg = response.text().await.unwrap_or_default();
                Err(Error::ApiError {
                    status_code: status.as_u16(),
                    message: error_msg,
                })
            }
        }
    }

    /// Make a GET request to an API endpoint.
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = self.build_url(path)?;
        let request = self.build_request(Method::GET, url)?;
        let response = self.execute_request(request).await?;
        let data = response.json().await?;
        Ok(data)
    }

    /// Make a GET request with query parameters.
    pub async fn get_with_params<T: DeserializeOwned, P: Serialize>(
        &self,
        path: &str,
        params: &P,
    ) -> Result<T> {
        let url = self.build_url(path)?;
        let request = self.build_request(Method::GET, url)?.query(params);
        let response = self.execute_request(request).await?;
        let data = response.json().await?;
        Ok(data)
    }

    /// Make a POST request to create a resource.
    pub async fn post<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = self.build_url(path)?;
        let request = self
            .build_request(Method::POST, url)?
            .header(header::CONTENT_TYPE, "application/json")
            .json(body);
        let response = self.execute_request(request).await?;
        let data = response.json().await?;
        Ok(data)
    }

    /// Make a PUT request to update a resource.
    pub async fn put<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = self.build_url(path)?;
        let request = self
            .build_request(Method::PUT, url)?
            .header(header::CONTENT_TYPE, "application/json")
            .json(body);
        let response = self.execute_request(request).await?;
        let data = response.json().await?;
        Ok(data)
    }

    /// Make a DELETE request to remove a resource.
    pub async fn delete(&self, path: &str) -> Result<()> {
        let url = self.build_url(path)?;
        let request = self.build_request(Method::DELETE, url)?;
        self.execute_request(request).await?;
        Ok(())
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
