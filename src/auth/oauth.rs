//
//  bitbucket-cli
//  auth/oauth.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! # OAuth 2.0 Authentication Module
//!
//! This module implements the OAuth 2.0 authorization code flow with PKCE for Bitbucket Cloud
//! authentication. It provides secure, token-based authentication with automatic
//! refresh capabilities.
//!
//! ## OAuth Flow Overview
//!
//! The OAuth 2.0 authorization code flow with PKCE consists of the following steps:
//!
//! 1. **Generate PKCE**: Create code verifier and code challenge
//! 2. **Authorization Request**: Open browser to Bitbucket's authorization URL
//! 3. **User Consent**: User grants permissions to the application
//! 4. **Authorization Code**: Bitbucket redirects to callback URL with auth code
//! 5. **Token Exchange**: Exchange authorization code for access/refresh tokens
//! 6. **API Access**: Use access token for authenticated API requests
//! 7. **Token Refresh**: Use refresh token to obtain new access tokens
//!
//! ## Scopes
//!
//! Bitbucket supports the following OAuth scopes:
//!
//! - `repository`: Read access to repositories
//! - `repository:write`: Write access to repositories
//! - `pullrequest`: Read access to pull requests
//! - `pullrequest:write`: Write access to pull requests
//! - `account`: Read access to account information
//! - `webhook`: Manage webhooks
//! - `pipeline`: Access to pipelines
//! - `pipeline:write`: Write access to pipelines
//!
//! ## Example
//!
//! ```rust,no_run
//! use bitbucket_cli::auth::{OAuthConfig, oauth_login};
//!
//! async fn authenticate() -> anyhow::Result<()> {
//!     let config = OAuthConfig {
//!         client_id: "your_client_id".to_string(),
//!         client_secret: Some("your_client_secret".to_string()),
//!         ..Default::default()
//!     };
//!
//!     let tokens = oauth_login(&config).await?;
//!     println!("Access token: {}", tokens.access_token);
//!     Ok(())
//! }
//! ```
//!
//! ## Security Considerations
//!
//! - PKCE (Proof Key for Code Exchange) is used for all OAuth flows
//! - Store client secrets securely; never commit them to version control
//! - Always use HTTPS for the redirect URI in production
//! - Store refresh tokens securely using the system keyring

use anyhow::{Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::RngCore;
use reqwest::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::time::Duration;
use url::Url;

/// The default OAuth client ID for the Bitbucket CLI.
/// Users can override this with their own OAuth consumer.
pub const DEFAULT_CLIENT_ID: &str = "Pyydmsf5kLpEqs24kw";

/// Bitbucket Cloud OAuth authorization endpoint.
const AUTHORIZE_URL: &str = "https://bitbucket.org/site/oauth2/authorize";

/// Bitbucket Cloud OAuth token endpoint.
const TOKEN_URL: &str = "https://bitbucket.org/site/oauth2/access_token";

/// Configuration for OAuth 2.0 authentication with Bitbucket Cloud.
///
/// This struct holds all the parameters required to initiate and complete
/// the OAuth 2.0 authorization code flow with PKCE.
///
/// # Fields
///
/// - `client_id`: The OAuth application's client ID from Bitbucket settings.
/// - `client_secret`: Optional client secret for confidential clients.
/// - `redirect_uri`: The callback URL where Bitbucket sends the authorization code.
/// - `scopes`: List of permission scopes to request from the user.
///
/// # Example
///
/// ```rust
/// use bitbucket_cli::auth::OAuthConfig;
///
/// // Create config with custom scopes
/// let config = OAuthConfig {
///     client_id: "your_app_client_id".to_string(),
///     client_secret: Some("your_client_secret".to_string()),
///     redirect_uri: "http://localhost:8085/callback".to_string(),
///     scopes: vec![
///         "repository".to_string(),
///         "repository:write".to_string(),
///         "pullrequest".to_string(),
///     ],
/// };
///
/// // Or use defaults with just the client ID
/// let config = OAuthConfig {
///     client_id: "your_app_client_id".to_string(),
///     ..Default::default()
/// };
/// ```
///
/// # Notes
///
/// - `client_secret` is optional for public clients (native/mobile apps).
/// - The default redirect URI uses port 8085 on localhost.
/// - Default scopes provide comprehensive access for CLI operations.
pub struct OAuthConfig {
    /// The OAuth 2.0 client identifier for this application.
    ///
    /// Obtain this from Bitbucket's OAuth consumer settings at:
    /// `https://bitbucket.org/{workspace}/workspace/settings/api`
    pub client_id: String,

    /// The OAuth 2.0 client secret for confidential clients.
    ///
    /// This should be `None` for public clients (CLI tools, native apps).
    /// When using PKCE, the client secret is not required.
    pub client_secret: Option<String>,

    /// The redirect URI where Bitbucket will send the authorization code.
    ///
    /// This must exactly match one of the callback URLs registered in
    /// the Bitbucket OAuth consumer settings.
    pub redirect_uri: String,

    /// The list of OAuth scopes to request from the user.
    ///
    /// Each scope grants access to specific Bitbucket resources.
    /// Request only the minimum scopes necessary for your use case.
    pub scopes: Vec<String>,
}

impl Default for OAuthConfig {
    /// Creates a default OAuth configuration with common settings.
    ///
    /// # Default Values
    ///
    /// - `client_id`: Empty string (must be set before use)
    /// - `client_secret`: None
    /// - `redirect_uri`: `http://localhost:8085/callback`
    /// - `scopes`: Comprehensive scopes for CLI operations
    ///
    /// # Example
    ///
    /// ```rust
    /// use bitbucket_cli::auth::OAuthConfig;
    ///
    /// let mut config = OAuthConfig::default();
    /// config.client_id = "your_client_id".to_string();
    /// ```
    fn default() -> Self {
        Self {
            client_id: String::new(),
            client_secret: None,
            redirect_uri: "http://localhost:8085/callback".to_string(),
            scopes: vec![
                "repository".to_string(),
                "repository:write".to_string(),
                "pullrequest".to_string(),
                "pullrequest:write".to_string(),
                "account".to_string(),
                "pipeline".to_string(),
                "pipeline:write".to_string(),
                "webhook".to_string(),
            ],
        }
    }
}

/// Represents the response from Bitbucket's OAuth token endpoint.
///
/// This struct contains all the tokens and metadata returned after a successful
/// authorization code exchange or token refresh operation.
///
/// # Fields
///
/// - `access_token`: The bearer token for API authentication.
/// - `refresh_token`: Optional token for obtaining new access tokens.
/// - `token_type`: The type of token (typically "bearer").
/// - `expires_in`: Optional token lifetime in seconds.
/// - `scopes`: The actual scopes granted by the user.
///
/// # Example
///
/// ```rust
/// use bitbucket_cli::auth::OAuthTokenResponse;
///
/// // Parse response from Bitbucket
/// let response = OAuthTokenResponse {
///     access_token: "access_token_here".to_string(),
///     refresh_token: Some("refresh_token_here".to_string()),
///     token_type: "bearer".to_string(),
///     expires_in: Some(7200), // 2 hours
///     scopes: vec!["repository".to_string(), "account".to_string()],
/// };
///
/// // Calculate expiration time
/// if let Some(expires_in) = response.expires_in {
///     println!("Token expires in {} seconds", expires_in);
/// }
/// ```
///
/// # Notes
///
/// - The `scopes` returned may differ from those requested if the user
///   denied some permissions.
/// - Store the `refresh_token` securely for future token renewals.
/// - `expires_in` is typically 7200 seconds (2 hours) for Bitbucket.
#[derive(Debug, Clone)]
pub struct OAuthTokenResponse {
    /// The OAuth 2.0 access token for authenticating API requests.
    ///
    /// This token should be included in the Authorization header as:
    /// `Authorization: Bearer {access_token}`
    pub access_token: String,

    /// The refresh token for obtaining new access tokens without re-authorization.
    ///
    /// This is `None` if the authorization server doesn't support refresh tokens
    /// or if the `offline_access` scope wasn't requested.
    pub refresh_token: Option<String>,

    /// The type of token issued, typically "bearer".
    pub token_type: String,

    /// The lifetime of the access token in seconds.
    ///
    /// If `None`, the token doesn't expire or the expiration is unknown.
    pub expires_in: Option<u64>,

    /// The list of scopes actually granted by the user.
    ///
    /// This may be a subset of the scopes requested if the user
    /// denied certain permissions during authorization.
    pub scopes: Vec<String>,
}

/// Internal struct for deserializing token responses from Bitbucket.
#[derive(Deserialize)]
struct TokenResponseRaw {
    access_token: String,
    refresh_token: Option<String>,
    token_type: String,
    expires_in: Option<u64>,
    scopes: Option<String>,
}

impl From<TokenResponseRaw> for OAuthTokenResponse {
    fn from(raw: TokenResponseRaw) -> Self {
        let scopes = raw
            .scopes
            .map(|s| s.split_whitespace().map(String::from).collect())
            .unwrap_or_default();

        Self {
            access_token: raw.access_token,
            refresh_token: raw.refresh_token,
            token_type: raw.token_type,
            expires_in: raw.expires_in,
            scopes,
        }
    }
}

/// PKCE (Proof Key for Code Exchange) challenge data.
///
/// PKCE is used to secure the OAuth flow for public clients (like CLI tools)
/// that cannot securely store a client secret.
struct PkceChallenge {
    /// The code verifier - a high-entropy cryptographic random string.
    verifier: String,
    /// The code challenge - SHA256 hash of the verifier, base64url encoded.
    challenge: String,
}

impl PkceChallenge {
    /// Generates a new PKCE challenge.
    ///
    /// Creates a cryptographically secure random code verifier and
    /// computes the corresponding code challenge using S256 method.
    fn new() -> Self {
        // Generate 32 bytes of random data for the verifier
        let mut verifier_bytes = [0u8; 32];
        rand::rng().fill_bytes(&mut verifier_bytes);
        let verifier = URL_SAFE_NO_PAD.encode(verifier_bytes);

        // Compute SHA256 hash and base64url encode it
        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        let challenge = URL_SAFE_NO_PAD.encode(hasher.finalize());

        Self {
            verifier,
            challenge,
        }
    }
}

/// Initiates and completes the OAuth 2.0 authorization code flow with PKCE.
///
/// This function performs the complete OAuth login process:
/// 1. Generates PKCE code verifier and challenge
/// 2. Starts a local HTTP server to receive the callback
/// 3. Opens the user's browser to Bitbucket's authorization page
/// 4. Waits for the user to grant permissions
/// 5. Receives the authorization code via callback
/// 6. Exchanges the code for access and refresh tokens
///
/// # Parameters
///
/// - `config`: The [`OAuthConfig`] containing client credentials and settings.
///
/// # Returns
///
/// Returns `Ok(OAuthTokenResponse)` containing the access token, refresh token,
/// and other metadata on success.
///
/// Returns `Err` if:
/// - The local server fails to start
/// - The browser cannot be opened
/// - The user denies authorization
/// - The token exchange fails
/// - Network errors occur
///
/// # Example
///
/// ```rust,no_run
/// use bitbucket_cli::auth::{OAuthConfig, oauth_login};
///
/// async fn login() -> anyhow::Result<()> {
///     let config = OAuthConfig {
///         client_id: "your_client_id".to_string(),
///         ..Default::default()
///     };
///
///     println!("Opening browser for authentication...");
///     let tokens = oauth_login(&config).await?;
///
///     println!("Successfully authenticated!");
///     println!("Access token: {}...", &tokens.access_token[..20.min(tokens.access_token.len())]);
///
///     if tokens.refresh_token.is_some() {
///         println!("Refresh token obtained for future renewals");
///     }
///
///     Ok(())
/// }
/// ```
///
/// # Notes
///
/// - The function will block until the user completes or cancels authorization.
/// - Ensure the redirect URI port (default: 8085) is available.
/// - The callback server runs on localhost and doesn't require internet access.
pub async fn oauth_login(config: &OAuthConfig) -> Result<OAuthTokenResponse> {
    // Generate PKCE challenge
    let pkce = PkceChallenge::new();

    // Parse redirect URI to extract port
    let redirect_url = Url::parse(&config.redirect_uri).context("Invalid redirect URI")?;
    let port = redirect_url.port().unwrap_or(8085);

    // Build authorization URL
    let mut auth_url = Url::parse(AUTHORIZE_URL)?;
    auth_url
        .query_pairs_mut()
        .append_pair("client_id", &config.client_id)
        .append_pair("response_type", "code")
        .append_pair("redirect_uri", &config.redirect_uri)
        .append_pair("code_challenge", &pkce.challenge)
        .append_pair("code_challenge_method", "S256");

    if !config.scopes.is_empty() {
        auth_url
            .query_pairs_mut()
            .append_pair("scope", &config.scopes.join(" "));
    }

    // Create channel to receive the authorization code
    let (tx, rx) = mpsc::channel::<String>();

    // Start local server in a background thread
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).context(format!(
        "Failed to bind to port {}. Is another process using it?",
        port
    ))?;
    listener.set_nonblocking(false)?;

    // Spawn thread to handle the callback
    let _redirect_uri = config.redirect_uri.clone();
    std::thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let buf_reader = BufReader::new(&stream);
            if let Some(Ok(request_line)) = buf_reader.lines().next() {
                // Parse the authorization code from the request
                if let Some(code) = extract_code_from_request(&request_line) {
                    let _ = tx.send(code);

                    // Send success response
                    let response = "HTTP/1.1 200 OK\r\n\
                         Content-Type: text/html\r\n\
                         Connection: close\r\n\
                         \r\n\
                         <!DOCTYPE html>\
                         <html><head><title>Authentication Successful</title></head>\
                         <body style=\"font-family: system-ui, sans-serif; text-align: center; padding: 50px;\">\
                         <h1>Authentication Successful!</h1>\
                         <p>You can close this window and return to the terminal.</p>\
                         <script>window.close();</script>\
                         </body></html>";
                    let _ = stream.write_all(response.as_bytes());
                } else {
                    // Check for error
                    if request_line.contains("error=") {
                        let response = "HTTP/1.1 400 Bad Request\r\n\
                             Content-Type: text/html\r\n\
                             Connection: close\r\n\
                             \r\n\
                             <!DOCTYPE html>\
                             <html><head><title>Authentication Failed</title></head>\
                             <body style=\"font-family: system-ui, sans-serif; text-align: center; padding: 50px;\">\
                             <h1>Authentication Failed</h1>\
                             <p>The authorization was denied or an error occurred.</p>\
                             <p>You can close this window and try again.</p>\
                             </body></html>";
                        let _ = stream.write_all(response.as_bytes());
                    }
                }
            }
        }
        drop(listener);
    });

    // Open browser
    println!("Opening browser for authentication...");
    println!("If the browser doesn't open, visit this URL:");
    println!("{}", auth_url);
    println!();

    if let Err(e) = webbrowser::open(auth_url.as_str()) {
        eprintln!("Warning: Could not open browser: {}", e);
        eprintln!("Please manually visit the URL above.");
    }

    // Wait for authorization code (with timeout)
    println!("Waiting for authorization...");
    let code = rx
        .recv_timeout(Duration::from_secs(300))
        .context("Authorization timed out. Please try again.")?;

    println!("Authorization received. Exchanging code for tokens...");

    // Exchange code for tokens
    exchange_code_for_tokens(config, &code, &pkce.verifier).await
}

/// Extracts the authorization code from an HTTP request line.
fn extract_code_from_request(request_line: &str) -> Option<String> {
    // Request line format: GET /callback?code=xxx HTTP/1.1
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }

    let path = parts[1];
    if let Some(query_start) = path.find('?') {
        let query = &path[query_start + 1..];
        for param in query.split('&') {
            if let Some((key, value)) = param.split_once('=') {
                if key == "code" {
                    return Some(value.to_string());
                }
            }
        }
    }
    None
}

/// Exchanges an authorization code for access and refresh tokens.
async fn exchange_code_for_tokens(
    config: &OAuthConfig,
    code: &str,
    code_verifier: &str,
) -> Result<OAuthTokenResponse> {
    let client = Client::new();

    let mut params = vec![
        ("grant_type", "authorization_code"),
        ("code", code),
        ("redirect_uri", &config.redirect_uri),
        ("code_verifier", code_verifier),
    ];

    let mut request = client.post(TOKEN_URL);

    // If we have a client secret, use Basic auth; otherwise include client_id in body
    if let Some(ref secret) = config.client_secret {
        request = request.basic_auth(&config.client_id, Some(secret));
    } else {
        params.push(("client_id", &config.client_id));
    }

    let response = request
        .form(&params)
        .send()
        .await
        .context("Failed to exchange authorization code")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("Token exchange failed ({}): {}", status, body);
    }

    let token_response: TokenResponseRaw = response
        .json()
        .await
        .context("Failed to parse token response")?;

    Ok(token_response.into())
}

/// Refreshes an OAuth 2.0 access token using a refresh token.
///
/// When an access token expires, this function can be used to obtain a new one
/// without requiring the user to re-authenticate. The refresh token is exchanged
/// for a new access token (and potentially a new refresh token).
///
/// # Parameters
///
/// - `refresh_token`: The refresh token obtained from a previous authorization.
/// - `client_id`: Optional client ID. If None, uses the default CLI client ID.
/// - `client_secret`: Optional client secret for confidential clients.
///
/// # Returns
///
/// Returns `Ok(OAuthTokenResponse)` containing new tokens on success.
///
/// Returns `Err` if:
/// - The refresh token is invalid or expired
/// - The refresh token has been revoked
/// - Network errors occur
/// - The authorization server is unavailable
///
/// # Example
///
/// ```rust,no_run
/// use bitbucket_cli::auth::refresh_oauth_token;
///
/// async fn refresh_credentials(refresh_token: &str) -> anyhow::Result<()> {
///     let new_tokens = refresh_oauth_token(refresh_token, None, None).await?;
///     println!("New access token obtained!");
///     Ok(())
/// }
/// ```
///
/// # Notes
///
/// - Some OAuth servers issue a new refresh token with each refresh; always
///   store the latest refresh token from the response.
/// - Refresh tokens may have a limited number of uses or a maximum lifetime.
/// - If refresh fails, the user must re-authenticate via the full OAuth flow.
pub async fn refresh_oauth_token(
    refresh_token: &str,
    client_id: Option<&str>,
    client_secret: Option<&str>,
) -> Result<OAuthTokenResponse> {
    let client = Client::new();
    let client_id = client_id.unwrap_or(DEFAULT_CLIENT_ID);

    let params = [
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
    ];

    let mut request = client.post(TOKEN_URL);

    if let Some(secret) = client_secret {
        request = request.basic_auth(client_id, Some(secret));
    } else {
        request = request.form(&[("client_id", client_id)]);
    }

    let response = request
        .form(&params)
        .send()
        .await
        .context("Failed to refresh token")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("Token refresh failed ({}): {}", status, body);
    }

    let token_response: TokenResponseRaw = response
        .json()
        .await
        .context("Failed to parse token response")?;

    Ok(token_response.into())
}

/// Legacy function for backward compatibility.
/// Use [`refresh_oauth_token`] instead.
pub async fn refresh_token(refresh_token: &str) -> Result<OAuthTokenResponse> {
    refresh_oauth_token(refresh_token, None, None).await
}
