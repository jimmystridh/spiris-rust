//! Error types for the Visma eAccounting API client.

use thiserror::Error;

/// Result type for Visma eAccounting API operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur when using the Visma eAccounting API client.
#[derive(Error, Debug)]
pub enum Error {
    /// HTTP request failed.
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// Failed to parse JSON response.
    #[error("JSON parsing failed: {0}")]
    Json(#[from] serde_json::Error),

    /// API returned an error response.
    #[error("API error: {status_code} - {message}")]
    ApiError {
        status_code: u16,
        message: String,
    },

    /// Authentication failed.
    #[error("Authentication failed: {0}")]
    AuthError(String),

    /// Token has expired.
    #[error("Access token has expired")]
    TokenExpired,

    /// Invalid configuration.
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Rate limit exceeded.
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    /// Resource not found.
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Invalid request parameters.
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// URL parsing error.
    #[error("URL parsing failed: {0}")]
    UrlParseError(#[from] url::ParseError),

    /// OAuth2 error.
    #[error("OAuth2 error: {0}")]
    OAuth2Error(String),
}
