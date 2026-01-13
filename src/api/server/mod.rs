//
//  bitbucket-cli
//  api/server/mod.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! # Bitbucket Server/Data Center API v1.0
//!
//! This module provides the implementation for interacting with Bitbucket Server
//! (also known as Bitbucket Data Center) REST API v1.0. Unlike Bitbucket Cloud,
//! Server/DC instances are self-hosted and use a different API structure.
//!
//! ## Module Organization
//!
//! The server API is organized into the following submodules:
//!
//! - [`repositories`] - Repository management (create, list, clone links)
//! - [`pullrequests`] - Pull request operations (create, merge, review)
//! - [`projects`] - Project management (create, update, list)
//!
//! ## API Differences from Cloud
//!
//! Bitbucket Server/DC API differs from Cloud in several ways:
//!
//! - Uses project keys instead of workspace UUIDs
//! - Different authentication mechanisms (typically personal access tokens)
//! - Repository URLs follow pattern: `/rest/api/1.0/projects/{projectKey}/repos/{repoSlug}`
//! - Timestamps are Unix milliseconds instead of ISO 8601 strings
//!
//! ## Example
//!
//! ```rust,ignore
//! use bitbucket_cli::api::server::{Repository, Project};
//!
//! // Types are re-exported for convenience
//! let repo: Repository = fetch_repository("PROJECT", "my-repo").await?;
//! println!("Repository: {} ({})", repo.name, repo.slug);
//! ```
//!
//! ## Notes
//!
//! - All types implement `Debug`, `Clone`, `Serialize`, and `Deserialize`
//! - Optional fields use `Option<T>` and default to `None` during deserialization
//! - Boolean fields default to `false` when not present in the API response

pub mod projects;
pub mod pullrequests;
pub mod repositories;

// Re-export common types (avoiding ambiguous re-exports)
pub use projects::{Project, ProjectLinks};
pub use repositories::{CloneLink, Repository};
