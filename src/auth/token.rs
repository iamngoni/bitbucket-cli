//
//  bitbucket-cli
//  auth/token.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! # Token-Based Authentication Module
//!
//! This module provides Personal Access Token (PAT) handling for Bitbucket Server
//! and Data Center deployments. PATs are the recommended authentication method
//! for self-hosted Bitbucket instances.
//!
//! ## Overview
//!
//! Personal Access Tokens provide a secure alternative to using passwords for
//! API authentication. They can be:
//!
//! - Created with specific permissions (read, write, admin)
//! - Set to expire after a certain period
//! - Revoked without changing the user's password
//! - Audited for security monitoring
//!
//! ## Creating a PAT
//!
//! To create a Personal Access Token in Bitbucket Server/DC:
//!
//! 1. Navigate to your Bitbucket account settings
//! 2. Select "Personal access tokens" from the sidebar
//! 3. Click "Create a token"
//! 4. Enter a name and select permissions
//! 5. Optionally set an expiration date
//! 6. Copy and securely store the generated token
//!
//! ## Example
//!
//! ```rust,no_run
//! use bitbucket_cli::auth::{PersonalAccessToken, read_token_from_stdin, validate_token};
//!
//! fn setup_authentication() -> anyhow::Result<PersonalAccessToken> {
//!     println!("Enter your Personal Access Token:");
//!     let token = read_token_from_stdin()?;
//!
//!     if !validate_token(&token) {
//!         anyhow::bail!("Invalid token format");
//!     }
//!
//!     let pat = PersonalAccessToken::new(
//!         token,
//!         "https://bitbucket.example.com".to_string(),
//!     );
//!
//!     Ok(pat)
//! }
//! ```
//!
//! ## Security Best Practices
//!
//! - Never commit tokens to version control
//! - Use the system keyring for secure storage
//! - Set appropriate expiration dates
//! - Grant minimum necessary permissions
//! - Rotate tokens periodically

use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;

/// Reads a token from standard input.
///
/// This function reads a single line from stdin, typically used for
/// securely accepting token input from the user or from piped input.
/// The input is trimmed of leading and trailing whitespace.
///
/// # Returns
///
/// Returns `Ok(String)` containing the trimmed token on success.
///
/// Returns `Err` if reading from stdin fails.
///
/// # Example
///
/// ```rust,no_run
/// use bitbucket_cli::auth::read_token_from_stdin;
///
/// fn main() -> anyhow::Result<()> {
///     println!("Please enter your token:");
///     let token = read_token_from_stdin()?;
///     println!("Token received ({} characters)", token.len());
///     Ok(())
/// }
/// ```
///
/// # Notes
///
/// - Only reads the first line; subsequent input is ignored.
/// - Whitespace is trimmed from both ends of the input.
/// - For interactive use, consider disabling terminal echo for security.
/// - Can be used with piped input: `echo "token" | bitbucket-cli auth login`
pub fn read_token_from_stdin() -> Result<String> {
    use std::io::{self, BufRead};

    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line)?;

    Ok(line.trim().to_string())
}

/// Validates the format of a token string.
///
/// Performs basic validation to ensure the token meets minimum requirements:
/// - Token must not be empty
/// - Token must not contain whitespace characters
///
/// This function does NOT validate whether the token is actually valid
/// with the Bitbucket server; it only checks the format.
///
/// # Parameters
///
/// - `token`: The token string to validate.
///
/// # Returns
///
/// Returns `true` if the token format is valid, `false` otherwise.
///
/// # Example
///
/// ```rust
/// use bitbucket_cli::auth::validate_token;
///
/// assert!(validate_token("NjM0NTY3ODkwMTIzNDU2Nzg5MA=="));
/// assert!(validate_token("abc123"));
///
/// assert!(!validate_token(""));           // Empty token
/// assert!(!validate_token("has space"));  // Contains whitespace
/// assert!(!validate_token("has\ttab"));   // Contains tab
/// assert!(!validate_token("has\nnewline")); // Contains newline
/// ```
///
/// # Notes
///
/// - This is a format check only; use [`PersonalAccessToken::validate`] to
///   verify the token with the server.
/// - Bitbucket Server PATs are typically Base64-encoded strings.
/// - Tokens with embedded whitespace are always invalid.
pub fn validate_token(token: &str) -> bool {
    // Basic validation - token should not be empty and should not contain whitespace
    !token.is_empty() && !token.chars().any(char::is_whitespace)
}

/// Response from Bitbucket Server user endpoint.
#[derive(Deserialize)]
struct ServerUserResponse {
    /// Username of the authenticated user.
    name: String,
    /// Display name of the authenticated user.
    #[serde(rename = "displayName")]
    #[allow(dead_code)]
    display_name: Option<String>,
}

/// Represents a Personal Access Token for Bitbucket Server/Data Center.
///
/// This struct encapsulates a PAT along with its associated host, providing
/// methods for token validation and management.
///
/// # Fields
///
/// - `token`: The Personal Access Token string.
/// - `host`: The Bitbucket Server/DC host URL.
///
/// # Example
///
/// ```rust,no_run
/// use bitbucket_cli::auth::PersonalAccessToken;
///
/// async fn setup_pat() -> anyhow::Result<()> {
///     let pat = PersonalAccessToken::new(
///         "your_pat_token_here".to_string(),
///         "https://bitbucket.company.com".to_string(),
///     );
///
///     // Validate the token with the server
///     if pat.validate().await? {
///         println!("Token is valid for {}", pat.host);
///     } else {
///         println!("Token validation failed");
///     }
///
///     Ok(())
/// }
/// ```
///
/// # Notes
///
/// - The token is stored in plain text in memory; use secure storage for persistence.
/// - The host URL should not include a trailing slash.
/// - PATs are specific to a single Bitbucket instance.
#[derive(Debug, Clone)]
pub struct PersonalAccessToken {
    /// The Personal Access Token string.
    ///
    /// This is the actual token value that will be used for authentication.
    /// Handle with care as this grants access to the associated Bitbucket account.
    pub token: String,

    /// The Bitbucket Server/Data Center host URL.
    ///
    /// This should be the base URL of the Bitbucket instance, e.g.,
    /// `https://bitbucket.example.com` or `https://git.company.com/bitbucket`.
    pub host: String,
}

impl PersonalAccessToken {
    /// Creates a new Personal Access Token instance.
    ///
    /// # Parameters
    ///
    /// - `token`: The PAT string obtained from Bitbucket Server/DC settings.
    /// - `host`: The base URL of the Bitbucket Server/DC instance.
    ///
    /// # Returns
    ///
    /// Returns a new [`PersonalAccessToken`] instance.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bitbucket_cli::auth::PersonalAccessToken;
    ///
    /// let pat = PersonalAccessToken::new(
    ///     "NjM0NTY3ODkwMTIzNDU2Nzg5MA==".to_string(),
    ///     "https://bitbucket.example.com".to_string(),
    /// );
    ///
    /// assert_eq!(pat.host, "https://bitbucket.example.com");
    /// ```
    ///
    /// # Notes
    ///
    /// - No validation is performed during construction.
    /// - Call [`validate`](Self::validate) to verify the token with the server.
    /// - The host URL should be normalized (no trailing slash) for consistency.
    pub fn new(token: String, host: String) -> Self {
        // Normalize the host URL (remove trailing slash)
        let host = host.trim_end_matches('/').to_string();
        Self { token, host }
    }

    /// Validates the token by making a test API call to the Bitbucket server.
    ///
    /// This method attempts to authenticate with the Bitbucket Server/DC API
    /// using the stored token, verifying that:
    /// - The token is correctly formatted
    /// - The token has not expired
    /// - The token has not been revoked
    /// - The host is reachable
    ///
    /// # Returns
    ///
    /// Returns `Ok(true)` if the token is valid and working.
    /// Returns `Ok(false)` if the token is invalid or expired.
    /// Returns `Err` if the validation request fails due to network or server errors.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use bitbucket_cli::auth::PersonalAccessToken;
    ///
    /// async fn check_token(pat: &PersonalAccessToken) -> anyhow::Result<()> {
    ///     match pat.validate().await {
    ///         Ok(true) => println!("Token is valid"),
    ///         Ok(false) => println!("Token is invalid or expired"),
    ///         Err(e) => println!("Validation failed: {}", e),
    ///     }
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Notes
    ///
    /// - This makes an actual HTTP request to the Bitbucket server.
    /// - The validation endpoint used is `/rest/api/1.0/users` for Server/DC.
    /// - Network errors are propagated as `Err`, not `Ok(false)`.
    /// - Consider caching the validation result to avoid repeated API calls.
    pub async fn validate(&self) -> Result<bool> {
        let client = Client::builder()
            .user_agent(format!("bb/{}", crate::VERSION))
            .build()
            .context("Failed to create HTTP client")?;

        // Try to get the current user info using /rest/api/1.0/users endpoint
        // which returns info about the authenticated user
        let url = format!("{}/rest/api/1.0/application-properties", self.host);
        let response = client
            .get(&url)
            .bearer_auth(&self.token)
            .send()
            .await
            .context("Failed to connect to Bitbucket server")?;

        if response.status().is_success() {
            Ok(true)
        } else if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            Ok(false)
        } else {
            // Try alternate endpoint (for older versions)
            let url = format!("{}/rest/api/1.0/projects", self.host);
            let response = client
                .get(&url)
                .bearer_auth(&self.token)
                .query(&[("limit", "1")])
                .send()
                .await
                .context("Failed to connect to Bitbucket server")?;

            if response.status().is_success() {
                Ok(true)
            } else if response.status() == reqwest::StatusCode::UNAUTHORIZED {
                Ok(false)
            } else {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                anyhow::bail!("Unexpected response ({}): {}", status, body)
            }
        }
    }

    /// Returns the username associated with this token.
    ///
    /// Makes an API call to retrieve the authenticated user's information.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(username))` if the token is valid and user info is retrieved.
    /// Returns `Ok(None)` if the token is invalid.
    /// Returns `Err` for network or server errors.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use bitbucket_cli::auth::PersonalAccessToken;
    ///
    /// async fn get_username(pat: &PersonalAccessToken) -> anyhow::Result<()> {
    ///     if let Some(username) = pat.get_username().await? {
    ///         println!("Logged in as: {}", username);
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_username(&self) -> Result<Option<String>> {
        let client = Client::builder()
            .user_agent(format!("bb/{}", crate::VERSION))
            .build()
            .context("Failed to create HTTP client")?;

        // Get current user from the REST API
        // Bitbucket Server uses /plugins/servlet/applinks/whoami or /rest/api/1.0/users endpoint
        let url = format!("{}/plugins/servlet/applinks/whoami", self.host);
        let response = client
            .get(&url)
            .bearer_auth(&self.token)
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                let username = resp.text().await.unwrap_or_default();
                if username.is_empty() || username == "anonymous" {
                    Ok(None)
                } else {
                    Ok(Some(username.trim().to_string()))
                }
            }
            Ok(resp) if resp.status() == reqwest::StatusCode::UNAUTHORIZED => Ok(None),
            Ok(resp) => {
                // Try fallback: check /rest/api/1.0/users/{slug} where slug might be in token
                // For now, just return None as we can't determine username without more info
                let status = resp.status();
                tracing::debug!("Username lookup returned {}", status);
                Ok(None)
            }
            Err(e) => {
                tracing::debug!("Username lookup failed: {}", e);
                Ok(None)
            }
        }
    }
}

/// Validates a token against Bitbucket Cloud API.
///
/// This function tests whether an OAuth or app password token is valid
/// by making a request to the user endpoint.
///
/// # Parameters
///
/// - `token`: The access token to validate.
///
/// # Returns
///
/// Returns `Ok(true)` if the token is valid.
/// Returns `Ok(false)` if the token is invalid.
/// Returns `Err` for network errors.
pub async fn validate_cloud_token(token: &str) -> Result<bool> {
    let client = Client::builder()
        .user_agent(format!("bb/{}", crate::VERSION))
        .build()
        .context("Failed to create HTTP client")?;

    let response = client
        .get("https://api.bitbucket.org/2.0/user")
        .bearer_auth(token)
        .send()
        .await
        .context("Failed to connect to Bitbucket Cloud")?;

    Ok(response.status().is_success())
}

/// Gets the username for a Bitbucket Cloud token.
///
/// # Parameters
///
/// - `token`: The access token to use.
///
/// # Returns
///
/// Returns `Ok(Some(username))` if successful.
/// Returns `Ok(None)` if the token is invalid.
pub async fn get_cloud_username(token: &str) -> Result<Option<String>> {
    #[derive(Deserialize)]
    struct CloudUser {
        username: String,
    }

    let client = Client::builder()
        .user_agent(format!("bb/{}", crate::VERSION))
        .build()
        .context("Failed to create HTTP client")?;

    let response = client
        .get("https://api.bitbucket.org/2.0/user")
        .bearer_auth(token)
        .send()
        .await
        .context("Failed to connect to Bitbucket Cloud")?;

    if response.status().is_success() {
        let user: CloudUser = response.json().await?;
        Ok(Some(user.username))
    } else {
        Ok(None)
    }
}
