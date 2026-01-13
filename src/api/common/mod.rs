//
//  bitbucket-cli
//  api/common/mod.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! Common API Types for Bitbucket Cloud and Server
//!
//! This module provides shared types and utilities used across both Bitbucket Cloud
//! and Bitbucket Server/Data Center API implementations. It includes error handling,
//! response wrappers, and common data structures.
//!
//! # Overview
//!
//! The common module serves as the foundation for API interactions, providing:
//!
//! - [`ApiError`] - Unified error type for all API operations
//! - [`ApiResponse`] - Generic wrapper for API response data
//! - [`Link`] - HATEOAS-style link representation
//! - [`UserRef`] - Lightweight user reference for cross-API compatibility
//! - Pagination types (re-exported from [`pagination`] submodule)
//!
//! # Example
//!
//! ```rust
//! use bitbucket_cli::api::common::{ApiError, ApiResponse, UserRef};
//!
//! // Handle API errors
//! fn handle_result<T>(result: Result<T, ApiError>) {
//!     match result {
//!         Ok(data) => println!("Success!"),
//!         Err(ApiError::AuthRequired) => println!("Please authenticate first"),
//!         Err(ApiError::NotFound(resource)) => println!("Resource not found: {}", resource),
//!         Err(e) => println!("Error: {}", e),
//!     }
//! }
//! ```
//!
//! # Notes
//!
//! - All types implement `Debug` for easy inspection
//! - Serialization/deserialization is handled via `serde` for JSON compatibility
//! - Field aliases are used to normalize differences between Cloud and Server APIs

use serde::{Deserialize, Serialize};
use thiserror::Error;

mod pagination;

pub use pagination::*;

/// Unified error type for all Bitbucket API operations.
///
/// `ApiError` provides a comprehensive set of error variants covering common
/// failure scenarios when interacting with Bitbucket APIs. It implements the
/// standard `Error` trait via `thiserror` for ergonomic error handling.
///
/// # Variants
///
/// | Variant | Description | HTTP Status |
/// |---------|-------------|-------------|
/// | `AuthRequired` | No authentication credentials provided | 401 |
/// | `AuthFailed` | Invalid or expired credentials | 401 |
/// | `NotFound` | Requested resource does not exist | 404 |
/// | `RateLimited` | Too many requests, retry later | 429 |
/// | `Forbidden` | Insufficient permissions | 403 |
/// | `BadRequest` | Invalid request parameters | 400 |
/// | `ServerError` | Internal server error | 5xx |
/// | `Network` | Network connectivity issues | N/A |
/// | `Unknown` | Unexpected or unclassified errors | N/A |
///
/// # Example
///
/// ```rust
/// use bitbucket_cli::api::common::ApiError;
///
/// fn fetch_repository() -> Result<(), ApiError> {
///     // Simulate a not found error
///     Err(ApiError::NotFound("repository 'my-repo'".to_string()))
/// }
///
/// match fetch_repository() {
///     Ok(_) => println!("Repository fetched successfully"),
///     Err(ApiError::NotFound(resource)) => {
///         eprintln!("Could not find: {}", resource);
///     }
///     Err(e) => eprintln!("Unexpected error: {}", e),
/// }
/// ```
///
/// # Notes
///
/// - The `Network` variant automatically converts from `reqwest::Error`
/// - Error messages are designed to be user-friendly and actionable
/// - Consider using the `?` operator for ergonomic error propagation
#[derive(Error, Debug)]
pub enum ApiError {
    /// Authentication credentials are required but not provided.
    ///
    /// This error occurs when attempting to access a protected resource
    /// without any authentication token or credentials configured.
    #[error("Authentication required")]
    AuthRequired,

    /// Authentication failed due to invalid or expired credentials.
    ///
    /// # Parameters
    ///
    /// - `0` - Detailed reason for the authentication failure
    #[error("Authentication failed: {0}")]
    AuthFailed(String),

    /// The requested resource was not found.
    ///
    /// This typically indicates a 404 HTTP response, meaning the repository,
    /// pull request, or other resource does not exist or is not accessible.
    ///
    /// # Parameters
    ///
    /// - `0` - Description of the resource that was not found
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// API rate limit has been exceeded.
    ///
    /// Bitbucket APIs enforce rate limits to prevent abuse. When this error
    /// occurs, the client should wait before retrying. Check response headers
    /// for retry timing information.
    #[error("Rate limit exceeded")]
    RateLimited,

    /// Access to the resource is forbidden.
    ///
    /// The authenticated user does not have sufficient permissions to perform
    /// the requested operation. This maps to HTTP 403 responses.
    ///
    /// # Parameters
    ///
    /// - `0` - Description of the forbidden action or resource
    #[error("Permission denied: {0}")]
    Forbidden(String),

    /// The request was malformed or contained invalid parameters.
    ///
    /// This typically indicates a client-side error such as missing required
    /// fields, invalid field values, or malformed JSON.
    ///
    /// # Parameters
    ///
    /// - `0` - Description of what was wrong with the request
    #[error("Bad request: {0}")]
    BadRequest(String),

    /// An internal server error occurred on the Bitbucket server.
    ///
    /// This indicates a problem on the server side (HTTP 5xx responses).
    /// These are typically transient and may succeed on retry.
    ///
    /// # Parameters
    ///
    /// - `0` - Error message or details from the server
    #[error("Server error: {0}")]
    ServerError(String),

    /// A network-level error occurred during the request.
    ///
    /// This covers connection failures, timeouts, DNS resolution errors,
    /// and other transport-layer issues.
    ///
    /// # Parameters
    ///
    /// - `0` - The underlying `reqwest::Error` with network details
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// An unknown or unexpected error occurred.
    ///
    /// This is a catch-all for errors that don't fit other categories.
    /// The message should provide enough context for debugging.
    ///
    /// # Parameters
    ///
    /// - `0` - Description of the unknown error
    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Generic wrapper for API response data.
///
/// `ApiResponse` provides a simple container for successful API responses,
/// wrapping the typed response data. This structure allows for consistent
/// handling of API responses across different endpoints.
///
/// # Type Parameters
///
/// - `T` - The type of data contained in the response
///
/// # Example
///
/// ```rust
/// use bitbucket_cli::api::common::ApiResponse;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Repository {
///     name: String,
///     slug: String,
/// }
///
/// // Parse an API response
/// let json = r#"{"data": {"name": "My Repo", "slug": "my-repo"}}"#;
/// let response: ApiResponse<Repository> = serde_json::from_str(json).unwrap();
/// println!("Repository: {}", response.data.name);
/// ```
///
/// # Notes
///
/// - The `data` field contains the actual response payload
/// - Implements `Serialize` and `Deserialize` for JSON compatibility
/// - Use [`PaginatedResponse`] for list endpoints with pagination
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// The response payload data.
    ///
    /// Contains the actual data returned by the API endpoint,
    /// deserialized into the specified type `T`.
    pub data: T,
}

/// HATEOAS-style link for API resource navigation.
///
/// `Link` represents a hyperlink reference commonly found in REST APIs that
/// follow HATEOAS (Hypermedia as the Engine of Application State) principles.
/// Bitbucket APIs include links to related resources and actions.
///
/// # Fields
///
/// | Field | Type | Description |
/// |-------|------|-------------|
/// | `href` | `String` | The URL of the linked resource |
/// | `name` | `Option<String>` | Optional descriptive name for the link |
///
/// # Example
///
/// ```rust
/// use bitbucket_cli::api::common::Link;
///
/// let link = Link {
///     href: "https://api.bitbucket.org/2.0/repositories/owner/repo".to_string(),
///     name: Some("self".to_string()),
/// };
///
/// println!("Navigate to: {}", link.href);
/// if let Some(name) = &link.name {
///     println!("Link type: {}", name);
/// }
/// ```
///
/// # Notes
///
/// - Common link names include: `self`, `html`, `avatar`, `clone`
/// - The `href` field is always present and contains a valid URL
/// - The `name` field defaults to `None` if not provided in the response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    /// The URL of the linked resource.
    ///
    /// This is always a fully-qualified URL that can be used directly
    /// for navigation or subsequent API calls.
    pub href: String,

    /// Optional descriptive name for the link.
    ///
    /// When present, this describes the relationship or purpose of the link.
    /// Common values include `self`, `html`, `commits`, `pullrequests`, etc.
    #[serde(default)]
    pub name: Option<String>,
}

/// Lightweight user reference for cross-API compatibility.
///
/// `UserRef` provides a minimal representation of a Bitbucket user that works
/// across both Cloud and Server APIs. It uses field aliases to normalize the
/// different field names used by each API variant.
///
/// # Field Mapping
///
/// | UserRef Field | Cloud API Field | Server API Field |
/// |---------------|-----------------|------------------|
/// | `uuid` | `account_id` | `uuid` |
/// | `name` | `display_name` | `display_name` |
/// | `username` | `nickname` | `nickname` |
///
/// # Example
///
/// ```rust
/// use bitbucket_cli::api::common::UserRef;
///
/// // Works with both Cloud and Server API responses
/// let cloud_json = r#"{
///     "account_id": "557058:12345678-1234-1234-1234-123456789012",
///     "display_name": "John Doe",
///     "nickname": "johnd"
/// }"#;
///
/// let user: UserRef = serde_json::from_str(cloud_json).unwrap();
/// println!("User: {} (@{})", user.name, user.username.unwrap_or_default());
/// ```
///
/// # Notes
///
/// - The `uuid` field may be `None` for anonymous or system users
/// - The `name` field (display name) is always present
/// - The `username` field may be `None` if the user hasn't set a nickname
/// - Field aliases ensure seamless deserialization from either API variant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRef {
    /// Unique identifier for the user (Cloud: account_id format).
    ///
    /// For Bitbucket Cloud, this is the `account_id`.
    /// May be `None` for anonymous or system-generated references.
    #[serde(default, rename = "account_id")]
    pub account_id: Option<String>,

    /// UUID identifier for the user.
    ///
    /// For Bitbucket Cloud, this is the `uuid` field (with curly braces).
    /// For Bitbucket Server, this may also be present.
    /// May be `None` for anonymous or system-generated references.
    #[serde(default)]
    pub uuid: Option<String>,

    /// Display name of the user.
    ///
    /// The human-readable name as configured in the user's profile.
    /// This field is always present and maps from `display_name` in both APIs.
    #[serde(alias = "display_name")]
    pub name: String,

    /// Username or nickname of the user.
    ///
    /// The short identifier used in URLs and @mentions.
    /// Maps from `nickname` in both Cloud and Server APIs.
    /// May be `None` if the user hasn't configured a username.
    #[serde(alias = "nickname")]
    pub username: Option<String>,
}
