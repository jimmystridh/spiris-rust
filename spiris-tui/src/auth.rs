// OAuth2 authentication helpers
// This module handles OAuth2 flow and token management

use anyhow::Result;
use spiris::{AccessToken, auth::{OAuth2Config, OAuth2Handler}};

/// Start OAuth2 authentication flow
pub async fn start_oauth_flow(
    client_id: String,
    client_secret: String,
    redirect_uri: String,
) -> Result<(String, String, String)> {
    let config = OAuth2Config::new(client_id, client_secret, redirect_uri);
    let handler = OAuth2Handler::new(config)?;
    Ok(handler.authorize_url())
}

/// Exchange authorization code for access token
#[allow(dead_code)]
pub async fn exchange_code(
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    code: String,
    pkce_verifier: String,
) -> Result<AccessToken> {
    let config = OAuth2Config::new(client_id, client_secret, redirect_uri);
    let handler = OAuth2Handler::new(config)?;
    Ok(handler.exchange_code(code, pkce_verifier).await?)
}

/// Refresh an expired access token
#[allow(dead_code)]
pub async fn refresh_token(
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    refresh_token: String,
) -> Result<AccessToken> {
    let config = OAuth2Config::new(client_id, client_secret, redirect_uri);
    let handler = OAuth2Handler::new(config)?;
    Ok(handler.refresh_token(refresh_token).await?)
}
