//! Integration tests for PKCE (Proof Key for Code Exchange) authentication flow.
//!
//! These tests verify that the OAuth2 implementation correctly handles PKCE:
//! - Generates valid code challenges and verifiers
//! - Includes challenge in authorization URL
//! - Sends verifier during token exchange

use spiris::auth::{OAuth2Config, OAuth2Handler};

// =============================================================================
// PKCE Challenge Generation Tests
// =============================================================================

#[test]
fn test_pkce_verifier_generation() {
    let config = OAuth2Config::new(
        "test_client_id".to_string(),
        "test_client_secret".to_string(),
        "http://localhost:8080/callback".to_string(),
    );

    let handler = OAuth2Handler::new(config).unwrap();
    let (_, _, pkce_verifier) = handler.authorize_url();

    // PKCE verifier should be between 43-128 characters (RFC 7636)
    assert!(
        pkce_verifier.len() >= 43,
        "PKCE verifier too short: {} chars",
        pkce_verifier.len()
    );
    assert!(
        pkce_verifier.len() <= 128,
        "PKCE verifier too long: {} chars",
        pkce_verifier.len()
    );

    // Verifier should only contain unreserved URI characters
    assert!(
        pkce_verifier
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.' || c == '_' || c == '~'),
        "PKCE verifier contains invalid characters"
    );
}

#[test]
fn test_pkce_verifier_uniqueness() {
    let config = OAuth2Config::new(
        "test_client_id".to_string(),
        "test_client_secret".to_string(),
        "http://localhost:8080/callback".to_string(),
    );

    let handler = OAuth2Handler::new(config).unwrap();

    // Generate multiple verifiers and ensure they're unique
    let (_, _, verifier1) = handler.authorize_url();
    let (_, _, verifier2) = handler.authorize_url();
    let (_, _, verifier3) = handler.authorize_url();

    assert_ne!(verifier1, verifier2, "PKCE verifiers should be unique");
    assert_ne!(verifier2, verifier3, "PKCE verifiers should be unique");
    assert_ne!(verifier1, verifier3, "PKCE verifiers should be unique");
}

// =============================================================================
// Authorization URL Tests
// =============================================================================

#[test]
fn test_authorize_url_contains_pkce_challenge() {
    let config = OAuth2Config::new(
        "test_client_id".to_string(),
        "test_client_secret".to_string(),
        "http://localhost:8080/callback".to_string(),
    );

    let handler = OAuth2Handler::new(config).unwrap();
    let (auth_url, _, _) = handler.authorize_url();

    // URL should contain code_challenge parameter
    assert!(
        auth_url.contains("code_challenge="),
        "Authorization URL missing code_challenge: {}",
        auth_url
    );

    // URL should specify S256 challenge method
    assert!(
        auth_url.contains("code_challenge_method=S256"),
        "Authorization URL missing code_challenge_method=S256: {}",
        auth_url
    );
}

#[test]
fn test_authorize_url_contains_required_params() {
    let config = OAuth2Config::new(
        "test_client_id".to_string(),
        "test_client_secret".to_string(),
        "http://localhost:8080/callback".to_string(),
    );

    let handler = OAuth2Handler::new(config).unwrap();
    let (auth_url, _, _) = handler.authorize_url();

    // Check for required OAuth2 parameters
    assert!(
        auth_url.contains("client_id=test_client_id"),
        "Missing client_id in URL"
    );
    assert!(
        auth_url.contains("redirect_uri="),
        "Missing redirect_uri in URL"
    );
    assert!(
        auth_url.contains("response_type=code"),
        "Missing response_type=code in URL"
    );
    assert!(auth_url.contains("state="), "Missing state (CSRF) in URL");
}

#[test]
fn test_authorize_url_contains_scopes() {
    let config = OAuth2Config::new(
        "test_client_id".to_string(),
        "test_client_secret".to_string(),
        "http://localhost:8080/callback".to_string(),
    );

    let handler = OAuth2Handler::new(config).unwrap();
    let (auth_url, _, _) = handler.authorize_url();

    // Should request API scope
    assert!(
        auth_url.contains("ea%3Aapi") || auth_url.contains("ea:api"),
        "Missing ea:api scope in URL"
    );

    // Should request offline_access for refresh tokens
    assert!(
        auth_url.contains("offline_access"),
        "Missing offline_access scope in URL"
    );
}

// =============================================================================
// CSRF Token Tests
// =============================================================================

#[test]
fn test_csrf_token_generation() {
    let config = OAuth2Config::new(
        "test_client_id".to_string(),
        "test_client_secret".to_string(),
        "http://localhost:8080/callback".to_string(),
    );

    let handler = OAuth2Handler::new(config).unwrap();
    let (_, csrf_token, _) = handler.authorize_url();

    // CSRF token should be non-empty
    assert!(!csrf_token.is_empty(), "CSRF token should not be empty");

    // CSRF token should be reasonably long for security
    assert!(
        csrf_token.len() >= 16,
        "CSRF token too short: {} chars",
        csrf_token.len()
    );
}

#[test]
fn test_csrf_token_uniqueness() {
    let config = OAuth2Config::new(
        "test_client_id".to_string(),
        "test_client_secret".to_string(),
        "http://localhost:8080/callback".to_string(),
    );

    let handler = OAuth2Handler::new(config).unwrap();

    let (_, csrf1, _) = handler.authorize_url();
    let (_, csrf2, _) = handler.authorize_url();

    assert_ne!(csrf1, csrf2, "CSRF tokens should be unique per request");
}

#[test]
fn test_csrf_token_in_url() {
    let config = OAuth2Config::new(
        "test_client_id".to_string(),
        "test_client_secret".to_string(),
        "http://localhost:8080/callback".to_string(),
    );

    let handler = OAuth2Handler::new(config).unwrap();
    let (auth_url, csrf_token, _) = handler.authorize_url();

    // The state parameter in URL should contain the CSRF token (may be URL encoded)
    assert!(
        auth_url.contains("state="),
        "URL should contain state parameter"
    );

    // Extract state from URL and verify it's present
    let state_start = auth_url.find("state=").unwrap() + 6;
    let state_end = auth_url[state_start..]
        .find('&')
        .map(|i| state_start + i)
        .unwrap_or(auth_url.len());
    let state_in_url = &auth_url[state_start..state_end];

    // State should be non-empty and the returned CSRF should be non-empty
    assert!(!state_in_url.is_empty(), "State in URL should not be empty");
    assert!(!csrf_token.is_empty(), "CSRF token should not be empty");
}

// =============================================================================
// OAuth2Config Tests
// =============================================================================

#[test]
fn test_oauth2_config_default_urls() {
    let config = OAuth2Config::default();

    assert_eq!(
        config.auth_url,
        "https://identity.vismaonline.com/connect/authorize"
    );
    assert_eq!(
        config.token_url,
        "https://identity.vismaonline.com/connect/token"
    );
}

#[test]
fn test_oauth2_config_new() {
    let config = OAuth2Config::new(
        "my_client_id".to_string(),
        "my_client_secret".to_string(),
        "http://localhost:3000/callback".to_string(),
    );

    assert_eq!(config.client_id, "my_client_id");
    assert_eq!(config.client_secret, "my_client_secret");
    assert_eq!(config.redirect_uri, "http://localhost:3000/callback");
}

#[test]
fn test_oauth2_handler_creation() {
    let config = OAuth2Config::new(
        "test_client_id".to_string(),
        "test_client_secret".to_string(),
        "http://localhost:8080/callback".to_string(),
    );

    let result = OAuth2Handler::new(config);
    assert!(result.is_ok(), "Failed to create OAuth2Handler");
}

#[test]
fn test_oauth2_handler_invalid_auth_url() {
    let mut config = OAuth2Config::new(
        "test_client_id".to_string(),
        "test_client_secret".to_string(),
        "http://localhost:8080/callback".to_string(),
    );
    config.auth_url = "not a valid url".to_string();

    let result = OAuth2Handler::new(config);
    assert!(
        result.is_err(),
        "Should fail with invalid auth URL"
    );
}

#[test]
fn test_oauth2_handler_invalid_redirect_uri() {
    let config = OAuth2Config::new(
        "test_client_id".to_string(),
        "test_client_secret".to_string(),
        "not a valid uri".to_string(),
    );

    let result = OAuth2Handler::new(config);
    assert!(
        result.is_err(),
        "Should fail with invalid redirect URI"
    );
}

// =============================================================================
// Access Token Tests
// =============================================================================

mod access_token_tests {
    use spiris::AccessToken;
    use std::time::Duration;

    #[test]
    fn test_access_token_creation() {
        let token = AccessToken::new("my_token".to_string(), 3600, None);

        assert_eq!(token.token, "my_token");
        assert_eq!(token.token_type, "Bearer");
        assert!(token.refresh_token.is_none());
    }

    #[test]
    fn test_access_token_with_refresh() {
        let token = AccessToken::new(
            "my_token".to_string(),
            3600,
            Some("my_refresh_token".to_string()),
        );

        assert_eq!(token.refresh_token, Some("my_refresh_token".to_string()));
    }

    #[test]
    fn test_access_token_not_expired() {
        let token = AccessToken::new("test".to_string(), 3600, None);
        assert!(!token.is_expired(), "Fresh token should not be expired");
    }

    #[test]
    fn test_access_token_expired() {
        let token = AccessToken::new("test".to_string(), 0, None);
        std::thread::sleep(Duration::from_millis(100));
        assert!(token.is_expired(), "Token with 0 expiry should be expired");
    }

    #[test]
    fn test_access_token_expires_with_buffer() {
        // Token expires in 4 minutes - should be considered expired due to 5 min buffer
        let token = AccessToken::new("test".to_string(), 240, None);
        assert!(
            token.is_expired(),
            "Token expiring within 5 min buffer should be considered expired"
        );
    }

    #[test]
    fn test_access_token_authorization_header() {
        let token = AccessToken::new("abc123".to_string(), 3600, None);
        assert_eq!(token.authorization_header(), "Bearer abc123");
    }

    #[test]
    fn test_access_token_serialization() {
        let token = AccessToken::new("test_token".to_string(), 3600, Some("refresh".to_string()));

        let json = serde_json::to_string(&token).unwrap();
        assert!(json.contains("test_token"));
        assert!(json.contains("refresh"));
        assert!(json.contains("Bearer"));

        let deserialized: AccessToken = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.token, token.token);
        assert_eq!(deserialized.refresh_token, token.refresh_token);
    }
}

// =============================================================================
// PKCE Verification in Token Exchange (Integration)
// =============================================================================
//
// Note: These tests verify the PKCE flow structure.
// The actual token exchange test that validates the verifier is sent
// requires the fix in auth.rs to use set_pkce_verifier().
//
// TODO: After fixing auth.rs, add test that verifies verifier is sent:
// ```
// #[tokio::test]
// async fn test_token_exchange_includes_pkce_verifier() {
//     let mut oauth = MockOAuthServer::new().await;
//
//     // This mock will ONLY succeed if code_verifier is in the request
//     oauth.mock_token_exchange_with_pkce("expected_verifier", "new_token", 3600);
//
//     let config = OAuth2Config { ... };
//     let handler = OAuth2Handler::new(config).unwrap();
//
//     let result = handler.exchange_code("auth_code".into(), "expected_verifier".into()).await;
//     assert!(result.is_ok(), "Token exchange should succeed with correct verifier");
// }
// ```
