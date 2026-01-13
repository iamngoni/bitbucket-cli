//
//  bitbucket-cli
//  api/mod.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! # API Client Layer
//!
//! This module provides HTTP client implementations for interacting with Bitbucket's REST APIs.
//!
//! ## Supported Platforms
//!
//! - **Bitbucket Cloud**: API v2.0 at `api.bitbucket.org`
//! - **Bitbucket Server/Data Center**: API v1.0 at your custom host
//!
//! ## Architecture
//!
//! The API layer is organized as follows:
//!
//! - [`client`]: Core HTTP client with authentication and request handling
//! - [`cloud`]: Cloud-specific API implementations (repositories, PRs, pipelines)
//! - [`server`]: Server/DC-specific API implementations (repositories, PRs, projects)
//! - [`common`]: Shared types (pagination, errors, user references)
//!
//! ## Usage
//!
//! ### Creating a Client
//!
//! ```rust,no_run
//! use bitbucket_cli::api::BitbucketClient;
//! use bitbucket_cli::auth::AuthCredential;
//!
//! // Cloud client
//! let cloud_client = BitbucketClient::cloud()
//!     .expect("Failed to create client")
//!     .with_auth(AuthCredential::bearer("your-token"));
//!
//! // Server/DC client
//! let server_client = BitbucketClient::server("bitbucket.example.com")
//!     .expect("Failed to create client")
//!     .with_auth(AuthCredential::bearer("your-pat"));
//! ```
//!
//! ## Error Handling
//!
//! API errors are returned as [`ApiError`] variants, which map to common HTTP error scenarios:
//!
//! - `AuthRequired`: 401 Unauthorized
//! - `Forbidden`: 403 Forbidden
//! - `NotFound`: 404 Not Found
//! - `RateLimited`: 429 Too Many Requests
//! - `ServerError`: 5xx Server Errors

/// Core HTTP client wrapper for Bitbucket APIs.
///
/// Provides the [`BitbucketClient`] struct which handles:
/// - Platform detection (Cloud vs Server/DC)
/// - Authentication header injection
/// - Request/response serialization
/// - Error handling and status code mapping
pub mod client;

/// Bitbucket Cloud API v2.0 implementation.
///
/// Contains modules for Cloud-specific resources:
/// - [`cloud::repositories`]: Repository CRUD operations
/// - [`cloud::pullrequests`]: Pull request management
/// - [`cloud::workspaces`]: Workspace operations
/// - [`cloud::pipelines`]: CI/CD pipeline operations
/// - [`cloud::issues`]: Issue tracker operations
pub mod cloud;

/// Bitbucket Server/Data Center API v1.0 implementation.
///
/// Contains modules for Server/DC-specific resources:
/// - [`server::repositories`]: Repository CRUD operations
/// - [`server::pullrequests`]: Pull request management
/// - [`server::projects`]: Project operations
pub mod server;

/// Common types shared between Cloud and Server APIs.
///
/// Includes:
/// - [`ApiError`]: Standardized error types
/// - [`ApiResponse`]: Generic response wrapper
/// - [`PaginatedResponse`]: Cloud pagination format
/// - [`ServerPaginatedResponse`]: Server/DC pagination format
/// - [`Link`]: HATEOAS link type
/// - [`UserRef`]: User reference type
pub mod common;

/// Re-export of the main Bitbucket API client.
///
/// This is the primary entry point for making API requests to either
/// Bitbucket Cloud or Server/Data Center.
pub use client::BitbucketClient;

/// Re-export of common API types.
///
/// - [`ApiError`]: Error type for API operations
/// - [`ApiResponse`]: Generic wrapper for API responses
pub use common::{ApiError, ApiResponse};
