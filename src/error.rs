//! Error types for the Visma eAccounting API client.

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Result type for Visma eAccounting API operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Structured API error response from the Visma eAccounting API.
///
/// This struct represents the parsed error response body returned by the API
/// when a request fails. It provides structured access to error details including
/// an error code, message, and any field-level validation errors.
///
/// # Example
///
/// ```
/// use spiris::error::{Error, ApiErrorResponse};
///
/// fn handle_error(err: Error) {
///     if let Error::ApiError { status_code, response, .. } = err {
///         println!("Error {}: {}", status_code, response.message);
///         for validation_err in &response.validation_errors {
///             println!("  Field '{}': {}", validation_err.field, validation_err.message);
///         }
///     }
/// }
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ApiErrorResponse {
    /// Error code from the API (if provided).
    #[serde(default)]
    pub error_code: Option<String>,

    /// Human-readable error message.
    #[serde(default)]
    pub message: String,

    /// Field-level validation errors.
    #[serde(default)]
    pub validation_errors: Vec<ValidationError>,
}

impl ApiErrorResponse {
    /// Create a new API error response from a raw message string.
    ///
    /// This is used as a fallback when the API returns a non-JSON error body.
    pub fn from_raw(message: String) -> Self {
        Self {
            error_code: None,
            message,
            validation_errors: vec![],
        }
    }

    /// Check if this error contains validation errors.
    pub fn has_validation_errors(&self) -> bool {
        !self.validation_errors.is_empty()
    }

    /// Get a validation error for a specific field, if present.
    pub fn validation_error_for(&self, field: &str) -> Option<&ValidationError> {
        self.validation_errors.iter().find(|e| e.field == field)
    }
}

impl fmt::Display for ApiErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;
        if !self.validation_errors.is_empty() {
            write!(f, " (")?;
            for (i, err) in self.validation_errors.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}: {}", err.field, err.message)?;
            }
            write!(f, ")")?;
        }
        Ok(())
    }
}

/// A field-level validation error from the API.
///
/// These are typically returned when a request body fails validation,
/// such as missing required fields or invalid field values.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ValidationError {
    /// The field name that failed validation.
    pub field: String,

    /// The validation error message.
    pub message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

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
    ///
    /// This error includes the HTTP status code, a structured response
    /// (if parseable), and the raw response body for debugging.
    ///
    /// # Example
    ///
    /// ```
    /// use spiris::error::Error;
    ///
    /// fn check_error(err: &Error) {
    ///     if let Error::ApiError { status_code, response, raw_body } = err {
    ///         println!("Status: {}", status_code);
    ///         println!("Message: {}", response.message);
    ///         if response.has_validation_errors() {
    ///             for err in &response.validation_errors {
    ///                 println!("  {}: {}", err.field, err.message);
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    #[error("API error ({status_code}): {response}")]
    ApiError {
        /// HTTP status code.
        status_code: u16,
        /// Parsed error response.
        response: ApiErrorResponse,
        /// Raw response body for debugging.
        raw_body: String,
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

impl Error {
    /// Create an API error from a status code and response body.
    ///
    /// Attempts to parse the body as JSON. If parsing fails, uses the
    /// raw body as the error message.
    pub fn from_api_response(status_code: u16, raw_body: String) -> Self {
        let response: ApiErrorResponse = serde_json::from_str(&raw_body)
            .unwrap_or_else(|_| ApiErrorResponse::from_raw(raw_body.clone()));

        Error::ApiError {
            status_code,
            response,
            raw_body,
        }
    }

    /// Check if this error is retryable.
    ///
    /// Returns `true` for transient errors like rate limiting or server errors.
    pub fn is_retryable(&self) -> bool {
        match self {
            Error::RateLimitExceeded(_) => true,
            Error::ApiError { status_code, .. } => *status_code >= 500,
            Error::Http(e) => e.is_timeout() || e.is_connect(),
            _ => false,
        }
    }

    /// Get the HTTP status code if this is an API error.
    pub fn status_code(&self) -> Option<u16> {
        match self {
            Error::ApiError { status_code, .. } => Some(*status_code),
            _ => None,
        }
    }

    /// Get the validation errors if this is an API error with validation failures.
    pub fn validation_errors(&self) -> Option<&[ValidationError]> {
        match self {
            Error::ApiError { response, .. } if response.has_validation_errors() => {
                Some(&response.validation_errors)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_error_response_from_raw() {
        let response = ApiErrorResponse::from_raw("Something went wrong".to_string());
        assert_eq!(response.message, "Something went wrong");
        assert!(response.error_code.is_none());
        assert!(response.validation_errors.is_empty());
    }

    #[test]
    fn test_api_error_response_parsing() {
        let json = r#"{
            "ErrorCode": "InvalidInput",
            "Message": "Validation failed",
            "ValidationErrors": [
                {"Field": "Name", "Message": "Name is required"},
                {"Field": "Email", "Message": "Invalid email format"}
            ]
        }"#;

        let response: ApiErrorResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.error_code, Some("InvalidInput".to_string()));
        assert_eq!(response.message, "Validation failed");
        assert_eq!(response.validation_errors.len(), 2);
        assert_eq!(response.validation_errors[0].field, "Name");
        assert_eq!(response.validation_errors[0].message, "Name is required");
    }

    #[test]
    fn test_api_error_response_has_validation_errors() {
        let response = ApiErrorResponse {
            error_code: None,
            message: "Test".to_string(),
            validation_errors: vec![],
        };
        assert!(!response.has_validation_errors());

        let response = ApiErrorResponse {
            error_code: None,
            message: "Test".to_string(),
            validation_errors: vec![ValidationError {
                field: "Field".to_string(),
                message: "Error".to_string(),
            }],
        };
        assert!(response.has_validation_errors());
    }

    #[test]
    fn test_api_error_response_validation_error_for() {
        let response = ApiErrorResponse {
            error_code: None,
            message: "Test".to_string(),
            validation_errors: vec![
                ValidationError {
                    field: "Name".to_string(),
                    message: "Required".to_string(),
                },
                ValidationError {
                    field: "Email".to_string(),
                    message: "Invalid".to_string(),
                },
            ],
        };

        assert_eq!(
            response.validation_error_for("Name").unwrap().message,
            "Required"
        );
        assert_eq!(
            response.validation_error_for("Email").unwrap().message,
            "Invalid"
        );
        assert!(response.validation_error_for("Phone").is_none());
    }

    #[test]
    fn test_api_error_response_display() {
        let response = ApiErrorResponse {
            error_code: Some("ERR001".to_string()),
            message: "Something failed".to_string(),
            validation_errors: vec![],
        };
        assert_eq!(response.to_string(), "Something failed");

        let response = ApiErrorResponse {
            error_code: None,
            message: "Validation failed".to_string(),
            validation_errors: vec![
                ValidationError {
                    field: "Name".to_string(),
                    message: "Required".to_string(),
                },
                ValidationError {
                    field: "Email".to_string(),
                    message: "Invalid".to_string(),
                },
            ],
        };
        assert_eq!(
            response.to_string(),
            "Validation failed (Name: Required, Email: Invalid)"
        );
    }

    #[test]
    fn test_error_from_api_response_json() {
        let json = r#"{"Message": "Not found", "ErrorCode": "404"}"#;
        let err = Error::from_api_response(404, json.to_string());

        if let Error::ApiError {
            status_code,
            response,
            raw_body,
        } = err
        {
            assert_eq!(status_code, 404);
            assert_eq!(response.message, "Not found");
            assert_eq!(response.error_code, Some("404".to_string()));
            assert_eq!(raw_body, json);
        } else {
            panic!("Expected ApiError");
        }
    }

    #[test]
    fn test_error_from_api_response_plain_text() {
        let text = "Internal Server Error";
        let err = Error::from_api_response(500, text.to_string());

        if let Error::ApiError {
            status_code,
            response,
            raw_body,
        } = err
        {
            assert_eq!(status_code, 500);
            assert_eq!(response.message, text);
            assert!(response.error_code.is_none());
            assert_eq!(raw_body, text);
        } else {
            panic!("Expected ApiError");
        }
    }

    #[test]
    fn test_error_is_retryable() {
        assert!(Error::RateLimitExceeded("test".to_string()).is_retryable());
        assert!(Error::from_api_response(500, "Server error".to_string()).is_retryable());
        assert!(Error::from_api_response(502, "Bad gateway".to_string()).is_retryable());
        assert!(!Error::from_api_response(400, "Bad request".to_string()).is_retryable());
        assert!(!Error::TokenExpired.is_retryable());
        assert!(!Error::NotFound("resource".to_string()).is_retryable());
    }

    #[test]
    fn test_error_status_code() {
        assert_eq!(
            Error::from_api_response(400, "test".to_string()).status_code(),
            Some(400)
        );
        assert!(Error::TokenExpired.status_code().is_none());
    }

    #[test]
    fn test_error_validation_errors() {
        let json = r#"{
            "Message": "Validation failed",
            "ValidationErrors": [{"Field": "Name", "Message": "Required"}]
        }"#;
        let err = Error::from_api_response(400, json.to_string());
        let validation_errors = err.validation_errors().unwrap();
        assert_eq!(validation_errors.len(), 1);
        assert_eq!(validation_errors[0].field, "Name");

        assert!(Error::TokenExpired.validation_errors().is_none());
    }
}
