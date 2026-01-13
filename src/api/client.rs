//
//  bitbucket-cli
//  api/client.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! # HTTP Client Wrapper for Bitbucket API
//!
//! This module provides the core HTTP client for interacting with Bitbucket APIs.
//! It handles platform detection, authentication, and request/response serialization.
//!
//! ## Features
//!
//! - Automatic platform detection (Cloud vs Server/DC)
//! - Authentication header injection
//! - JSON serialization/deserialization
//! - Error handling with detailed messages
//! - Custom User-Agent header

use anyhow::Result;
use reqwest::{Client, StatusCode};
use serde::de::DeserializeOwned;

use crate::auth::AuthCredential;
use crate::config::HostConfig;

/// Parses a Bitbucket API error response and extracts a user-friendly message.
///
/// Bitbucket Cloud returns errors in the format:
/// ```json
/// {"type": "error", "error": {"message": "Human readable message"}}
/// ```
///
/// Bitbucket Server returns errors in the format:
/// ```json
/// {"errors": [{"message": "Human readable message"}]}
/// ```
///
/// This function attempts to extract the message from either format.
/// If parsing fails, it returns the original error body.
///
/// # Parameters
///
/// * `status` - The HTTP status code
/// * `body` - The raw error response body
///
/// # Returns
///
/// Returns an `anyhow::Error` with a clean, user-friendly message.
pub fn format_api_error(status: StatusCode, body: &str) -> anyhow::Error {
    // Try to parse as Bitbucket Cloud error format
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(body) {
        // Cloud format: {"type": "error", "error": {"message": "..."}}
        if let Some(message) = json
            .get("error")
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str())
        {
            return anyhow::anyhow!("{}", message);
        }

        // Server format: {"errors": [{"message": "..."}]}
        if let Some(message) = json
            .get("errors")
            .and_then(|e| e.as_array())
            .and_then(|arr| arr.first())
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str())
        {
            return anyhow::anyhow!("{}", message);
        }

        // Alternative Cloud format: {"error": {"detail": "..."}}
        if let Some(detail) = json
            .get("error")
            .and_then(|e| e.get("detail"))
            .and_then(|m| m.as_str())
        {
            return anyhow::anyhow!("{}", detail);
        }

        // Simple message format: {"message": "..."}
        if let Some(message) = json.get("message").and_then(|m| m.as_str()) {
            return anyhow::anyhow!("{}", message);
        }
    }

    // Fallback to raw body if parsing fails
    anyhow::anyhow!("API error ({}): {}", status, body)
}

/// Represents the type of Bitbucket platform being accessed.
///
/// This enum is used to determine API endpoints and behavior differences
/// between Bitbucket Cloud and Bitbucket Server/Data Center.
///
/// # Variants
///
/// * `Cloud` - Bitbucket Cloud at bitbucket.org, using API v2.0
/// * `Server` - Bitbucket Server/Data Center at a custom host, using API v1.0
///
/// # Example
///
/// ```rust
/// use bitbucket_cli::api::client::HostType;
///
/// let cloud = HostType::Cloud;
/// let server = HostType::Server { version: Some("7.21".to_string()) };
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum HostType {
    /// Bitbucket Cloud (bitbucket.org)
    ///
    /// Uses API v2.0 at `https://api.bitbucket.org/2.0`
    Cloud,

    /// Bitbucket Server/Data Center
    ///
    /// Uses API v1.0 at `https://<host>/rest/api/1.0`
    ///
    /// # Fields
    ///
    /// * `version` - Optional server version string (e.g., "7.21.0")
    Server {
        /// The server version, if known
        version: Option<String>,
    },
}

impl Default for HostType {
    /// Returns [`HostType::Cloud`] as the default.
    ///
    /// Cloud is the default because it's the most common use case
    /// and doesn't require additional configuration.
    fn default() -> Self {
        Self::Cloud
    }
}

/// The main HTTP client for interacting with Bitbucket APIs.
///
/// This client handles all HTTP communication with Bitbucket, including:
/// - Building request URLs based on platform type
/// - Applying authentication headers
/// - Serializing request bodies and deserializing responses
/// - Error handling for non-success status codes
///
/// # Creating a Client
///
/// Use one of the factory methods to create a client:
///
/// ```rust,no_run
/// use bitbucket_cli::api::BitbucketClient;
///
/// // For Bitbucket Cloud
/// let cloud_client = BitbucketClient::cloud()?;
///
/// // For Bitbucket Server/DC
/// let server_client = BitbucketClient::server("bitbucket.example.com")?;
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// # Authentication
///
/// Add authentication using the builder pattern:
///
/// ```rust,no_run
/// use bitbucket_cli::api::BitbucketClient;
/// use bitbucket_cli::auth::AuthCredential;
///
/// let client = BitbucketClient::cloud()?
///     .with_auth(AuthCredential::bearer("your-token"));
/// # Ok::<(), anyhow::Error>(())
/// ```
pub struct BitbucketClient {
    /// The underlying HTTP client
    http: Client,
    /// The host (e.g., "api.bitbucket.org" or "bitbucket.example.com")
    host: String,
    /// The platform type (Cloud or Server/DC)
    host_type: HostType,
    /// Optional authentication credentials
    auth: Option<AuthCredential>,
}

impl BitbucketClient {
    /// Creates a new client configured for Bitbucket Cloud.
    ///
    /// This creates a client that targets the Bitbucket Cloud API at
    /// `https://api.bitbucket.org/2.0`.
    ///
    /// # Returns
    ///
    /// Returns `Ok(BitbucketClient)` on success, or an error if the HTTP client
    /// could not be created.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use bitbucket_cli::api::BitbucketClient;
    ///
    /// let client = BitbucketClient::cloud()?;
    /// assert!(client.is_cloud());
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn cloud() -> Result<Self> {
        Ok(Self {
            http: Client::builder()
                .user_agent(format!("bb/{}", crate::VERSION))
                .build()?,
            host: "api.bitbucket.org".to_string(),
            host_type: HostType::Cloud,
            auth: None,
        })
    }

    /// Creates a new client configured for Bitbucket Server/Data Center.
    ///
    /// This creates a client that targets a Bitbucket Server/DC instance at
    /// `https://<host>/rest/api/1.0`.
    ///
    /// # Parameters
    ///
    /// * `host` - The hostname of the Bitbucket Server/DC instance
    ///   (e.g., "bitbucket.example.com")
    ///
    /// # Returns
    ///
    /// Returns `Ok(BitbucketClient)` on success, or an error if the HTTP client
    /// could not be created.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use bitbucket_cli::api::BitbucketClient;
    ///
    /// let client = BitbucketClient::server("bitbucket.mycompany.com")?;
    /// assert!(client.is_server());
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn server(host: &str) -> Result<Self> {
        Ok(Self {
            http: Client::builder()
                .user_agent(format!("bb/{}", crate::VERSION))
                .build()?,
            host: host.to_string(),
            host_type: HostType::Server { version: None },
            auth: None,
        })
    }

    /// Creates a client from a host configuration.
    ///
    /// This is typically used when loading configuration from the config file,
    /// where the host details are already stored.
    ///
    /// # Parameters
    ///
    /// * `config` - The host configuration containing host and API version details
    ///
    /// # Returns
    ///
    /// Returns `Ok(BitbucketClient)` on success, or an error if the HTTP client
    /// could not be created.
    ///
    /// # Platform Detection
    ///
    /// The platform type is automatically detected based on the host:
    /// - `bitbucket.org` or `api.bitbucket.org` → Cloud
    /// - Any other host → Server/DC
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use bitbucket_cli::api::BitbucketClient;
    /// use bitbucket_cli::config::HostConfig;
    ///
    /// let config = HostConfig {
    ///     host: "bitbucket.example.com".to_string(),
    ///     api_version: Some("1.0".to_string()),
    ///     ..Default::default()
    /// };
    ///
    /// let client = BitbucketClient::from_config(&config)?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn from_config(config: &HostConfig) -> Result<Self> {
        let host_type = if config.host == "bitbucket.org" || config.host == "api.bitbucket.org" {
            HostType::Cloud
        } else {
            HostType::Server {
                version: config.api_version.clone(),
            }
        };

        Ok(Self {
            http: Client::builder()
                .user_agent(format!("bb/{}", crate::VERSION))
                .build()?,
            host: config.host.clone(),
            host_type,
            auth: None,
        })
    }

    /// Sets the authentication credentials for this client.
    ///
    /// This method uses the builder pattern and returns `self` for chaining.
    ///
    /// # Parameters
    ///
    /// * `auth` - The authentication credentials to use for requests
    ///
    /// # Returns
    ///
    /// Returns `self` with authentication configured.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use bitbucket_cli::api::BitbucketClient;
    /// use bitbucket_cli::auth::AuthCredential;
    ///
    /// let client = BitbucketClient::cloud()?
    ///     .with_auth(AuthCredential::bearer("your-oauth-token"));
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn with_auth(mut self, auth: AuthCredential) -> Self {
        self.auth = Some(auth);
        self
    }

    /// Checks if this client is configured for Bitbucket Cloud.
    ///
    /// # Returns
    ///
    /// Returns `true` if this client targets Bitbucket Cloud, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use bitbucket_cli::api::BitbucketClient;
    ///
    /// let client = BitbucketClient::cloud()?;
    /// assert!(client.is_cloud());
    /// assert!(!client.is_server());
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn is_cloud(&self) -> bool {
        matches!(self.host_type, HostType::Cloud)
    }

    /// Checks if this client is configured for Bitbucket Server/DC.
    ///
    /// # Returns
    ///
    /// Returns `true` if this client targets Bitbucket Server/DC, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use bitbucket_cli::api::BitbucketClient;
    ///
    /// let client = BitbucketClient::server("bitbucket.example.com")?;
    /// assert!(client.is_server());
    /// assert!(!client.is_cloud());
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn is_server(&self) -> bool {
        matches!(self.host_type, HostType::Server { .. })
    }

    /// Returns the base URL for API requests.
    ///
    /// The base URL depends on the platform type:
    /// - Cloud: `https://api.bitbucket.org/2.0`
    /// - Server/DC: `https://<host>/rest/api/1.0`
    ///
    /// # Returns
    ///
    /// The base URL string for this client's target platform.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use bitbucket_cli::api::BitbucketClient;
    ///
    /// let cloud = BitbucketClient::cloud()?;
    /// assert_eq!(cloud.base_url(), "https://api.bitbucket.org/2.0");
    ///
    /// let server = BitbucketClient::server("bb.example.com")?;
    /// assert_eq!(server.base_url(), "https://bb.example.com/rest/api/1.0");
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn base_url(&self) -> String {
        match &self.host_type {
            HostType::Cloud => "https://api.bitbucket.org/2.0".to_string(),
            HostType::Server { .. } => format!("https://{}/rest/api/1.0", self.host),
        }
    }

    /// Makes an HTTP GET request to the specified path.
    ///
    /// The path is appended to the base URL. Authentication headers are
    /// automatically added if credentials were configured.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type to deserialize the response JSON into
    ///
    /// # Parameters
    ///
    /// * `path` - The API path (e.g., "/repositories/workspace/repo")
    ///
    /// # Returns
    ///
    /// Returns `Ok(T)` with the deserialized response on success, or an error
    /// if the request failed or the response could not be deserialized.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The network request fails
    /// - The response status is not successful (2xx)
    /// - The response body cannot be deserialized to type `T`
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use bitbucket_cli::api::BitbucketClient;
    /// use bitbucket_cli::api::cloud::Repository;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let client = BitbucketClient::cloud()?;
    /// let repo: Repository = client.get("/repositories/myworkspace/myrepo").await?;
    /// println!("Repository: {}", repo.name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", self.base_url(), path);
        let mut request = self.http.get(&url);

        if let Some(auth) = &self.auth {
            request = auth.apply_to_request(request);
        }

        let response = request.send().await?;
        let status = response.status();

        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(format_api_error(status, &text));
        }

        Ok(response.json().await?)
    }

    /// Makes an HTTP POST request to the specified path with a JSON body.
    ///
    /// The path is appended to the base URL. The body is serialized to JSON.
    /// Authentication headers are automatically added if credentials were configured.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type to deserialize the response JSON into
    /// * `B` - The type of the request body (must implement `Serialize`)
    ///
    /// # Parameters
    ///
    /// * `path` - The API path (e.g., "/repositories/workspace/repo")
    /// * `body` - The request body to serialize as JSON
    ///
    /// # Returns
    ///
    /// Returns `Ok(T)` with the deserialized response on success, or an error
    /// if the request failed.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The request body cannot be serialized
    /// - The network request fails
    /// - The response status is not successful (2xx)
    /// - The response body cannot be deserialized to type `T`
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use bitbucket_cli::api::BitbucketClient;
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[derive(Serialize)]
    /// struct CreateRepo { name: String, is_private: bool }
    ///
    /// #[derive(Deserialize)]
    /// struct Repository { name: String }
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let client = BitbucketClient::cloud()?;
    /// let body = CreateRepo { name: "myrepo".to_string(), is_private: true };
    /// let repo: Repository = client.post("/repositories/myworkspace/myrepo", &body).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn post<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url(), path);
        let mut request = self.http.post(&url).json(body);

        if let Some(auth) = &self.auth {
            request = auth.apply_to_request(request);
        }

        let response = request.send().await?;
        let status = response.status();

        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(format_api_error(status, &text));
        }

        Ok(response.json().await?)
    }

    /// Makes an HTTP DELETE request to the specified path.
    ///
    /// The path is appended to the base URL. Authentication headers are
    /// automatically added if credentials were configured.
    ///
    /// # Parameters
    ///
    /// * `path` - The API path (e.g., "/repositories/workspace/repo")
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if the request failed.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The network request fails
    /// - The response status is not successful (2xx)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use bitbucket_cli::api::BitbucketClient;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let client = BitbucketClient::cloud()?;
    /// client.delete("/repositories/myworkspace/myrepo").await?;
    /// println!("Repository deleted");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete(&self, path: &str) -> Result<()> {
        let url = format!("{}{}", self.base_url(), path);
        let mut request = self.http.delete(&url);

        if let Some(auth) = &self.auth {
            request = auth.apply_to_request(request);
        }

        let response = request.send().await?;
        let status = response.status();

        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(format_api_error(status, &text));
        }

        Ok(())
    }

    /// Makes an HTTP PUT request to the specified path with a JSON body.
    ///
    /// The path is appended to the base URL. The body is serialized to JSON.
    /// Authentication headers are automatically added if credentials were configured.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type to deserialize the response JSON into
    /// * `B` - The type of the request body (must implement `Serialize`)
    ///
    /// # Parameters
    ///
    /// * `path` - The API path (e.g., "/repositories/workspace/repo/issues/1")
    /// * `body` - The request body to serialize as JSON
    ///
    /// # Returns
    ///
    /// Returns `Ok(T)` with the deserialized response on success, or an error
    /// if the request failed.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The request body cannot be serialized
    /// - The network request fails
    /// - The response status is not successful (2xx)
    /// - The response body cannot be deserialized to type `T`
    pub async fn put<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url(), path);
        let mut request = self.http.put(&url).json(body);

        if let Some(auth) = &self.auth {
            request = auth.apply_to_request(request);
        }

        let response = request.send().await?;
        let status = response.status();

        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(format_api_error(status, &text));
        }

        Ok(response.json().await?)
    }
}
