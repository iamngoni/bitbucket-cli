//
//  bitbucket-cli
//  api/cloud/mod.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! Bitbucket Cloud API v2.0 implementation.
//!
//! This module provides type-safe Rust bindings for the Bitbucket Cloud REST API v2.0.
//! It contains data structures for serialization/deserialization of API requests and
//! responses, organized by resource type.
//!
//! # Module Organization
//!
//! The Cloud API is organized into the following submodules:
//!
//! - [`repositories`] - Repository management (create, read, update, delete)
//! - [`pullrequests`] - Pull request operations (create, merge, review)
//! - [`workspaces`] - Workspace and team management
//! - [`pipelines`] - CI/CD pipeline operations
//! - [`issues`] - Issue tracking functionality
//!
//! # Example
//!
//! ```rust,no_run
//! use bitbucket_cli::api::cloud::{Repository, CreateRepositoryRequest};
//!
//! // Create a new repository request
//! let request = CreateRepositoryRequest {
//!     name: "my-new-repo".to_string(),
//!     description: Some("A new repository".to_string()),
//!     is_private: Some(true),
//!     project: None,
//!     language: Some("rust".to_string()),
//! };
//! ```
//!
//! # API Version
//!
//! This module targets Bitbucket Cloud API v2.0. For Bitbucket Server/Data Center,
//! see the [`server`](super::server) module.
//!
//! # Notes
//!
//! - All timestamps are in ISO 8601 format
//! - UUIDs are returned with curly braces (e.g., `{123e4567-e89b-...}`)
//! - Pagination uses cursor-based navigation with `next` and `previous` links

pub mod issues;
pub mod pipelines;
pub mod pullrequests;
pub mod repositories;
pub mod workspaces;

// Re-export common types
pub use repositories::*;
