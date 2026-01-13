//
//  bitbucket-cli
//  auth/mod.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! # Authentication Module
//!
//! This module provides comprehensive authentication management for the Bitbucket CLI,
//! supporting multiple authentication methods for both Bitbucket Cloud and Server/Data Center
//! deployments.
//!
//! ## Supported Authentication Methods
//!
//! - **OAuth 2.0**: Recommended for Bitbucket Cloud, provides secure token-based authentication
//!   with automatic refresh capabilities.
//! - **App Password**: Legacy authentication method for Bitbucket Cloud (deprecated, use OAuth instead).
//! - **Personal Access Token (PAT)**: Primary authentication method for Bitbucket Server/Data Center.
//! - **Basic Authentication**: Username/password authentication for backward compatibility.
//!
//! ## Module Structure
//!
//! - [`oauth`]: OAuth 2.0 authentication flow implementation
//! - [`token`]: Personal Access Token handling and validation
//! - [`keyring`]: Secure credential storage using system keyring
//! - [`profile`]: Authentication profile management for multiple accounts
//!
//! ## Example
//!
//! ```rust,no_run
//! use bitbucket_cli::auth::{AuthCredential, ProfileManager, KeyringStore};
//!
//! // Create OAuth credentials
//! let credential = AuthCredential::OAuth {
//!     access_token: "your_access_token".to_string(),
//!     refresh_token: Some("your_refresh_token".to_string()),
//!     expires_at: None,
//! };
//!
//! // Check if credential needs refresh
//! if credential.is_expired() && credential.can_refresh() {
//!     // Perform token refresh
//! }
//! ```

mod keyring;
mod oauth;
mod profile;
mod token;

pub use keyring::*;
pub use oauth::*;
pub use profile::*;
pub use token::*;

use reqwest::RequestBuilder;

/// Represents different types of authentication credentials supported by the Bitbucket CLI.
///
/// This enum encapsulates all authentication methods available for connecting to
/// Bitbucket Cloud and Server/Data Center instances. Each variant contains the
/// necessary data for its specific authentication mechanism.
///
/// # Variants
///
/// - `OAuth`: OAuth 2.0 authentication with access token, optional refresh token,
///   and expiration tracking. Recommended for Bitbucket Cloud.
/// - `AppPassword`: Username and app password combination. Deprecated for Cloud.
/// - `PersonalAccessToken`: Bearer token authentication for Server/DC deployments.
/// - `Basic`: Standard HTTP Basic authentication with username and password.
///
/// # Example
///
/// ```rust
/// use bitbucket_cli::auth::AuthCredential;
/// use chrono::{Utc, Duration};
///
/// // Create an OAuth credential with expiration
/// let oauth_cred = AuthCredential::OAuth {
///     access_token: "eyJhbGciOiJIUzI1NiIs...".to_string(),
///     refresh_token: Some("refresh_token_here".to_string()),
///     expires_at: Some(Utc::now() + Duration::hours(1)),
/// };
///
/// // Create a PAT credential for Server/DC
/// let pat_cred = AuthCredential::PersonalAccessToken {
///     token: "NjM0NTY3ODkw...".to_string(),
/// };
/// ```
///
/// # Notes
///
/// - OAuth credentials should always include a refresh token for uninterrupted access.
/// - AppPassword is deprecated; migrate to OAuth for Bitbucket Cloud.
/// - PAT is the recommended method for Bitbucket Server/Data Center.
#[derive(Debug, Clone)]
pub enum AuthCredential {
    /// OAuth 2.0 token authentication for Bitbucket Cloud.
    ///
    /// This is the recommended authentication method for Cloud deployments,
    /// providing secure, scoped access with automatic token refresh capabilities.
    OAuth {
        /// The OAuth 2.0 access token used for API authentication.
        access_token: String,
        /// Optional refresh token for obtaining new access tokens without re-authentication.
        refresh_token: Option<String>,
        /// Optional expiration timestamp for the access token.
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
    },
    /// App password authentication for Bitbucket Cloud (deprecated).
    ///
    /// App passwords are being phased out in favor of OAuth 2.0.
    /// Consider migrating to OAuth for better security and granular permissions.
    AppPassword {
        /// The Bitbucket username (Atlassian account email or username).
        username: String,
        /// The app password generated from Bitbucket settings.
        password: String,
    },
    /// Personal Access Token for Bitbucket Server/Data Center.
    ///
    /// PATs provide a secure way to authenticate with self-hosted Bitbucket instances
    /// without exposing user passwords.
    PersonalAccessToken {
        /// The personal access token string.
        token: String,
    },
    /// Basic HTTP authentication with username and password.
    ///
    /// Use this method for backward compatibility or when other methods
    /// are not available. Not recommended for production use.
    Basic {
        /// The username for authentication.
        username: String,
        /// The password for authentication.
        password: String,
    },
}

impl AuthCredential {
    /// Applies the authentication credential to an HTTP request.
    ///
    /// This method adds the appropriate authentication headers to the given
    /// request builder based on the credential type:
    /// - OAuth and PAT use Bearer token authentication
    /// - AppPassword and Basic use HTTP Basic authentication
    ///
    /// # Parameters
    ///
    /// - `request`: The [`RequestBuilder`] to add authentication headers to.
    ///
    /// # Returns
    ///
    /// Returns the modified [`RequestBuilder`] with authentication headers applied.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use bitbucket_cli::auth::AuthCredential;
    /// use reqwest::Client;
    ///
    /// async fn make_authenticated_request(credential: &AuthCredential) {
    ///     let client = Client::new();
    ///     let request = client.get("https://api.bitbucket.org/2.0/user");
    ///     let authenticated_request = credential.apply_to_request(request);
    ///     let response = authenticated_request.send().await;
    /// }
    /// ```
    ///
    /// # Notes
    ///
    /// - For OAuth credentials, only the access token is used; expiration is not checked.
    /// - Call [`is_expired`](Self::is_expired) before making requests to ensure token validity.
    pub fn apply_to_request(&self, request: RequestBuilder) -> RequestBuilder {
        match self {
            Self::OAuth { access_token, .. } => request.bearer_auth(access_token),
            Self::AppPassword { username, password } => {
                request.basic_auth(username, Some(password))
            }
            Self::PersonalAccessToken { token } => request.bearer_auth(token),
            Self::Basic { username, password } => request.basic_auth(username, Some(password)),
        }
    }

    /// Checks if the credential has expired.
    ///
    /// This method determines whether the credential is no longer valid based on
    /// its expiration time. Only OAuth credentials with an explicit `expires_at`
    /// timestamp can expire; all other credential types always return `false`.
    ///
    /// # Returns
    ///
    /// Returns `true` if the credential has expired, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bitbucket_cli::auth::AuthCredential;
    /// use chrono::{Utc, Duration};
    ///
    /// let expired_cred = AuthCredential::OAuth {
    ///     access_token: "token".to_string(),
    ///     refresh_token: Some("refresh".to_string()),
    ///     expires_at: Some(Utc::now() - Duration::hours(1)), // Expired 1 hour ago
    /// };
    ///
    /// assert!(expired_cred.is_expired());
    ///
    /// // PAT credentials never expire (from the credential's perspective)
    /// let pat_cred = AuthCredential::PersonalAccessToken {
    ///     token: "pat_token".to_string(),
    /// };
    ///
    /// assert!(!pat_cred.is_expired());
    /// ```
    ///
    /// # Notes
    ///
    /// - OAuth credentials without `expires_at` are considered non-expiring.
    /// - Server-side token revocation is not detected by this method.
    pub fn is_expired(&self) -> bool {
        match self {
            Self::OAuth {
                expires_at: Some(exp),
                ..
            } => *exp < chrono::Utc::now(),
            _ => false,
        }
    }

    /// Checks if this credential supports token refresh.
    ///
    /// Only OAuth credentials with a refresh token can be refreshed. All other
    /// credential types require full re-authentication when they become invalid.
    ///
    /// # Returns
    ///
    /// Returns `true` if the credential can be refreshed, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bitbucket_cli::auth::AuthCredential;
    ///
    /// let refreshable = AuthCredential::OAuth {
    ///     access_token: "token".to_string(),
    ///     refresh_token: Some("refresh_token".to_string()),
    ///     expires_at: None,
    /// };
    ///
    /// let non_refreshable = AuthCredential::OAuth {
    ///     access_token: "token".to_string(),
    ///     refresh_token: None,
    ///     expires_at: None,
    /// };
    ///
    /// assert!(refreshable.can_refresh());
    /// assert!(!non_refreshable.can_refresh());
    /// ```
    ///
    /// # Notes
    ///
    /// - Always check this before attempting to refresh a credential.
    /// - PAT, AppPassword, and Basic credentials cannot be refreshed.
    pub fn can_refresh(&self) -> bool {
        matches!(
            self,
            Self::OAuth {
                refresh_token: Some(_),
                ..
            }
        )
    }
}
