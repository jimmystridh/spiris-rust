//! OAuth2 authentication for the Spiris Bokföring och Fakturering API.

use crate::error::{Error, Result};
use chrono::{DateTime, Duration, Utc};
use oauth2::{
    basic::BasicClient, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    PkceCodeChallenge, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use serde::{Deserialize, Serialize};

/// OAuth2 configuration for Spiris Bokföring och Fakturering.
///
/// You can obtain OAuth2 credentials by registering your application
/// in the [Visma Developer Portal](https://developer.visma.com/).
#[derive(Debug, Clone)]
pub struct OAuth2Config {
    /// Client ID from Visma developer portal.
    pub client_id: String,

    /// Client secret from Visma developer portal.
    pub client_secret: String,

    /// Redirect URI registered in Visma developer portal.
    /// Must exactly match the URI registered in your application settings.
    pub redirect_uri: String,

    /// Authorization endpoint URL.
    pub auth_url: String,

    /// Token endpoint URL.
    pub token_url: String,
}

impl Default for OAuth2Config {
    fn default() -> Self {
        Self {
            client_id: String::new(),
            client_secret: String::new(),
            redirect_uri: String::new(),
            auth_url: "https://identity.vismaonline.com/connect/authorize".to_string(),
            token_url: "https://identity.vismaonline.com/connect/token".to_string(),
        }
    }
}

impl OAuth2Config {
    /// Create a new OAuth2 configuration.
    ///
    /// # Arguments
    ///
    /// * `client_id` - OAuth2 client ID from developer portal
    /// * `client_secret` - OAuth2 client secret from developer portal
    /// * `redirect_uri` - Callback URI for OAuth2 flow
    ///
    /// # Example
    ///
    /// ```
    /// use spiris_bokforing::auth::OAuth2Config;
    ///
    /// let config = OAuth2Config::new(
    ///     "your_client_id".to_string(),
    ///     "your_client_secret".to_string(),
    ///     "http://localhost:8080/callback".to_string(),
    /// );
    /// ```
    pub fn new(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        Self {
            client_id,
            client_secret,
            redirect_uri,
            ..Default::default()
        }
    }
}

/// Access token with expiration tracking.
///
/// Tokens typically expire after 1 hour. Use `is_expired()` to check
/// if a token needs to be refreshed before making API requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessToken {
    /// The access token string.
    pub token: String,

    /// When the token expires (UTC).
    pub expires_at: DateTime<Utc>,

    /// Refresh token for obtaining new access tokens.
    /// Required for token refresh flow.
    pub refresh_token: Option<String>,

    /// Token type (usually "Bearer").
    pub token_type: String,
}

impl AccessToken {
    /// Create a new access token.
    ///
    /// # Arguments
    ///
    /// * `token` - The access token string
    /// * `expires_in` - Token lifetime in seconds
    /// * `refresh_token` - Optional refresh token for token renewal
    ///
    /// # Example
    ///
    /// ```
    /// use spiris_bokforing::AccessToken;
    ///
    /// // Token expires in 1 hour (3600 seconds)
    /// let token = AccessToken::new(
    ///     "access_token_string".to_string(),
    ///     3600,
    ///     Some("refresh_token_string".to_string())
    /// );
    /// ```
    pub fn new(token: String, expires_in: i64, refresh_token: Option<String>) -> Self {
        let expires_at = Utc::now() + Duration::seconds(expires_in);
        Self {
            token,
            expires_at,
            refresh_token,
            token_type: "Bearer".to_string(),
        }
    }

    /// Check if the token is expired or will expire soon (within 5 minutes).
    ///
    /// Returns `true` if the token should be refreshed.
    ///
    /// # Example
    ///
    /// ```
    /// # use spiris_bokforing::AccessToken;
    /// # let token = AccessToken::new("token".to_string(), 3600, None);
    /// if token.is_expired() {
    ///     println!("Token needs to be refreshed!");
    /// }
    /// ```
    pub fn is_expired(&self) -> bool {
        let buffer = Duration::minutes(5);
        Utc::now() + buffer >= self.expires_at
    }

    /// Get the authorization header value.
    ///
    /// Returns a string in the format "Bearer {token}" suitable
    /// for use in HTTP Authorization headers.
    pub fn authorization_header(&self) -> String {
        format!("{} {}", self.token_type, self.token)
    }
}

/// OAuth2 authentication handler.
pub struct OAuth2Handler {
    #[allow(dead_code)]
    config: OAuth2Config,
    client: BasicClient,
}

impl OAuth2Handler {
    /// Create a new OAuth2 handler.
    pub fn new(config: OAuth2Config) -> Result<Self> {
        let client = BasicClient::new(
            ClientId::new(config.client_id.clone()),
            Some(ClientSecret::new(config.client_secret.clone())),
            AuthUrl::new(config.auth_url.clone())
                .map_err(|e| Error::InvalidConfig(format!("Invalid auth URL: {}", e)))?,
            Some(
                TokenUrl::new(config.token_url.clone())
                    .map_err(|e| Error::InvalidConfig(format!("Invalid token URL: {}", e)))?,
            ),
        )
        .set_redirect_uri(
            RedirectUrl::new(config.redirect_uri.clone())
                .map_err(|e| Error::InvalidConfig(format!("Invalid redirect URI: {}", e)))?,
        );

        Ok(Self { config, client })
    }

    /// Generate an authorization URL for the OAuth2 flow.
    ///
    /// Returns a tuple of (authorization_url, csrf_token, pkce_verifier).
    /// The user should be redirected to the authorization URL to approve access.
    pub fn authorize_url(&self) -> (String, String, String) {
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let (auth_url, csrf_token) = self
            .client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("ea:api".to_string()))
            .add_scope(Scope::new("ea:sales".to_string()))
            .add_scope(Scope::new("offline_access".to_string()))
            .set_pkce_challenge(pkce_challenge)
            .url();

        (
            auth_url.to_string(),
            csrf_token.secret().to_string(),
            pkce_verifier.secret().to_string(),
        )
    }

    /// Exchange an authorization code for an access token.
    ///
    /// This should be called after the user approves access and is redirected
    /// back to your application with an authorization code.
    pub async fn exchange_code(&self, code: String, _pkce_verifier: String) -> Result<AccessToken> {
        let token_result = self
            .client
            .exchange_code(AuthorizationCode::new(code))
            .request_async(oauth2::reqwest::async_http_client)
            .await
            .map_err(|e| Error::OAuth2Error(format!("Token exchange failed: {}", e)))?;

        let expires_in = token_result
            .expires_in()
            .map(|d| d.as_secs() as i64)
            .unwrap_or(3600); // Default to 1 hour

        Ok(AccessToken::new(
            token_result.access_token().secret().to_string(),
            expires_in,
            token_result.refresh_token().map(|t| t.secret().to_string()),
        ))
    }

    /// Refresh an access token using a refresh token.
    ///
    /// # Arguments
    ///
    /// * `refresh_token` - The refresh token obtained during initial authorization
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use spiris_bokforing::auth::{OAuth2Config, OAuth2Handler};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let config = OAuth2Config::new("id".to_string(), "secret".to_string(), "uri".to_string());
    /// let handler = OAuth2Handler::new(config)?;
    /// let refresh_token = "existing_refresh_token".to_string();
    /// let new_token = handler.refresh_token(refresh_token).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn refresh_token(&self, refresh_token: String) -> Result<AccessToken> {
        use oauth2::RefreshToken;

        let token_result = self
            .client
            .exchange_refresh_token(&RefreshToken::new(refresh_token))
            .request_async(oauth2::reqwest::async_http_client)
            .await
            .map_err(|e| Error::OAuth2Error(format!("Token refresh failed: {}", e)))?;

        let expires_in = token_result
            .expires_in()
            .map(|d| d.as_secs() as i64)
            .unwrap_or(3600);

        Ok(AccessToken::new(
            token_result.access_token().secret().to_string(),
            expires_in,
            token_result.refresh_token().map(|t| t.secret().to_string()),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_access_token_expiration() {
        let token = AccessToken::new("test_token".to_string(), 3600, None);
        assert!(!token.is_expired());

        let expired_token = AccessToken::new("test_token".to_string(), 0, None);
        // Wait a bit to ensure expiration
        std::thread::sleep(std::time::Duration::from_millis(100));
        assert!(expired_token.is_expired());
    }

    #[test]
    fn test_authorization_header() {
        let token = AccessToken::new("test_token_123".to_string(), 3600, None);
        assert_eq!(token.authorization_header(), "Bearer test_token_123");
    }
}
