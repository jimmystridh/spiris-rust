//! Example: OAuth2 authentication flow
//!
//! This example demonstrates how to authenticate with the Visma eAccounting API
//! using OAuth2.
//!
//! Run with:
//! ```
//! cargo run --example oauth_flow
//! ```

use visma_eaccounting::auth::{OAuth2Config, OAuth2Handler};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get OAuth2 credentials from environment variables
    let client_id =
        std::env::var("VISMA_CLIENT_ID").expect("Please set VISMA_CLIENT_ID environment variable");
    let client_secret = std::env::var("VISMA_CLIENT_SECRET")
        .expect("Please set VISMA_CLIENT_SECRET environment variable");
    let redirect_uri = std::env::var("VISMA_REDIRECT_URI")
        .unwrap_or_else(|_| "http://localhost:8080/callback".to_string());

    // Create OAuth2 configuration
    let config = OAuth2Config::new(client_id, client_secret, redirect_uri);

    // Create OAuth2 handler
    let handler = OAuth2Handler::new(config)?;

    // Generate authorization URL
    let (auth_url, csrf_token, pkce_verifier) = handler.authorize_url();

    println!("OAuth2 Authorization Flow");
    println!("{:=<80}", "");
    println!("\n1. Visit this URL to authorize the application:\n");
    println!("{}\n", auth_url);
    println!("2. After authorizing, you will be redirected to your redirect URI");
    println!("   with a 'code' parameter in the URL.\n");
    println!("3. Extract the code from the URL and use it to exchange for an access token.\n");
    println!("CSRF Token (verify this matches): {}", csrf_token);
    println!("\nPKCE Verifier (save this): {}\n", pkce_verifier);

    println!("Example code to exchange authorization code for token:");
    println!("{:-<80}", "");
    println!("let token = handler.exchange_code(code, pkce_verifier).await?;");
    println!("let client = Client::new(token);");
    println!("{:-<80}", "");

    Ok(())
}
